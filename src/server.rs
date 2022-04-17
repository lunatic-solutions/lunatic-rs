use std::{cell::UnsafeCell, marker::PhantomData};

use crate::{
    host,
    mailbox::{LinkMailbox, LinkTrapped},
    serializer::{Bincode, Serializer},
    Mailbox, Process, ProcessConfig, Resource, Tag,
};

/// A trait for implementing the server of a client-server relation.
///
/// Types that implement the `Server` trait become the state of the server process. This state can
/// be mutated through messages and requests. To define a handler for them, [`HandleMessage`] or
/// [`ServerRequest`] can be used.
///
/// [`HandleMessage`] provides a `send` method to send messages to the server, without
/// waiting on a response. [`ServerRequest`] provides a `request` method that will block
/// until a response is received.
pub trait Server
where
    Self: Sized,
{
    /// The argument received by the `init` function.
    ///
    /// This argument is sent from the parent to the child and needs to be serializable.
    type Arg: serde::Serialize + serde::de::DeserializeOwned;

    /// The state of the server.
    ///
    /// In most cases this value is set to `Self`.
    type State;

    /// Entry function of the new process.
    ///
    /// This function is executed inside the new process. It will receive the arguments passed
    /// to the `start` or `start_link` function by the parent. And will return the starting state
    /// of the newly spawned server.
    ///
    /// The parent will block on the call of `start` or `start_link` until this function finishes.
    /// This allows server startup to be synchronized.
    fn init(arg: Self::Arg) -> Self::State;

    /// This function will be called if the server is set to catch link deaths with
    /// `host::api::process::die_when_link_dies(1)` and a linked process traps.
    fn handle_link_trapped(_state: &mut Self::State, _tag: Tag) {}
}

/// Defines a handler for a server message of type `M`.
pub trait ServerMessage<M, S = Bincode>
where
    S: Serializer<M>,
{
    fn handle(&mut self, message: M);
}

/// Defines a handler for a server request of type `M`.
pub trait ServerRequest<M, S = Bincode>
where
    S: Serializer<M>,
{
    type Response;

    fn handle(&mut self, request: M) -> Self::Response;
}

pub trait StartServer<T>
where
    T: Server,
{
    fn start(arg: T::Arg, name: Option<&str>) -> ServerRef<T>;
    fn start_config(arg: T::Arg, name: Option<&str>, config: &ProcessConfig) -> ServerRef<T>;
    fn start_link(arg: T::Arg, name: Option<&str>) -> ServerRef<T>;
    fn start_link_config(arg: T::Arg, name: Option<&str>, config: &ProcessConfig) -> ServerRef<T>;
}

impl<T> StartServer<T> for T
where
    T: Server,
{
    /// Start a server process.
    fn start(arg: T::Arg, name: Option<&str>) -> ServerRef<T> {
        start::<T>(arg, name, false, None).unwrap()
    }

    /// Start a server process with configuration.
    fn start_config(arg: T::Arg, name: Option<&str>, config: &ProcessConfig) -> ServerRef<T> {
        start::<T>(arg, name, false, Some(config)).unwrap()
    }

    /// Start a linked server process.
    fn start_link(arg: T::Arg, name: Option<&str>) -> ServerRef<T> {
        start::<T>(arg, name, true, None).unwrap()
    }

    /// Start a linked server process with configuration.
    fn start_link_config(arg: T::Arg, name: Option<&str>, config: &ProcessConfig) -> ServerRef<T> {
        start::<T>(arg, name, true, Some(config)).unwrap()
    }
}

/// An internal interface that catches failures inside the `init` function of a `Server`.
///
/// Only "link" functions are provided, because a panic can't be propagated to the parent without a
/// link. Currently, only the `Supervisor` uses this functionality to check for failures inside of
/// children.
pub(crate) trait StartFailableServer<T>
where
    T: Server,
{
    fn start_link_or_fail(arg: T::Arg, name: Option<&str>) -> Result<ServerRef<T>, LinkTrapped>;
    fn start_link_config_or_fail(
        arg: T::Arg,
        name: Option<&str>,
        config: &ProcessConfig,
    ) -> Result<ServerRef<T>, LinkTrapped>;
}

impl<T> StartFailableServer<T> for T
where
    T: Server,
{
    /// Start a linked server process.
    fn start_link_or_fail(arg: T::Arg, name: Option<&str>) -> Result<ServerRef<T>, LinkTrapped> {
        start::<T>(arg, name, true, None)
    }

    /// Start a linked server process with configuration.
    fn start_link_config_or_fail(
        arg: T::Arg,
        name: Option<&str>,
        config: &ProcessConfig,
    ) -> Result<ServerRef<T>, LinkTrapped> {
        start::<T>(arg, name, true, Some(config))
    }
}

fn start<T>(
    arg: T::Arg,
    name: Option<&str>,
    link: bool,
    config: Option<&ProcessConfig>,
) -> Result<ServerRef<T>, LinkTrapped>
where
    T: Server,
{
    let tag = Tag::new();
    let parent = unsafe { <Process<(), Bincode> as Resource>::from_id(host::api::process::this()) };
    let server = if let Some(config) = config {
        if link {
            Process::<(), Bincode>::spawn_link_config_tag(
                config,
                (parent, tag, arg, T::init as usize as i32),
                tag,
                starter::<T>,
            )
        } else {
            Process::<(), Bincode>::spawn_config(
                config,
                (parent, tag, arg, T::init as usize as i32),
                starter::<T>,
            )
        }
    } else {
        if link {
            Process::<(), Bincode>::spawn_link_tag(
                (parent, tag, arg, T::init as usize as i32),
                tag,
                starter::<T>,
            )
        } else {
            Process::<(), Bincode>::spawn((parent, tag, arg, T::init as usize as i32), starter::<T>)
        }
    };

    // Don't return until `init()` finishes
    let mailbox: LinkMailbox<(), Bincode> = unsafe { LinkMailbox::new() };
    let _ = mailbox.tag_receive(Some(&[tag]))?;

    // If a name is given, register the process under this name
    if let Some(_name) = name {
        // TODO: We need to add back the register host function
    }

    Ok(ServerRef {
        server,
        consumed: UnsafeCell::new(false),
        phantom: PhantomData,
    })
}

// Entry point of the server process.
fn starter<T>(
    (parent, tag, capture, entry): (Process<(), Bincode>, Tag, T::Arg, i32),
    _: Mailbox<(), Bincode>,
) where
    T: Server,
{
    let entry: fn(arg: T::Arg) -> T::State = unsafe { std::mem::transmute(entry) };
    let mut state = entry(capture);
    // Let parent know that the `init()` call finished
    parent.tag_send(tag, ());

    let mailbox: LinkMailbox<Sendable, Bincode> = unsafe { LinkMailbox::new() };
    // Run server forever and respond to requests.
    loop {
        let dispatcher = mailbox.tag_receive(None);
        match dispatcher {
            Ok(dispatcher) => match dispatcher {
                Sendable::Message(handler) => {
                    let handler: fn(state: &mut T::State) = unsafe { std::mem::transmute(handler) };
                    handler(&mut state);
                }
                Sendable::Request(handler, sender) => {
                    let handler: fn(state: &mut T::State, sender: Process<()>) =
                        unsafe { std::mem::transmute(handler) };
                    handler(&mut state, sender);
                }
            },
            Err(link_trapped) => T::handle_link_trapped(&mut state, link_trapped.tag()),
        }
    }
}

pub trait Message<M, S>
where
    S: Serializer<M>,
{
    fn send(&self, message: M);
}

pub trait Request<M, S>
where
    S: Serializer<M>,
{
    type Result;

    fn request(&self, request: M) -> Self::Result;
}

/// A reference to a running server.
pub struct ServerRef<T> {
    server: Process<()>,
    // If set to true, the host call `lunatic::process::drop_process` will not be executed on drop.
    consumed: UnsafeCell<bool>,
    phantom: PhantomData<T>,
}

impl<T> ServerRef<T> {
    /// Construct a process from a raw ID.
    unsafe fn from(id: u64) -> Self {
        let server = <Process<()> as Resource>::from_id(id);
        ServerRef {
            server,
            consumed: UnsafeCell::new(false),
            phantom: PhantomData,
        }
    }

    /// Returns a globally unique process ID.
    pub fn uuid(&self) -> u128 {
        let mut uuid: [u8; 16] = [0; 16];
        unsafe { host::api::process::id(self.server.id(), &mut uuid as *mut [u8; 16]) };
        u128::from_le_bytes(uuid)
    }

    /// Marks the process as consumed.
    ///
    /// Consumed processes don't call the `lunatic::process::drop_process` host function when they
    /// are dropped. This characteristic is useful when implementing serializers for processes.
    /// Serializers will move the process out of the local state into the message scratch buffer,
    /// and they can't be dropped from the local state anymore.
    unsafe fn consume(&self) {
        *self.consumed.get() = true;
    }
}

// This is a wrapper around the message/request that is sent to the server.
//
// The first `i32` value is a pointer
#[derive(serde::Serialize, serde::Deserialize)]
enum Sendable {
    Message(i32),
    // The process type can't be carried over as a generic and is set here to `()`, but overwritten
    // at the time of returning with the correct type.
    Request(i32, Process<()>),
}

impl<M, S, T> Message<M, S> for ServerRef<T>
where
    T: Server,
    T::State: ServerMessage<M, S>,
    S: Serializer<M>,
{
    /// Send message to the server.
    fn send(&self, message: M) {
        fn unpacker<TU, MU, SU>(this: &mut TU)
        where
            TU: ServerMessage<MU, SU>,
            SU: Serializer<MU>,
        {
            let message: MU = SU::decode().unwrap();
            <TU as ServerMessage<MU, SU>>::handle(this, message);
        }

        // Create new message buffer.
        unsafe { host::api::message::create_data(1, 0) };
        // First encode the handler inside the message buffer.
        let handler = unpacker::<T::State, M, S> as usize as i32;
        let handler_message = Sendable::Message(handler);
        Bincode::encode(&handler_message).unwrap();
        // Then the message itself.
        S::encode(&message).unwrap();
        // Send the message
        unsafe { host::api::message::send(self.server.id()) };
    }
}

impl<M, S, T> Request<M, S> for ServerRef<T>
where
    T: Server,
    T::State: ServerRequest<M, S>,
    S: Serializer<M>
        + Serializer<Sendable>
        + Serializer<<T::State as ServerRequest<M, S>>::Response>,
{
    type Result = <T::State as ServerRequest<M, S>>::Response;

    /// Send request to the server and block until an answer is received.
    fn request(&self, request: M) -> Self::Result {
        fn unpacker<TU, MU, SU>(
            this: &mut TU,
            sender: Process<<TU as ServerRequest<MU, SU>>::Response, SU>,
        ) where
            TU: ServerRequest<MU, SU>,
            SU: Serializer<MU> + Serializer<<TU as ServerRequest<MU, SU>>::Response>,
        {
            // Get content out of message
            let message: MU = SU::decode().unwrap();
            // Get tag out of message before the handler function maybe manipulates it.
            let tag = unsafe { host::api::message::get_tag() };
            let tag = Tag::from(tag);
            let result = <TU as ServerRequest<MU, SU>>::handle(this, message);
            sender.tag_send(tag, result);
        }

        let tag = Tag::new();
        // Create new message buffer.
        unsafe { host::api::message::create_data(tag.id(), 0) };
        // Create reference to self
        let this: Process<()> = unsafe { Process::from_id(host::api::process::this()) };
        // First encode the handler inside the message buffer.
        let handler = unpacker::<T::State, M, S> as usize as i32;
        let handler_message = Sendable::Request(handler, this);
        S::encode(&handler_message).unwrap();
        // Then the message itself.
        S::encode(&request).unwrap();
        // Send it & wait on a response!
        unsafe { host::api::message::send_receive_skip_search(self.server.id(), 0) };
        S::decode().unwrap()
    }
}

// Processes are equal if their UUID is equal.
impl<T> PartialEq for ServerRef<T> {
    fn eq(&self, other: &Self) -> bool {
        self.uuid() == other.uuid()
    }
}

impl<T> std::fmt::Debug for ServerRef<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Server")
            .field("uuid", &self.uuid())
            .finish()
    }
}

impl<T> Clone for ServerRef<T> {
    fn clone(&self) -> Self {
        let id = unsafe { host::api::process::clone_process(self.server.id()) };
        unsafe { ServerRef::from(id) }
    }
}

impl<T> serde::Serialize for ServerRef<T> {
    fn serialize<B>(&self, serializer: B) -> Result<B::Ok, B::Error>
    where
        B: serde::Serializer,
    {
        // Mark process as consumed.
        unsafe { self.consume() };

        let index = unsafe { host::api::message::push_process(self.server.id()) };
        serializer.serialize_u64(index)
    }
}

struct ServerRefVisitor<T> {
    _phantom: PhantomData<T>,
}

impl<'de, T> serde::de::Visitor<'de> for ServerRefVisitor<T> {
    type Value = ServerRef<T>;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("an u64 index")
    }

    fn visit_u64<E>(self, index: u64) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        let id = unsafe { host::api::message::take_process(index) };
        Ok(unsafe { ServerRef::from(id) })
    }
}

impl<'de, T> serde::de::Deserialize<'de> for ServerRef<T> {
    fn deserialize<D>(deserializer: D) -> Result<ServerRef<T>, D::Error>
    where
        D: serde::de::Deserializer<'de>,
    {
        deserializer.deserialize_u64(ServerRefVisitor {
            _phantom: PhantomData {},
        })
    }
}

#[cfg(test)]
mod tests {
    use lunatic_test::test;
    use std::time::Duration;

    use super::*;
    use crate::sleep;

    struct TestServer(i32);

    #[derive(serde::Serialize, serde::Deserialize)]
    struct Inc(i32);
    #[derive(serde::Serialize, serde::Deserialize)]
    struct Count;
    #[derive(serde::Serialize, serde::Deserialize)]
    struct Panic;

    impl Server for TestServer {
        type Arg = ();
        type State = Self;

        fn init(_: ()) -> Self {
            TestServer(0)
        }
    }

    impl ServerMessage<Inc> for TestServer {
        fn handle(&mut self, message: Inc) {
            self.0 += message.0;
        }
    }

    impl ServerRequest<Count> for TestServer {
        type Response = i32;

        fn handle(&mut self, _: Count) -> Self::Response {
            self.0
        }
    }

    impl ServerMessage<Panic> for TestServer {
        fn handle(&mut self, _: Panic) {
            panic!("fail");
        }
    }

    #[test]
    fn spawn_test() {
        let child = TestServer::start((), None);
        child.send(Inc(33));
        child.send(Inc(55));
        let result = child.request(Count);
        assert_eq!(result, 88);
    }

    #[test]
    #[should_panic]
    fn spawn_link_test() {
        let child = TestServer::start_link((), None);
        child.send(Panic);
        // This process should fails too before 100ms
        sleep(Duration::from_millis(100));
    }
}
