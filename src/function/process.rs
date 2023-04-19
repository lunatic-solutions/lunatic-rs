//! As the name suggests, a "function" process can be spawned just from a
//! function. Opposite of a `AbstractProcess` that requires a `struct`.

use std::any::type_name;
use std::marker::PhantomData;
use std::time::Duration;

use serde::{Deserialize, Serialize};

use crate::host::{self, node_id, process_id};
use crate::mailbox::TIMEOUT;
use crate::protocol::ProtocolCapture;
use crate::serializer::{Bincode, CanSerialize};
use crate::time::TimerRef;
use crate::{MailboxResult, ProcessConfig, Tag};

/// Decides what can be turned into a process.
///
/// It's only implemented for two types: Mailbox & Protocol.
pub trait IntoProcess<M, S> {
    type Process;

    fn spawn<C>(
        capture: C,
        entry: fn(C, Self),
        link: Option<Tag>,
        config: Option<&ProcessConfig>,
        node: Option<u64>,
    ) -> Self::Process
    where
        S: CanSerialize<C> + CanSerialize<ProtocolCapture<C>>;
}

/// A marker trait expressing that a process can be spawned from this type
/// without linking.
///
/// This is used to forbid [`Protocol`](crate::protocol::Protocol) to use the
/// `spawn` functions and only allow usage of `spawn_link` functions.
pub trait NoLink {}

/// Processes are isolated units of compute.
///
/// In lunatic, all code runs inside processes. Processes run concurrently and
/// communicate via message passing.
///
/// Lunatic's processes should not be confused with operating system processes.
/// Processes in lunatic are extremely lightweight in terms of memory and CPU
/// (even compared to threads as used in many other programming languages).
/// Because of this, it is not uncommon to have tens or even hundreds of
/// thousands of processes running simultaneously.
///
/// The `Process` type allows us to spawn new processes from rust functions.
/// There are two kinds of processes:
/// 1. Mailbox based processes
/// 2. Protocol based processes
///
/// They are differentiated by the second argument of the entry function.
///
/// ### Mailbox based processes
///
/// A mailbox process takes a [`Mailbox`](crate::Mailbox) that can only receive
/// messages of one type.
///
/// # Example
///
/// ```
/// let child = Process::spawn(1, |capture, mailbox: Mailbox<i32>| {
///    assert_eq!(capture, 1);
///    assert_eq!(mailbox.receive(), 2);
/// });
///
/// child.send(2);
/// ```
///
/// Processes don't share any memory and messages sent between them need to be
/// serialized. By default, the [`Bincode`] serializer is used, but other
/// serializers that implement the [`CanSerialize`] trait can be used instead.
/// The serializer just needs to be added to the [`Mailbox`](crate::Mailbox)
/// type (e.g. `Mailbox<i32, MessagePack>`).
///
/// Processes can also be linked together using the
/// [`spawn_link`](Self::spawn_link`) function. This means that if one of them
/// fails (panics) the other will be killed too. It is always recommended to
/// spawn linked processes when they depend on each other. That way we can avoid
/// one process forever waiting on a message from another process that doesn't
/// exist anymore.
///
/// ### Protocol based processes
///
/// A protocol process takes a [`Protocol`](crate::protocol::Protocol) that can
/// define a sequence of messages that will be exchanged between two processes.
/// This is also known as a session type. The child will get a reference to the
/// protocol and the parent will get a reference to the opposite protocol.
///
/// # Example
///
/// ```
/// type AddProtocol = Recv<i32, Recv<i32, Send<i32, End>>>;
/// let child = Process::spawn(1, |capture: i32, protocol: Protocol<AddProtocol>| {
///     assert_eq!(capture, 1);
///     let (protocol, a) = protocol.receive();
///     let (protocol, b) = protocol.receive();
///     let _ = protocol.send(capture + a + b);
/// });
///
/// let child = child.send(2);
/// let child = child.send(2);
/// let (_, result) = child.receive();
/// assert_eq!(result, 5);
/// ```
///
/// The rust type system guarantees that the all messages are sent in the
/// correct order and are of correct type. Code that doesn't follow the protocol
/// would not compile.
///
/// Same as the mailbox, the protocol based process can choose another
/// serializer (e.g. `Protocol<AddProtocol, MessagePack>`).
///
/// If a protocol based process is dropped before the `End` state is reached,
/// the drop will panic.
#[derive(Serialize, Deserialize)]
pub struct Process<M, S = Bincode> {
    node_id: u64,
    id: u64,
    #[serde(skip_serializing, default)]
    serializer_type: PhantomData<(M, S)>,
}

impl<M, S> Process<M, S> {
    /// Creates a new process reference from a node_id and process_id.
    pub unsafe fn new(node_id: u64, process_id: u64) -> Self {
        Self {
            node_id,
            id: process_id,
            serializer_type: PhantomData,
        }
    }

    /// Return reference to self.
    pub unsafe fn this() -> Self {
        Self::new(node_id(), process_id())
    }

    /// Returns `true` for processes on the local node that are running.
    ///
    /// Panics if called on a remote process.
    pub fn is_alive(&self) -> bool {
        assert_eq!(
            self.node_id(),
            host::node_id(),
            "is_alive() can only be used with local processes"
        );
        unsafe { host::api::process::exists(self.id()) != 0 }
    }

    /// Spawn a process.
    pub fn spawn<C, T>(capture: C, entry: fn(C, T)) -> T::Process
    where
        S: CanSerialize<C> + CanSerialize<ProtocolCapture<C>>,
        T: IntoProcess<M, S>,
        T: NoLink,
    {
        T::spawn(capture, entry, None, None, None)
    }

    /// Spawn a process on a remote node.
    pub fn spawn_node<C, T>(node_id: u64, capture: C, entry: fn(C, T)) -> T::Process
    where
        S: CanSerialize<C> + CanSerialize<ProtocolCapture<C>>,
        T: IntoProcess<M, S>,
        T: NoLink,
    {
        T::spawn(capture, entry, None, None, Some(node_id))
    }

    /// Spawn a process on a remote node.
    pub fn spawn_node_config<C, T>(
        node_id: u64,
        config: &ProcessConfig,
        capture: C,
        entry: fn(C, T),
    ) -> T::Process
    where
        S: CanSerialize<C> + CanSerialize<ProtocolCapture<C>>,
        T: IntoProcess<M, S>,
        T: NoLink,
    {
        T::spawn(capture, entry, None, Some(config), Some(node_id))
    }

    /// Spawn a linked process.
    pub fn spawn_link<C, T>(capture: C, entry: fn(C, T)) -> T::Process
    where
        S: CanSerialize<C> + CanSerialize<ProtocolCapture<C>>,
        T: IntoProcess<M, S>,
    {
        T::spawn(capture, entry, Some(Tag::new()), None, None)
    }

    /// Spawn a linked process with a tag.
    ///
    /// Allows the caller to provide a tag for the link.
    pub fn spawn_link_tag<C, T>(capture: C, tag: Tag, entry: fn(C, T)) -> T::Process
    where
        S: CanSerialize<C> + CanSerialize<ProtocolCapture<C>>,
        T: IntoProcess<M, S>,
    {
        T::spawn(capture, entry, Some(tag), None, None)
    }

    /// Spawn a process with a custom configuration.
    pub fn spawn_config<C, T>(config: &ProcessConfig, capture: C, entry: fn(C, T)) -> T::Process
    where
        S: CanSerialize<C> + CanSerialize<ProtocolCapture<C>>,
        T: IntoProcess<M, S>,
        T: NoLink,
    {
        T::spawn(capture, entry, None, Some(config), None)
    }

    /// Spawn a linked process with a custom configuration.
    pub fn spawn_link_config<C, T>(
        config: &ProcessConfig,
        capture: C,
        entry: fn(C, T),
    ) -> T::Process
    where
        S: CanSerialize<C> + CanSerialize<ProtocolCapture<C>>,
        T: IntoProcess<M, S>,
    {
        T::spawn(capture, entry, Some(Tag::new()), Some(config), None)
    }

    /// Spawn a linked process with a custom configuration & provide tag for
    /// linking.
    pub fn spawn_link_config_tag<C, T>(
        config: &ProcessConfig,
        capture: C,
        tag: Tag,
        entry: fn(C, T),
    ) -> T::Process
    where
        S: CanSerialize<C> + CanSerialize<ProtocolCapture<C>>,
        T: IntoProcess<M, S>,
    {
        T::spawn(capture, entry, Some(tag), Some(config), None)
    }

    /// Returns a local node process ID.
    pub fn id(&self) -> u64 {
        self.id
    }

    /// Returns a node ID.
    pub fn node_id(&self) -> u64 {
        self.node_id
    }

    /// Link process to the one currently running.
    pub fn link(&self) {
        // Don't use tags because a process' [`Mailbox`] can't differentiate between
        // regular messages and signals. Both processes should almost always die
        // when a link is broken.
        unsafe { host::api::process::link(0, self.id) };
    }

    /// Unlink processes from the caller.
    pub fn unlink(&self) {
        unsafe { host::api::process::unlink(self.id) };
    }

    /// Kill this process
    pub fn kill(&self) {
        unsafe { host::api::process::kill(self.id) };
    }

    /// Register process under a name.
    pub fn register(&self, name: &str) {
        // Encode type information in name
        let name = process_name::<M, S>(ProcessType::Process, name);
        unsafe { host::api::registry::put(name.as_ptr(), name.len(), self.node_id, self.id) };
    }

    /// Look up a process.
    pub fn lookup(name: &str) -> Option<Self> {
        let name = process_name::<M, S>(ProcessType::Process, name);
        let mut id = 0;
        let mut node_id = 0;
        let result =
            unsafe { host::api::registry::get(name.as_ptr(), name.len(), &mut node_id, &mut id) };
        if result == 0 {
            Some(Self {
                node_id,
                id,
                serializer_type: PhantomData,
            })
        } else {
            None
        }
    }
}

impl<M, S> Process<M, S>
where
    S: CanSerialize<M>,
{
    /// Send a message to the process.
    ///
    /// # Panics
    ///
    /// This function will panic if the received message can't be serialized
    /// into `M` with serializer `S`.
    pub fn send(&self, message: M) {
        // Create new message.
        unsafe { host::api::message::create_data(Tag::none().id(), 0) };
        // During serialization resources will add themselves to the message.
        S::encode(&message).unwrap();
        // Send it!
        host::send(self.node_id, self.id);
    }

    /// Send a message to the process after the specified duration has passed.
    ///
    /// # Panics
    ///
    /// This function will panic if the received message can't be serialized
    /// into `M` with serializer `S`.
    pub fn send_after(&self, message: M, duration: Duration) -> TimerRef {
        // Create new message.
        unsafe { host::api::message::create_data(Tag::none().id(), 0) };
        // During serialization resources will add themselves to the message.
        S::encode(&message).unwrap();
        // Send it!
        let timer_id =
            unsafe { host::api::timer::send_after(self.id, duration.as_millis() as u64) };
        TimerRef::new(timer_id)
    }

    /// Send message to process with a specific tag.
    ///
    /// # Panics
    ///
    /// This function will panic if the received message can't be serialized
    /// into `M` with serializer `S`.
    pub fn tag_send(&self, tag: Tag, message: M) {
        // Create new message.
        unsafe { host::api::message::create_data(tag.id(), 0) };
        // During serialization resources will add themselves to the message.
        S::encode(&message).unwrap();
        // Send it!
        host::send(self.node_id, self.id);
    }

    /// Send a message to the process with a specific tag, after the specified
    /// duration has passed.
    ///
    /// # Panics
    ///
    /// This function will panic if the received message can't be serialized
    /// into `M` with serializer `S`.
    pub fn tag_send_after(&self, tag: Tag, message: M, duration: Duration) -> TimerRef {
        // Create new message.
        unsafe { host::api::message::create_data(tag.id(), 0) };
        // During serialization resources will add themselves to the message.
        S::encode(&message).unwrap();
        // Send it!
        let timer_id =
            unsafe { host::api::timer::send_after(self.id, duration.as_millis() as u64) };
        TimerRef::new(timer_id)
    }

    /// Sends message and waits on response until timeout (if specified).
    ///
    /// # Safety
    ///
    /// The other side needs to be aware that the response needs to be sent back
    /// with the `receive_tag`. There is no way to enforce this with the type
    /// system at this level.
    ///
    /// # Panics
    ///
    /// This function will panic if the received message can't be serialized
    /// into `M` with serializer `S` or the `Response` can't be deserialized.
    #[track_caller]
    pub(crate) unsafe fn tag_send_receive<Response>(
        &self,
        send_tag: Tag,
        receive_tag: Tag,
        message: M,
        timeout: Option<Duration>,
    ) -> MailboxResult<Response>
    where
        S: CanSerialize<M>,
        S: CanSerialize<Response>,
    {
        unsafe { host::api::message::create_data(send_tag.id(), 0) };

        S::encode(&message).unwrap();
        let timeout_ms = match timeout {
            Some(timeout) => timeout.as_millis() as u64,
            None => u64::MAX,
        };

        let result =
            host::send_receive_skip_search(self.node_id, self.id, receive_tag.id(), timeout_ms);
        if result == TIMEOUT {
            MailboxResult::TimedOut
        } else {
            match S::decode() {
                Ok(msg) => MailboxResult::Message(msg),
                Err(_) => panic!("Could not deserialize message: {}", type_name::<Response>()),
            }
        }
    }
}

/// Processes are equal if their process id and node id are equal.
impl<M, S> PartialEq for Process<M, S> {
    fn eq(&self, other: &Self) -> bool {
        self.id() == other.id() && self.node_id() == other.node_id()
    }
}

/// Process equality comparison is an equivalence relation
impl<M, S> Eq for Process<M, S> {}

// Implement Hash explicitly to match the behavior of PartialEq
impl<M, S> std::hash::Hash for Process<M, S> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.node_id.hash(state);
        self.id.hash(state);
    }
}

impl<M, S> std::fmt::Debug for Process<M, S> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Process")
            .field("id", &self.id())
            .field("node_id", &self.node_id())
            .finish()
    }
}

impl<M, S> Clone for Process<M, S> {
    fn clone(&self) -> Self {
        Self {
            node_id: self.node_id,
            id: self.id,
            serializer_type: self.serializer_type,
        }
    }
}

impl<M, S> Copy for Process<M, S> {}

#[derive(Clone, Copy, Debug)]
pub(crate) enum ProcessType {
    Process,
    ProcessRef,
}

impl std::fmt::Display for ProcessType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProcessType::Process => write!(f, "Process"),
            ProcessType::ProcessRef => write!(f, "ProcessRef"),
        }
    }
}

pub(crate) fn process_name<M, S>(pt: ProcessType, name: &str) -> String {
    format!(
        "{}/{}/{pt}/{name}",
        std::any::type_name::<M>(),
        std::any::type_name::<S>(),
    )
}
