//! As the name suggests, a "function" process can be spawned just from a function. Opposite of a
//! `AbstractProcess` that requires a `struct`.

use serde::{Deserialize, Serialize};
use std::{marker::PhantomData, time::Duration};

use crate::{
    host::{self, node_id, process_id},
    protocol::ProtocolCapture,
    serializer::{Bincode, Serializer},
    timer::TimerRef,
    ProcessConfig, Tag,
};

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
        S: Serializer<C> + Serializer<ProtocolCapture<C>>;
}

/// A marker trait expressing that a process can be spawned from this type without linking.
///
/// This is used to forbid [`Protocol`](crate::protocol::Protocol) to use the `spawn` functions
/// and only allow usage of `spawn_link` functions.
pub trait NoLink {}

/// Processes are isolated units of compute.
///
/// In lunatic, all code runs inside processes. Processes run concurrently and communicate via
/// message passing.
///
/// Lunatic's processes should not be confused with operating system processes. Processes in
/// lunatic are extremely lightweight in terms of memory and CPU (even compared to threads as used
/// in many other programming languages). Because of this, it is not uncommon to have tens or even
/// hundreds of thousands of processes running simultaneously.
///
/// The `Process` type allows us to spawn new processes from rust functions. There are two kinds
/// of processes:
/// 1. Mailbox based processes
/// 2. Protocol based processes
///
/// They are differentiated by the second argument of the entry function.
///
/// ### Mailbox based processes
///
/// A mailbox process takes a [`Mailbox`](crate::Mailbox) that can only receive messages of one
/// type.
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
/// Processes don't share any memory and messages sent between them need to be serialized. By
/// default, the [`Bincode`] serializer is used, but other serializers that implement the
/// [`Serializer`] trait can be used instead. The serializer just needs to be added to the
/// [`Mailbox`](crate::Mailbox) type (e.g. `Mailbox<i32, MessagePack>`).
///
/// Processes can also be linked together using the [`spawn_link`](Self::spawn_link`) function.
/// This means that if one of them fails (panics) the other will be killed too. It is always
/// recommended to spawn linked processes when they depend on each other. That way we can avoid
/// one process forever waiting on a message from another process that doesn't exist anymore.
///
/// ### Protocol based processes
///
/// A protocol process takes a [`Protocol`](crate::protocol::Protocol) that can define a sequence
/// of messages that will be exchanged between two processes. This is also known as a session type.
/// The child will get a reference to the protocol and the parent will get a reference to the
/// opposite protocol.
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
/// The rust type system guarantees that the all messages are sent in the correct order and are of
/// correct type. Code that doesn't follow the protocol would not compile.
///
/// Same as the mailbox, the protocol based process can choose another serializer (e.g.
/// `Protocol<AddProtocol, MessagePack>`).
///
/// If a protocol based process is dropped before the `End` state is reached, the drop will panic.
#[derive(Serialize, Deserialize)]
pub struct Process<M, S = Bincode> {
    node_id: u64,
    id: u64,
    #[serde(skip_serializing, default)]
    serializer_type: PhantomData<(M, S)>,
}

impl<M, S> Process<M, S> {
    pub fn new(node_id: u64, process_id: u64) -> Self {
        Self {
            node_id,
            id: process_id,
            serializer_type: PhantomData,
        }
    }

    pub fn this() -> Self {
        Self::new(node_id(), process_id())
    }
    /// Spawn a process.
    pub fn spawn<C, T>(capture: C, entry: fn(C, T)) -> T::Process
    where
        S: Serializer<C> + Serializer<ProtocolCapture<C>>,
        T: IntoProcess<M, S>,
        T: NoLink,
    {
        T::spawn(capture, entry, None, None, None)
    }

    /// Spawn a process on a remote node.
    pub fn spawn_node<C, T>(node_id: u64, capture: C, entry: fn(C, T)) -> T::Process
    where
        S: Serializer<C> + Serializer<ProtocolCapture<C>>,
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
        S: Serializer<C> + Serializer<ProtocolCapture<C>>,
        T: IntoProcess<M, S>,
        T: NoLink,
    {
        T::spawn(capture, entry, None, Some(config), Some(node_id))
    }

    /// Spawn a linked process.
    pub fn spawn_link<C, T>(capture: C, entry: fn(C, T)) -> T::Process
    where
        S: Serializer<C> + Serializer<ProtocolCapture<C>>,
        T: IntoProcess<M, S>,
    {
        T::spawn(capture, entry, Some(Tag::new()), None, None)
    }

    /// Spawn a linked process with a tag.
    ///
    /// Allows the caller to provide a tag for the link.
    pub fn spawn_link_tag<C, T>(capture: C, tag: Tag, entry: fn(C, T)) -> T::Process
    where
        S: Serializer<C> + Serializer<ProtocolCapture<C>>,
        T: IntoProcess<M, S>,
    {
        T::spawn(capture, entry, Some(tag), None, None)
    }

    /// Spawn a process with a custom configuration.
    pub fn spawn_config<C, T>(config: &ProcessConfig, capture: C, entry: fn(C, T)) -> T::Process
    where
        S: Serializer<C> + Serializer<ProtocolCapture<C>>,
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
        S: Serializer<C> + Serializer<ProtocolCapture<C>>,
        T: IntoProcess<M, S>,
    {
        T::spawn(capture, entry, Some(Tag::new()), Some(config), None)
    }

    /// Spawn a linked process with a custom configuration & provide tag for linking.
    pub fn spawn_link_config_tag<C, T>(
        config: &ProcessConfig,
        capture: C,
        tag: Tag,
        entry: fn(C, T),
    ) -> T::Process
    where
        S: Serializer<C> + Serializer<ProtocolCapture<C>>,
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
        // Don't use tags because a process' [`Mailbox`] can't differentiate between regular
        // messages and signals. Both processes should almost always die when a link is broken.
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
        let name = format!(
            "{} + Process + {}/{}",
            name,
            std::any::type_name::<M>(),
            std::any::type_name::<S>()
        );
        unsafe { host::api::registry::put(name.as_ptr(), name.len(), self.node_id, self.id) };
    }

    /// Look up a process.
    pub fn lookup(name: &str) -> Option<Self> {
        let name = format!(
            "{} + Process + {}/{}",
            name,
            std::any::type_name::<M>(),
            std::any::type_name::<S>()
        );
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
    S: Serializer<M>,
{
    /// Send a message to the process.
    ///
    /// # Panics
    ///
    /// This function will panic if the received message can't be serialized into `M`
    /// with serializer `S`.
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
    /// This function will panic if the received message can't be serialized into `M`
    /// with serializer `S`.
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
    /// This function will panic if the received message can't be serialized into `M`
    /// with serializer `S`.
    pub fn tag_send(&self, tag: Tag, message: M) {
        // Create new message.
        unsafe { host::api::message::create_data(tag.id(), 0) };
        // During serialization resources will add themselves to the message.
        S::encode(&message).unwrap();
        // Send it!
        host::send(self.node_id, self.id);
    }
}

/// Processes are equal if their process id and node id are equal.
impl<M, S> PartialEq for Process<M, S> {
    fn eq(&self, other: &Self) -> bool {
        self.id() == other.id() && self.node_id() == other.node_id()
    }
}

/// Proccess equality comparison is an equivalance relation
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
