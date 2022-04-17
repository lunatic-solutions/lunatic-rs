// This implementation was inspired by https://github.com/Munksgaard/session-types.

use std::{marker::PhantomData, mem::ManuallyDrop};

use crate::{serializer::Serializer, Mailbox, Process, Protocol, Resource, Tag};

// Cast session to any other possible type.
impl<P, S> Protocol<P, S> {
    /// Create a Protocol with a specific type.
    ///
    /// ### Safety
    ///
    /// It's not safe to mix different types of sessions inside one process. This function should
    /// never be used directly.
    pub(crate) unsafe fn new(connection: Process<()>, tag: Tag) -> Self {
        let id = connection.id();
        let _perserve = ManuallyDrop::new(connection);
        Self {
            id,
            tag,
            phantom: PhantomData {},
        }
    }

    fn cast<P2>(self) -> Protocol<P2, S> {
        let id = self.id;
        let tag = self.tag;
        // Don't drop the session yet.
        let _save = ManuallyDrop::new(self);
        Protocol {
            id,
            tag,
            phantom: PhantomData,
        }
    }
}

impl<P, A, S> Protocol<Send<A, P>, S>
where
    S: Serializer<A>,
{
    /// Send a value of type `A` over the session. Returns a session with protocol `P`.
    #[must_use]
    pub fn send(self, message: A) -> Protocol<P, S> {
        // Temporarily cast to right process type.
        let process: Process<A, S> = unsafe { Process::from_id(self.id) };
        process.tag_send(self.tag, message);
        // Don't drop the process resource
        let _save = ManuallyDrop::new(process);
        self.cast()
    }
}

impl<P, A, S> Protocol<Recv<A, P>, S>
where
    S: Serializer<A>,
{
    /// Receives a value of type `A` from the session. Returns a tuple containing the resulting
    /// session and the received value.
    #[must_use]
    pub fn receive(self) -> (Protocol<P, S>, A) {
        // Temporarily cast to right mailbox type.
        let mailbox: Mailbox<A, S> = unsafe { Mailbox::new() };
        let received = mailbox.tag_receive(Some(&[self.tag]));
        (self.cast(), received)
    }
}

impl<P, Q, S> Protocol<Choose<P, Q>, S>
where
    S: Serializer<bool>,
{
    /// Perform an active choice, selecting protocol `P`.
    #[must_use]
    pub fn select_left(self) -> Protocol<P, S> {
        // Temporarily cast to right process type.
        let process: Process<bool, S> = unsafe { Process::from_id(self.id) };
        process.tag_send(self.tag, true);
        // Don't drop the process resource
        let _save = ManuallyDrop::new(process);
        self.cast()
    }

    /// Perform an active choice, selecting protocol `Q`.
    #[must_use]
    pub fn select_right(self) -> Protocol<Q, S> {
        // Temporarily cast to right process type.
        let process: Process<bool, S> = unsafe { Process::from_id(self.id) };
        process.tag_send(self.tag, false);
        // Don't drop the process resource
        let _save = ManuallyDrop::new(process);
        self.cast()
    }
}

impl<P, Q, S> Protocol<Offer<P, Q>, S>
where
    S: Serializer<bool>,
{
    /// Passive choice. This allows the other end of the session to select one
    /// of two options for continuing the protocol: either `P` or `Q`.
    #[must_use]
    pub fn offer(self) -> Branch<Protocol<P, S>, Protocol<Q, S>> {
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

/// The HasDual trait defines the dual relationship between protocols.
///
/// Any valid protocol has a corresponding dual.
///
/// This trait is sealed and cannot be implemented outside of session-types
pub trait HasDual: private::Sealed {
    type Dual;
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

pub enum Branch<L, R> {
    Left(L),
    Right(R),
}

mod private {
    use super::*;
    pub trait Sealed {}

    // Impl for all exported protocol types
    impl Sealed for End {}
    impl<A, P> Sealed for Send<A, P> {}
    impl<A, P> Sealed for Recv<A, P> {}
    impl<P, Q> Sealed for Choose<P, Q> {}
    impl<P, Q> Sealed for Offer<P, Q> {}
}
