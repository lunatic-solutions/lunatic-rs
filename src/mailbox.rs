use std::fmt;
use std::marker::PhantomData;
use std::time::Duration;

use crate::function::process::{IntoProcess, NoLink};
use crate::host::api::message;
use crate::serializer::{Bincode, CanSerialize, DecodeError};
use crate::{host, Process, ProcessConfig, Tag};

pub const LINK_DIED: u32 = 1;
pub const TIMEOUT: u32 = 9027;

/// Marker type indicating that the [`Mailbox`] **IS** catching deaths of linked
/// processes.
pub struct Catching;

/// The mailbox of a [`Process`].
///
/// Each process in lunatic gets one dedicated mailbox. Messages sent to the
/// process will end up in this mailbox. Each [`Process`] and [`Mailbox`] pair
/// have matching message and serializer types, because of this de/serialization
/// should never fail.
///
/// One case where deserialization might fail is when the `Mailbox` type is used
/// on a function inside an external WebAssembly module that is loaded by
/// [`WasmModule`](crate::WasmModule). In this case we don't have any
/// compile-time information about what messages are going to be received by
/// this mailbox. For such cases the function
/// [`try_receive`](./struct.Mailbox.html#method.try_receive) can be used. It
/// will not panic in case it can't deserialize the message buffer.
///
/// ## Message ordering
///
/// Lunatic guarantees that messages sent between two processes will arrive in
/// the same order they were sent. Ordering is not guaranteed if more than two
/// processes are involved.
///
/// ## Link deaths
///
/// By default, if a linked process fails all the links will die too. This
/// behavior can be changed by using the [`catch_link_failure`]() function. The
/// returned [`Mailbox<_, _, Catching>`] will receive a special
/// [`MailboxResult::LinkDied`] in its mailbox containing the [`Tag`] used when
/// the process was spawned ([`spawn_link_tag`](Process::spawn_link_tag)).
pub struct Mailbox<M, S = Bincode, L = ()>
where
    S: CanSerialize<M>,
{
    phantom: PhantomData<(M, S, L)>,
}

impl<M, S> Mailbox<M, S, ()>
where
    S: CanSerialize<M>,
{
    /// Gets next message from process' mailbox.
    ///
    /// If the mailbox is empty, this function will block until a new message
    /// arrives.
    ///
    /// # Panics
    ///
    /// This function will panic if the received message can't be deserialized
    /// into `M` with serializer `S`.
    #[track_caller]
    pub fn receive(&self) -> M {
        self.receive_(&[], None).unwrap()
    }

    /// Gets next message from process' mailbox that is tagged with one of the
    /// `tags`.
    ///
    /// If no such message exists, this function will block until a new message
    /// arrives. If `tags` is an empty array, it will behave the same as
    /// `receive`.
    ///
    /// # Panics
    ///
    /// This function will panic if the received message can't be deserialized
    /// into `M` with serializer `S`.
    #[track_caller]
    pub fn tag_receive(&self, tags: &[Tag]) -> M {
        self.receive_(tags, None).unwrap()
    }

    /// Allow this mailbox to catch link failures.
    ///
    /// This function returns a [`Mailbox`] that will get
    /// [`MailboxResult::LinkDied`]  messages every time a linked process dies.
    pub fn catch_link_failure(self) -> Mailbox<M, S, Catching> {
        unsafe {
            host::api::process::die_when_link_dies(0);
            Mailbox::<M, S, Catching>::new()
        }
    }
}

/// A mailbox that is catching dead links.
impl<M, S> Mailbox<M, S, Catching>
where
    S: CanSerialize<M>,
{
    /// Gets next message from process' mailbox.
    ///
    /// If the mailbox is empty, this function will block until a new message
    /// arrives.
    ///
    /// A message indicating that a linked process died is returned as
    /// [`MailboxResult::LinkDied`] with the [`Tag`] used to spawn the linked
    /// process.
    pub fn receive(&self) -> MailboxResult<M> {
        self.receive_(&[], None)
    }

    /// Gets next message from process' mailbox that is tagged with one of the
    /// `tags`.
    ///
    /// If no such message exists, this function will block until a new message
    /// arrives. If `tags` is an empty array, it will behave the same as
    /// `receive`.
    ///
    /// This function can also be used to await the death of specific linked
    /// processes. In this case the `tags` array should contain tags
    /// corresponding to the processes we are awaiting to die.
    pub fn tag_receive(&self, tags: &[Tag]) -> MailboxResult<M> {
        self.receive_(tags, None)
    }
}

impl<M, S, L> Mailbox<M, S, L>
where
    S: CanSerialize<M>,
{
    /// Returns a reference to the currently running process
    pub fn this(&self) -> Process<M, S> {
        unsafe { Process::new(host::node_id(), host::process_id()) }
    }

    /// Same as `receive`, but doesn't panic in case the deserialization fails.
    /// Instead, it will return [`MailboxResult::DeserializationFailed`].
    pub fn try_receive(&self, timeout: Duration) -> MailboxResult<M> {
        self.receive_(&[], Some(timeout))
    }

    /// Same as `receive`, but only waits for the duration of timeout for the
    /// message. If the timeout expires it will return
    /// [`MailboxResult::TimedOut`].
    pub fn receive_timeout(&self, timeout: Duration) -> MailboxResult<M> {
        self.receive_(&[], Some(timeout))
    }

    /// Same as `tag_receive`, but only waits for the duration of timeout for
    /// the message. If the timeout expires it will return
    /// [`MailboxResult::TimedOut`].
    pub fn tag_receive_timeout(&self, tags: &[Tag], timeout: Duration) -> MailboxResult<M> {
        self.receive_(tags, Some(timeout))
    }

    fn receive_(&self, tags: &[Tag], timeout: Option<Duration>) -> MailboxResult<M> {
        let tags: Vec<i64> = tags.iter().map(|tag| tag.id()).collect();
        let timeout_ms = match timeout {
            Some(timeout) => timeout.as_millis() as u64,
            None => u64::MAX,
        };
        let message_type = unsafe { message::receive(tags.as_ptr(), tags.len(), timeout_ms) };
        match message_type {
            LINK_DIED => MailboxResult::LinkDied(unsafe { Tag::from(message::get_tag()) }),
            TIMEOUT => MailboxResult::TimedOut,
            _ => match S::decode() {
                Ok(msg) => MailboxResult::Message(msg),
                Err(err) => MailboxResult::DeserializationFailed(err),
            },
        }
    }

    /// Create a mailbox with a specific type.
    ///
    /// ### Safety
    ///
    /// It's not safe to mix different types of mailboxes inside one process.
    /// This function should never be used directly. The only reason it's public
    /// is that it's used inside the `main` macro and needs to be available
    /// outside this crate.
    pub unsafe fn new() -> Self {
        Self {
            phantom: PhantomData {},
        }
    }
}

impl<M, S, L> Clone for Mailbox<M, S, L>
where
    S: CanSerialize<M>,
{
    fn clone(&self) -> Self {
        Self {
            phantom: self.phantom,
        }
    }
}

impl<M, S, L> Copy for Mailbox<M, S, L> where S: CanSerialize<M> {}

impl<M, S, L> fmt::Debug for Mailbox<M, S, L>
where
    S: CanSerialize<M>,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Mailbox")
            .field("message", &std::any::type_name::<M>())
            .field("serializer", &std::any::type_name::<S>())
            .field("link", &std::any::type_name::<L>())
            .finish()
    }
}

/// Result of a `recieve*` call on a [`Mailbox`].
#[derive(Debug)]
pub enum MailboxResult<T> {
    Message(T),
    DeserializationFailed(DecodeError),
    TimedOut,
    LinkDied(Tag),
}

impl<T> MailboxResult<T> {
    #[track_caller]
    pub fn unwrap(self) -> T {
        match self {
            MailboxResult::Message(msg) => msg,
            MailboxResult::DeserializationFailed(err) => panic!("{:?}", err),
            MailboxResult::TimedOut => panic!("TimedOut"),
            MailboxResult::LinkDied(_) => panic!("LinkDied"),
        }
    }

    // Returns true if it's a regular message.
    pub fn is_message(&self) -> bool {
        matches!(self, MailboxResult::Message(_))
    }

    // Returns true if it's a link died signal turned into a message.
    pub fn is_link_died(&self) -> bool {
        matches!(self, MailboxResult::LinkDied(_))
    }

    // Returns true if it's a timeout.
    pub fn is_timed_out(&self) -> bool {
        matches!(self, MailboxResult::TimedOut)
    }
}

impl<M, S> NoLink for Mailbox<M, S> where S: CanSerialize<M> {}

impl<M, S> IntoProcess<M, S> for Mailbox<M, S>
where
    S: CanSerialize<M>,
{
    type Process = Process<M, S>;

    fn spawn<C>(
        capture: C,
        entry: fn(C, Self),
        link: Option<Tag>,
        config: Option<&ProcessConfig>,
        node: Option<u64>,
    ) -> Self::Process
    where
        S: CanSerialize<C> + CanSerialize<M>,
    {
        let entry = entry as usize as i32;
        let node_id = node.unwrap_or_else(host::node_id);

        // The `type_helper_wrapper` function is used here to create a pointer to a
        // function with generic types C, M & S. We can only send pointer data across
        // processes and this is the only way the Rust compiler will let us transfer
        // this information into the new process.
        match host::spawn(node, config, link, type_helper_wrapper::<C, M, S>, entry) {
            Ok(id) => {
                // If the captured variable is of size 0, we don't need to send it to another
                // process.
                if std::mem::size_of::<C>() == 0 {
                    unsafe { Process::new(node_id, id) }
                } else {
                    let child = unsafe { Process::<C, S>::new(node_id, id) };
                    child.send(capture);
                    // Processes can only receive one type of message, but to pass in the captured
                    // variable we pretend for the first message that our process is receiving
                    // messages of type `C`.
                    unsafe { std::mem::transmute(child) }
                }
            }
            Err(err) => panic!("Failed to spawn a process: {}", err),
        }
    }
}

/// Wrapper function to help transfer the generic types C, M & S into the new
/// process.
fn type_helper_wrapper<C, M, S>(function: i32)
where
    S: CanSerialize<C> + CanSerialize<M>,
{
    // If the captured variable is of size 0, don't wait on it.
    let captured = if std::mem::size_of::<C>() == 0 {
        unsafe { std::mem::MaybeUninit::<C>::zeroed().assume_init() }
    } else {
        unsafe { Mailbox::<C, S>::new() }.receive()
    };
    let mailbox = unsafe { Mailbox::new() };
    let function: fn(C, Mailbox<M, S>) = unsafe { std::mem::transmute(function as usize) };
    function(captured, mailbox);
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use lunatic_test::test;

    use super::*;
    use crate::{sleep, Mailbox};

    #[test]
    fn mailbox() {
        let child = Process::spawn(1, |capture, mailbox: Mailbox<i32>| {
            assert_eq!(capture, 1);
            assert_eq!(mailbox.receive(), 2);
        });

        child.send(2);
        sleep(Duration::from_millis(100));
    }

    #[test]
    #[should_panic]
    fn mailbox_link() {
        Process::spawn_link((), |_, _: Mailbox<()>| {
            panic!("fails");
        });

        // This process should fail before 100ms, because the link panics.
        sleep(Duration::from_millis(100));
    }
}
