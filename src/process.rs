use std::marker::PhantomData;

use crate::{
    host,
    mailbox::{LinkMailbox, LinkTrapped},
    serializer::{Bincode, Serializer},
    supervisor::{Supervisable, Supervisor, SupervisorConfig},
    Mailbox, Process, ProcessConfig, Resource, Tag,
};

/// Types that implement the `AbstractProcess` trait can be started as processes.
///
/// Their state can be mutated through messages and requests. To define a handler for them,
/// use [`ProcessMessage`] or [`ProcessRequest`].
///
/// [`Message`] provides a `send` method to send messages to the process, without waiting on a
/// response. [`Request`] provides a `request` method that will block until a response is received.
pub trait AbstractProcess {
    /// The argument received by the `init` function.
    ///
    /// This argument is sent from the parent to the child and needs to be serializable.
    type Arg: serde::Serialize + serde::de::DeserializeOwned;

    /// The state of the process.
    ///
    /// In most cases this value is set to `Self`.
    type State;

    /// Entry function of the new process.
    ///
    /// This function is executed inside the new process. It will receive the arguments passed
    /// to the `start` or `start_link` function by the parent. And will return the starting state
    /// of the newly spawned process.
    ///
    /// The parent will block on the call of `start` or `start_link` until this function finishes.
    /// This allows startups to be synchronized.
    fn init(this: ProcessRef<Self>, arg: Self::Arg) -> Self::State;

    /// Called when a `shutdown` command is received.
    fn terminate(_state: Self::State) {}

    /// This function will be called if the process is set to catch link deaths with
    /// `host::api::process::die_when_link_dies(1)` and a linked process traps.
    fn handle_link_trapped(_state: &mut Self::State, _tag: Tag) {}
}

/// Defines a handler for a message of type `M`.
pub trait ProcessMessage<M, S = Bincode>: AbstractProcess
where
    S: Serializer<M>,
{
    fn handle(state: &mut Self::State, message: M);
}

/// Defines a handler for a request of type `M`.
pub trait ProcessRequest<M, S = Bincode>: AbstractProcess
where
    S: Serializer<M>,
{
    type Response;

    fn handle(state: &mut Self::State, request: M) -> Self::Response;
}

pub trait StartProcess<T>
where
    T: AbstractProcess,
{
    fn start(arg: T::Arg, name: Option<&str>) -> ProcessRef<T>;
    fn start_config(arg: T::Arg, name: Option<&str>, config: &ProcessConfig) -> ProcessRef<T>;
    fn start_link(arg: T::Arg, name: Option<&str>) -> ProcessRef<T>;
    fn start_link_config(arg: T::Arg, name: Option<&str>, config: &ProcessConfig) -> ProcessRef<T>;
}

impl<T> StartProcess<T> for T
where
    T: AbstractProcess,
{
    /// Start a process.
    fn start(arg: T::Arg, name: Option<&str>) -> ProcessRef<T> {
        start::<T>(arg, name, None, None).unwrap()
    }

    /// Start a process with configuration.
    fn start_config(arg: T::Arg, name: Option<&str>, config: &ProcessConfig) -> ProcessRef<T> {
        start::<T>(arg, name, None, Some(config)).unwrap()
    }

    /// Start a linked process.
    fn start_link(arg: T::Arg, name: Option<&str>) -> ProcessRef<T> {
        start::<T>(arg, name, Some(Tag::new()), None).unwrap()
    }

    /// Start a linked process with configuration.
    fn start_link_config(arg: T::Arg, name: Option<&str>, config: &ProcessConfig) -> ProcessRef<T> {
        start::<T>(arg, name, Some(Tag::new()), Some(config)).unwrap()
    }
}

pub trait SelfReference<T> {
    /// Returns a reference to the currently running process.
    fn process(&self) -> ProcessRef<T>;
}

impl<T> SelfReference<T> for T
where
    T: AbstractProcess,
{
    fn process(&self) -> ProcessRef<T> {
        unsafe { ProcessRef::from(host::api::process::this()) }
    }
}

/// An internal interface that catches failures inside the `init` function of a `AbstractProcess`.
///
/// Only "link" functions are provided, because a panic can't be propagated to the parent without a
/// link. Currently, only the `Supervisor` uses this functionality to check for failures inside of
/// children.
pub(crate) trait StartFailableProcess<T>
where
    T: AbstractProcess,
{
    fn start_link_or_fail(
        arg: T::Arg,
        name: Option<&str>,
    ) -> Result<(ProcessRef<T>, Tag), LinkTrapped>;

    fn start_link_config_or_fail(
        arg: T::Arg,
        name: Option<&str>,
        config: &ProcessConfig,
    ) -> Result<(ProcessRef<T>, Tag), LinkTrapped>;
}

impl<T> StartFailableProcess<T> for T
where
    T: AbstractProcess,
{
    /// Start a linked process.
    fn start_link_or_fail(
        arg: T::Arg,
        name: Option<&str>,
    ) -> Result<(ProcessRef<T>, Tag), LinkTrapped> {
        let tag = Tag::new();
        let proc = start::<T>(arg, name, Some(tag), None)?;
        Ok((proc, tag))
    }

    /// Start a linked process with configuration.
    fn start_link_config_or_fail(
        arg: T::Arg,
        name: Option<&str>,
        config: &ProcessConfig,
    ) -> Result<(ProcessRef<T>, Tag), LinkTrapped> {
        let tag = Tag::new();
        let proc = start::<T>(arg, name, Some(tag), Some(config))?;
        Ok((proc, tag))
    }
}

fn start<T>(
    arg: T::Arg,
    name: Option<&str>,
    link: Option<Tag>,
    config: Option<&ProcessConfig>,
) -> Result<ProcessRef<T>, LinkTrapped>
where
    T: AbstractProcess,
{
    // If a link tag is provided, use the same tag for message matching.
    let tag = if let Some(tag) = link {
        tag
    } else {
        Tag::new()
    };
    let name = name.map(|name| name.to_owned());
    let parent = unsafe { <Process<(), Bincode> as Resource>::from_id(host::api::process::this()) };
    let process = if let Some(config) = config {
        if link.is_some() {
            Process::<(), Bincode>::spawn_link_config_tag(
                config,
                (parent, tag, arg, name, T::init as usize as i32),
                tag,
                starter::<T>,
            )
        } else {
            Process::<(), Bincode>::spawn_config(
                config,
                (parent, tag, arg, name, T::init as usize as i32),
                starter::<T>,
            )
        }
    } else if link.is_some() {
        Process::<(), Bincode>::spawn_link_tag(
            (parent, tag, arg, name, T::init as usize as i32),
            tag,
            starter::<T>,
        )
    } else {
        Process::<(), Bincode>::spawn(
            (parent, tag, arg, name, T::init as usize as i32),
            starter::<T>,
        )
    };

    // Don't return until `init()` finishes
    let mailbox: LinkMailbox<(), Bincode> = unsafe { LinkMailbox::new() };
    let _ = mailbox.tag_receive(Some(&[tag]))?;

    Ok(ProcessRef {
        process,
        phantom: PhantomData,
    })
}

// Entry point of the process.
fn starter<T>(
    (parent, tag, capture, name, entry): (Process<(), Bincode>, Tag, T::Arg, Option<String>, i32),
    _: Mailbox<(), Bincode>,
) where
    T: AbstractProcess,
{
    let entry: fn(this: ProcessRef<T>, arg: T::Arg) -> T::State =
        unsafe { std::mem::transmute(entry) };
    let this = unsafe { ProcessRef::from(host::api::process::this()) };

    // Register name
    let name = if let Some(name) = name {
        // Encode type information in name
        let name = format!("{} + ProcessRef + {}", name, std::any::type_name::<T>());
        unsafe { host::api::registry::put(name.as_ptr(), name.len(), this.process.id()) };
        Some(name)
    } else {
        None
    };

    let mut state = entry(this, capture);
    // Let parent know that the `init()` call finished
    parent.tag_send(tag, ());

    let mailbox: LinkMailbox<Sendable, Bincode> = unsafe { LinkMailbox::new() };
    // Run process forever and respond to requests.
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
                Sendable::Shutdown => {
                    T::terminate(state);
                    break;
                }
            },
            Err(link_trapped) => T::handle_link_trapped(&mut state, link_trapped.tag()),
        }
    }

    // Unregister name
    if let Some(name) = name {
        unsafe { host::api::registry::remove(name.as_ptr(), name.len()) };
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

/// A reference to a running process.
///
/// `ProcessRef<T>` is different from a `Process` in the ability to handle messages of different
/// types, as long as the traits `ProcessMessage<M>` or `ProcessRequest<R>` are implemented for T.
#[derive(serde::Serialize, serde::Deserialize)]
pub struct ProcessRef<T>
where
    T: ?Sized,
{
    process: Process<()>,
    phantom: PhantomData<T>,
}

impl<T> ProcessRef<T> {
    /// Construct a process from a raw ID.
    unsafe fn from(id: u64) -> Self {
        let process = <Process<()> as Resource>::from_id(id);
        ProcessRef {
            process,
            phantom: PhantomData,
        }
    }

    /// Returns a globally unique process ID.
    pub fn uuid(&self) -> u128 {
        let mut uuid: [u8; 16] = [0; 16];
        unsafe { host::api::process::id(self.process.id(), &mut uuid as *mut [u8; 16]) };
        u128::from_le_bytes(uuid)
    }

    pub fn lookup(name: &str) -> Option<Self> {
        let name = format!("{} + ProcessRef + {}", name, std::any::type_name::<T>());
        let mut id = 0;
        let result = unsafe { host::api::registry::get(name.as_ptr(), name.len(), &mut id) };
        if result == 0 {
            unsafe { Some(Self::from(id)) }
        } else {
            None
        }
    }
}

impl<T> Clone for ProcessRef<T> {
    fn clone(&self) -> Self {
        ProcessRef {
            process: self.process.clone(),
            phantom: PhantomData,
        }
    }
}

impl<T> ProcessRef<T>
where
    T: AbstractProcess,
{
    /// Shut down process
    pub fn shutdown(&self) {
        // Create new message buffer.
        unsafe { host::api::message::create_data(Tag::none().id(), 0) };
        Bincode::encode(&Sendable::Shutdown).unwrap();
        // Send the message
        unsafe { host::api::message::send(self.process.id()) };
    }
}

// This is a wrapper around the message/request that is sent to a process.
//
// The first `i32` value is a pointer
#[derive(serde::Serialize, serde::Deserialize)]
enum Sendable {
    Message(i32),
    // The process type can't be carried over as a generic and is set here to `()`, but overwritten
    // at the time of returning with the correct type.
    Request(i32, Process<()>),
    Shutdown,
}

impl<M, S, T> Message<M, S> for ProcessRef<T>
where
    T: AbstractProcess,
    T: ProcessMessage<M, S>,
    S: Serializer<M>,
{
    /// Send message to the process.
    fn send(&self, message: M) {
        fn unpacker<TU, MU, SU>(this: &mut TU::State)
        where
            TU: ProcessMessage<MU, SU>,
            SU: Serializer<MU>,
        {
            let message: MU = SU::decode().unwrap();
            <TU as ProcessMessage<MU, SU>>::handle(this, message);
        }

        // Create new message buffer.
        unsafe { host::api::message::create_data(Tag::none().id(), 0) };
        // First encode the handler inside the message buffer.
        let handler = unpacker::<T, M, S> as usize as i32;
        let handler_message = Sendable::Message(handler);
        Bincode::encode(&handler_message).unwrap();
        // Then the message itself.
        S::encode(&message).unwrap();
        // Send the message
        unsafe { host::api::message::send(self.process.id()) };
    }
}

impl<M, S, T> Request<M, S> for ProcessRef<T>
where
    T: AbstractProcess,
    T: ProcessRequest<M, S>,
    S: Serializer<M> + Serializer<Sendable> + Serializer<<T as ProcessRequest<M, S>>::Response>,
{
    type Result = <T as ProcessRequest<M, S>>::Response;

    /// Send request to the process and block until an answer is received.
    fn request(&self, request: M) -> Self::Result {
        fn unpacker<TU, MU, SU>(
            this: &mut TU::State,
            sender: Process<<TU as ProcessRequest<MU, SU>>::Response, SU>,
        ) where
            TU: ProcessRequest<MU, SU>,
            SU: Serializer<MU> + Serializer<<TU as ProcessRequest<MU, SU>>::Response>,
        {
            // Get content out of message
            let message: MU = SU::decode().unwrap();
            // Get tag out of message before the handler function maybe manipulates it.
            let tag = unsafe { host::api::message::get_tag() };
            let tag = Tag::from(tag);
            let result = <TU as ProcessRequest<MU, SU>>::handle(this, message);
            sender.tag_send(tag, result);
        }

        let tag = Tag::new();
        // Create new message buffer.
        unsafe { host::api::message::create_data(tag.id(), 0) };
        // Create reference to self
        let this: Process<()> = unsafe { Process::from_id(host::api::process::this()) };
        // First encode the handler inside the message buffer.
        let handler = unpacker::<T, M, S> as usize as i32;
        let handler_message = Sendable::Request(handler, this);
        S::encode(&handler_message).unwrap();
        // Then the message itself.
        S::encode(&request).unwrap();
        // Send it & wait on a response!
        unsafe { host::api::message::send_receive_skip_search(self.process.id(), 0) };
        S::decode().unwrap()
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
struct GetChildren;
impl<T> ProcessRequest<GetChildren> for T
where
    T: Supervisor,
    T: AbstractProcess<State = SupervisorConfig<T>>,
{
    type Response = <<T as Supervisor>::Children as Supervisable<T>>::Processes;

    fn handle(state: &mut Self::State, _: GetChildren) -> Self::Response {
        state.get_children()
    }
}

impl<T> ProcessRef<T>
where
    T: Supervisor,
    T: AbstractProcess<State = SupervisorConfig<T>>,
{
    pub fn children(&self) -> <<T as Supervisor>::Children as Supervisable<T>>::Processes {
        self.request(GetChildren)
    }
}

// Processes are equal if their UUID is equal.
impl<T> PartialEq for ProcessRef<T> {
    fn eq(&self, other: &Self) -> bool {
        self.uuid() == other.uuid()
    }
}

impl<T> std::fmt::Debug for ProcessRef<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ProcessRef")
            .field("uuid", &self.uuid())
            .finish()
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

    impl AbstractProcess for TestServer {
        type Arg = ();
        type State = Self;

        fn init(_: ProcessRef<Self>, _: ()) -> Self {
            TestServer(0)
        }
    }

    impl ProcessMessage<Inc> for TestServer {
        fn handle(state: &mut Self::State, message: Inc) {
            state.0 += message.0;
        }
    }

    impl ProcessRequest<Count> for TestServer {
        type Response = i32;

        fn handle(state: &mut Self::State, _: Count) -> Self::Response {
            state.0
        }
    }

    impl ProcessMessage<Panic> for TestServer {
        fn handle(_: &mut Self::State, _: Panic) {
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
        // This process should fail too before 100ms
        sleep(Duration::from_millis(100));
    }
}
