use std::any::TypeId;
use std::marker::PhantomData;
use std::mem::ManuallyDrop;
use std::time::Duration;

use crate::function::process::IntoProcess;
use crate::serializer::{Bincode, CanSerialize};
use crate::{host, Mailbox, MailboxResult, Process, ProcessConfig, Tag};

/// A value that the protocol captures from the parent process.
///
/// A protocol needs to capture more information from the parent than just the
/// value passed in by the user (`capture`). For a protocol to work it needs to
/// have a reference to the parent, so it knows where to send messages to. And
/// it needs a unique tag inside the parent so that protocol messages don't mix
/// with other messages received by the parent.
#[derive(serde::Serialize, serde::Deserialize, Debug, Hash)]
pub struct ProtocolCapture<C> {
    process: Process<()>,
    tag: Tag,
    capture: C,
}

/// A `Protocol` is a specific type of [`Process`](crate::Process).
///
/// It uses session types to check during compile time that all messages
/// exchanged between two processes are in the correct order and of the correct
/// type.
#[derive(Debug, Hash)]
pub struct Protocol<P: 'static, S = Bincode, Z: 'static = ()> {
    id: u64,
    node_id: u64,
    tag: Tag,
    phantom: PhantomData<(P, S, Z)>,
}

impl<P: 'static, S, Z: 'static> Drop for Protocol<P, S, Z> {
    fn drop(&mut self) {
        if TypeId::of::<P>() != TypeId::of::<End>() && TypeId::of::<P>() != TypeId::of::<TaskEnd>()
        {
            panic!(
                "Protocol prematurely dropped, before reaching the `End` or `TaskEnd` state (currently: {}).",
                std::any::type_name::<P>()
            );
        }
    }
}

impl<P, S, Z> Protocol<P, S, Z> {
    /// Turn a process into a protocol
    fn from_process<M, S2>(process: Process<M, S2>, tag: Tag) -> Self {
        // The transformation shouldn't drop the process resource.
        let process = ManuallyDrop::new(process);
        Self {
            id: process.id(),
            node_id: process.node_id(),
            tag,
            phantom: PhantomData,
        }
    }

    /// Cast the protocol to another type.
    fn cast<P2, Z2>(self) -> Protocol<P2, S, Z2> {
        // Don't drop the session yet.
        let self_ = ManuallyDrop::new(self);
        Protocol {
            id: self_.id,
            node_id: self_.node_id,
            tag: self_.tag,
            phantom: PhantomData,
        }
    }
}

impl<P, A, S, Z> Protocol<Send<A, P>, S, Z>
where
    S: CanSerialize<A>,
{
    /// Send a value of type `A` over the session. Returns a session with
    /// protocol `P`.
    #[must_use]
    pub fn send(self, message: A) -> Protocol<P, S, Z> {
        // Don't drop the session yet.
        let self_ = ManuallyDrop::new(self);
        // Temporarily cast to right process type.
        let process: Process<A, S> = unsafe { Process::new(self_.node_id, self_.id) };
        process.tag_send(self_.tag, message);
        Protocol::from_process(process, self_.tag)
    }
}

impl<P, A, S, Z> Protocol<Recv<A, P>, S, Z>
where
    S: CanSerialize<A>,
{
    /// Receives a value of type `A` from the session. Returns a tuple
    /// containing the resulting session and the received value.
    #[must_use]
    pub fn receive(self) -> (Protocol<P, S, Z>, A) {
        // Temporarily cast to right mailbox type.
        let mailbox: Mailbox<A, S> = unsafe { Mailbox::new() };
        let received = mailbox.tag_receive(&[self.tag]);
        (self.cast(), received)
    }
}

impl<A, S, Z> Protocol<Recv<A, TaskEnd>, S, Z>
where
    S: CanSerialize<A>,
{
    /// A task is a special case of a protocol spawned with the `spawn!(@task
    /// ...)` macro. It only returns one value.
    #[must_use]
    pub fn result(self) -> A {
        // Temporarily cast to right mailbox type.
        let mailbox: Mailbox<A, S> = unsafe { Mailbox::new() };
        let result = mailbox.tag_receive(&[self.tag]);
        let _: Protocol<TaskEnd, S, Z> = self.cast(); // Only `End` protocols can be dropped
        result
    }

    /// A task is a special case of a protocol spawned with the `spawn!(@task
    /// ...)` macro. It only returns one value.
    pub fn result_timeout(self, duration: Duration) -> MailboxResult<A> {
        // Temporarily cast to right mailbox type.
        let mailbox: Mailbox<A, S> = unsafe { Mailbox::new() };
        let result = mailbox.tag_receive_timeout(&[self.tag], duration);
        let _: Protocol<TaskEnd, S, Z> = self.cast(); // Only `End` protocols can be dropped
        result
    }
}

impl<P, Q, S, Z> Protocol<Choose<P, Q>, S, Z>
where
    S: CanSerialize<bool>,
{
    /// Perform an active choice, selecting protocol `P`.
    #[must_use]
    pub fn select_left(self) -> Protocol<P, S, Z> {
        // Don't drop the session yet.
        let self_ = ManuallyDrop::new(self);
        // Temporarily cast to right process type.
        let process: Process<bool, S> = unsafe { Process::new(self_.node_id, self_.id) };
        process.tag_send(self_.tag, true);
        Protocol::from_process(process, self_.tag)
    }

    /// Perform an active choice, selecting protocol `Q`.
    #[must_use]
    pub fn select_right(self) -> Protocol<Q, S, Z> {
        // Don't drop the session yet.
        let self_ = ManuallyDrop::new(self);
        // Temporarily cast to right process type.
        let process: Process<bool, S> = unsafe { Process::new(self_.node_id, self_.id) };
        process.tag_send(self_.tag, false);
        Protocol::from_process(process, self_.tag)
    }
}

impl<P, Q, S, Z> Protocol<Offer<P, Q>, S, Z>
where
    S: CanSerialize<bool>,
{
    /// Passive choice. This allows the other end of the session to select one
    /// of two options for continuing the protocol: either `P` or `Q`.
    #[must_use]
    pub fn offer(self) -> Branch<Protocol<P, S, Z>, Protocol<Q, S, Z>> {
        unsafe {
            // Temporarily cast to right mailbox type.
            let mailbox: Mailbox<bool, S> = Mailbox::new();
            if mailbox.receive() {
                Branch::Left(self.cast())
            } else {
                Branch::Right(self.cast())
            }
        }
    }
}

impl<P, S, Z> Protocol<Rec<P>, S, Z> {
    /// Repeat Protocol
    #[must_use]
    pub fn repeat(self) -> Protocol<P, S, Protocol<Rec<P>, S, Z>> {
        self.cast()
    }
}

impl<P2, S, Z> Protocol<Pop, S, Protocol<P2, S, Z>> {
    /// Pop
    #[must_use]
    pub fn pop(self) -> Protocol<P2, S, Z> {
        self.cast()
    }
}

impl<P, S, Z> From<Protocol<Rec<P>, S, Z>> for Protocol<P, S, Z> {
    fn from(p: Protocol<Rec<P>, S, Z>) -> Self {
        p.cast()
    }
}

/// A special case of the protocol with a `result()` function.
pub struct TaskEnd;

/// End of communication session
pub struct End;

/// Receive `A`, then `P`
pub struct Recv<A, P>(PhantomData<(A, P)>);

/// Send `A`, then `P`
pub struct Send<A, P>(PhantomData<(A, P)>);

/// Active choice between `P` and `Q`
pub struct Choose<P, Q>(PhantomData<(P, Q)>);

/// Passive choice (offer) between `P` and `Q`
pub struct Offer<P, Q>(PhantomData<(P, Q)>);

/// Allows recursively calling a protocol
pub struct Rec<P>(PhantomData<P>);

/// Allows recursing
pub struct Pop;

/// The HasDual trait defines the dual relationship between protocols.
///
/// Any valid protocol has a corresponding dual.
///
/// This trait is sealed and cannot be implemented outside session-types.
pub trait HasDual: private::Sealed {
    type Dual: HasDual;
}

impl HasDual for TaskEnd {
    type Dual = TaskEnd;
}

impl HasDual for End {
    type Dual = End;
}

impl<A, P: HasDual> HasDual for Send<A, P> {
    type Dual = Recv<A, P::Dual>;
}

impl<A, P: HasDual> HasDual for Recv<A, P> {
    type Dual = Send<A, P::Dual>;
}

impl<P: HasDual, Q: HasDual> HasDual for Choose<P, Q> {
    type Dual = Offer<P::Dual, Q::Dual>;
}

impl<P: HasDual, Q: HasDual> HasDual for Offer<P, Q> {
    type Dual = Choose<P::Dual, Q::Dual>;
}

impl<P: HasDual> HasDual for Rec<P> {
    type Dual = Rec<P::Dual>;
}

impl HasDual for Pop {
    type Dual = Pop;
}

pub enum Branch<L, R> {
    Left(L),
    Right(R),
}

mod private {
    use super::*;
    pub trait Sealed {}

    // Impl for all exported protocol types
    impl Sealed for TaskEnd {}
    impl Sealed for End {}
    impl<A, P> Sealed for Send<A, P> {}
    impl<A, P> Sealed for Recv<A, P> {}
    impl<P, Q> Sealed for Choose<P, Q> {}
    impl<P, Q> Sealed for Offer<P, Q> {}
    impl<P> Sealed for Rec<P> {}
    impl Sealed for Pop {}
}

impl<P, S, Z> IntoProcess<P, S> for Protocol<P, S, Z>
where
    P: HasDual,
{
    type Process = Protocol<<P as HasDual>::Dual, S, Z>;

    fn spawn<C>(
        capture: C,
        entry: fn(C, Protocol<P, S, Z>),
        link: Option<Tag>,
        config: Option<&ProcessConfig>,
        node: Option<u64>,
    ) -> Self::Process
    where
        S: CanSerialize<ProtocolCapture<C>>,
    {
        let entry = entry as usize as i32;
        let node_id = node.unwrap_or_else(host::node_id);

        // The `type_helper_wrapper` function is used here to create a pointer to a
        // function with generic types C, P & S. We can only send pointer data
        // across processes and this is the only way the Rust compiler will let
        // us transfer this information into the new process.
        match host::spawn(node, config, link, type_helper_wrapper::<C, P, S, Z>, entry) {
            Ok(id) => {
                // Use unique tag so that protocol messages are separated from regular messages.
                let tag = Tag::new();
                // Create reference to self
                let this = unsafe { Process::<()>::new(host::node_id(), host::process_id()) };
                let capture = ProtocolCapture {
                    process: this,
                    tag,
                    capture,
                };
                let child = unsafe { Process::<ProtocolCapture<C>, S>::new(node_id, id) };

                child.send(capture);
                Protocol::from_process(child, tag)
            }
            Err(err) => panic!("Failed to spawn a process: {}", err),
        }
    }
}

/// Wrapper function to help transfer the generic types C, P, S & Z into the new
/// process.
fn type_helper_wrapper<C, P, S, Z>(function: i32)
where
    S: CanSerialize<ProtocolCapture<C>>,
    P: HasDual + 'static,
    Z: 'static,
{
    let p_capture = unsafe { Mailbox::<ProtocolCapture<C>, S>::new() }.receive();
    let capture = p_capture.capture;
    let protocol = Protocol::from_process(p_capture.process, p_capture.tag);
    let function: fn(C, Protocol<P, S, Z>) = unsafe { std::mem::transmute(function as usize) };
    function(capture, protocol);
}

#[cfg(test)]
mod tests {
    use lunatic_test::test;

    use super::*;

    type AddProtocol = Recv<i32, Recv<i32, Send<i32, End>>>;

    #[test]
    fn protocol() {
        let child = Process::spawn_link(1, |capture: i32, protocol: Protocol<AddProtocol>| {
            assert_eq!(capture, 1);
            let (protocol, a) = protocol.receive();
            let (protocol, b) = protocol.receive();
            let _ = protocol.send(capture + a + b);
        });

        let child = child.send(2);
        let child = child.send(2);
        let (_, result) = child.receive();
        assert_eq!(result, 5);
    }
}
