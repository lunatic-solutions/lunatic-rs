use std::marker::PhantomData;
use std::time::Duration;

use crate::distributed::node_id;
use crate::host::{self, api};
use crate::mailbox::Catching;
use crate::serializer::{Bincode, CanSerialize};
use crate::timer::TimerRef;
use crate::{Mailbox, MailboxResult, Process, ProcessConfig, Tag};

pub fn process_id() -> u64 {
    unsafe { api::process::process_id() }
}

/// Types that implement the `AbstractProcess` trait can be started as
/// processes.
///
/// Their state can be mutated through messages and requests. To define a
/// handler for them, use [`MessageHandler`] or [`RequestHandler`].
///
/// [`Message`] provides a `send` method to send messages to the process,
/// without waiting on a response. [`Request`] provides a `request` method that
/// will block until a response is received.
///
/// # Example
///
/// ```
/// use lunatic::process::{
///     AbstractProcess, Message, MessageHandler, ProcessRef, RequestHandler,
///     Request, StartProcess,
/// };
///
/// struct Counter(u32);
///
/// impl AbstractProcess for Counter {
///     type Arg = u32;
///     type State = Self;
///
///     fn init(_: ProcessRef<Self>, start: u32) -> Self {
///         Self(start)
///     }
/// }
///
/// #[derive(serde::Serialize, serde::Deserialize)]
/// struct Inc;
/// impl MessageHandler<Inc> for Counter {
///     fn handle(state: &mut Self::State, _: Inc) {
///         state.0 += 1;
///     }
/// }
///
/// #[derive(serde::Serialize, serde::Deserialize)]
/// struct Count;
/// impl RequestHandler<Count> for Counter {
///     type Response = u32;
///
///     fn handle(state: &mut Self::State, _: Count) -> u32 {
///         state.0
///     }
/// }
///
///
/// let counter = Counter::start(5, None);
/// counter.send(Inc);
/// assert_eq!(counter.request(Count), 6);
/// ```
pub trait AbstractProcess {
    /// The argument received by the `init` function.
    ///
    /// This argument is sent from the parent to the child and needs to be
    /// serializable.
    type Arg: serde::Serialize + serde::de::DeserializeOwned;

    /// The state of the process.
    ///
    /// In most cases this value is set to `Self`.
    type State;

    /// Entry function of the new process.
    ///
    /// This function is executed inside the new process. It will receive the
    /// arguments passed to the [`start`](StartProcess::start) or
    /// [`start_link`](StartProcess::start_link) function by the parent. And
    /// will return the starting state of the newly spawned process.
    ///
    /// The parent will block on the call of `start` or `start_link` until this
    /// function finishes. This allows startups to be synchronized.
    fn init(this: ProcessRef<Self>, arg: Self::Arg) -> Self::State;

    /// Called when a `shutdown` command is received.
    fn terminate(_state: Self::State) {}

    /// This function will be called if the process is set to catch link deaths
    /// with `host::api::process::die_when_link_dies(1)` and a linked
    /// process traps.
    fn handle_link_trapped(_state: &mut Self::State, _tag: Tag) {}
}

/// Defines a handler for a message of type `M`.
pub trait MessageHandler<M, S = Bincode>: AbstractProcess
where
    S: CanSerialize<M>,
{
    fn handle(state: &mut Self::State, message: M);
}

/// Defines a handler for a request of type `M`.
pub trait RequestHandler<M, S = Bincode>: AbstractProcess
where
    S: CanSerialize<M>,
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
    fn start_node(arg: T::Arg, name: Option<&str>, node: u64) -> ProcessRef<T>;
    fn start_node_config(
        arg: T::Arg,
        name: Option<&str>,
        node: u64,
        config: &ProcessConfig,
    ) -> ProcessRef<T>;
}

impl<T> StartProcess<T> for T
where
    T: AbstractProcess,
{
    /// Start a process.
    #[track_caller]
    fn start(arg: T::Arg, name: Option<&str>) -> ProcessRef<T> {
        start::<T>(arg, name, None, None, None).unwrap()
    }

    /// Start a process with configuration.
    #[track_caller]
    fn start_config(arg: T::Arg, name: Option<&str>, config: &ProcessConfig) -> ProcessRef<T> {
        start::<T>(arg, name, None, Some(config), None).unwrap()
    }

    /// Start a linked process.
    #[track_caller]
    fn start_link(arg: T::Arg, name: Option<&str>) -> ProcessRef<T> {
        start::<T>(arg, name, Some(Tag::new()), None, None).unwrap()
    }

    /// Start a linked process with configuration.
    #[track_caller]
    fn start_link_config(arg: T::Arg, name: Option<&str>, config: &ProcessConfig) -> ProcessRef<T> {
        start::<T>(arg, name, Some(Tag::new()), Some(config), None).unwrap()
    }

    #[track_caller]
    fn start_node(
        arg: <T as AbstractProcess>::Arg,
        name: Option<&str>,
        node: u64,
    ) -> ProcessRef<T> {
        start::<T>(arg, name, None, None, Some(node)).unwrap()
    }

    #[track_caller]
    fn start_node_config(
        arg: <T as AbstractProcess>::Arg,
        name: Option<&str>,
        node: u64,
        config: &ProcessConfig,
    ) -> ProcessRef<T> {
        start::<T>(arg, name, None, Some(config), Some(node)).unwrap()
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
        unsafe { ProcessRef::new(node_id(), process_id()) }
    }
}

/// An internal interface that catches failures inside the `init` function of a
/// `AbstractProcess`.
///
/// Only "link" functions are provided, because a panic can't be propagated to
/// the parent without a link. Currently, only the `Supervisor` uses this
/// functionality to check for failures inside of children.
pub(crate) trait StartFailableProcess<T>
where
    T: AbstractProcess,
{
    fn start_link_or_fail(arg: T::Arg, name: Option<&str>) -> MailboxResult<(ProcessRef<T>, Tag)>;

    fn start_link_config_or_fail(
        arg: T::Arg,
        name: Option<&str>,
        config: &ProcessConfig,
    ) -> MailboxResult<(ProcessRef<T>, Tag)>;
}

impl<T> StartFailableProcess<T> for T
where
    T: AbstractProcess,
{
    /// Start a linked process.
    fn start_link_or_fail(arg: T::Arg, name: Option<&str>) -> MailboxResult<(ProcessRef<T>, Tag)> {
        let tag = Tag::new();
        match start::<T>(arg, name, Some(tag), None, None) {
            MailboxResult::Message(proc) => MailboxResult::Message((proc, tag)),
            MailboxResult::DeserializationFailed(err) => MailboxResult::DeserializationFailed(err),
            MailboxResult::TimedOut => MailboxResult::TimedOut,
            MailboxResult::LinkDied(tag) => MailboxResult::LinkDied(tag),
        }
    }

    /// Start a linked process with configuration.
    fn start_link_config_or_fail(
        arg: T::Arg,
        name: Option<&str>,
        config: &ProcessConfig,
    ) -> MailboxResult<(ProcessRef<T>, Tag)> {
        let tag = Tag::new();
        match start::<T>(arg, name, Some(tag), Some(config), None) {
            MailboxResult::Message(proc) => MailboxResult::Message((proc, tag)),
            MailboxResult::DeserializationFailed(err) => MailboxResult::DeserializationFailed(err),
            MailboxResult::TimedOut => MailboxResult::TimedOut,
            MailboxResult::LinkDied(tag) => MailboxResult::LinkDied(tag),
        }
    }
}

fn start<T>(
    arg: T::Arg,
    name: Option<&str>,
    link: Option<Tag>,
    config: Option<&ProcessConfig>,
    node: Option<u64>,
) -> MailboxResult<ProcessRef<T>>
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
    let parent = <Process<(), Bincode>>::new(node_id(), process_id());
    let process = if let Some(node) = node {
        // no link or config
        Process::<(), Bincode>::spawn_node(
            node,
            (parent, tag, arg, name, T::init as usize as i32),
            starter::<T>,
        )
    } else if let Some(config) = config {
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
    let mailbox: Mailbox<(), Bincode, Catching> = unsafe { Mailbox::new() };
    match mailbox.tag_receive(&[tag]) {
        MailboxResult::Message(_) => MailboxResult::Message(ProcessRef {
            process,
            phantom: PhantomData,
        }),
        MailboxResult::DeserializationFailed(err) => MailboxResult::DeserializationFailed(err),
        MailboxResult::TimedOut => MailboxResult::TimedOut,
        MailboxResult::LinkDied(tag) => MailboxResult::LinkDied(tag),
    }
}

/// Entry point of the process.
fn starter<T>(
    (parent, tag, capture, name, entry): (Process<(), Bincode>, Tag, T::Arg, Option<String>, i32),
    _: Mailbox<(), Bincode>,
) where
    T: AbstractProcess,
{
    let entry: fn(this: ProcessRef<T>, arg: T::Arg) -> T::State =
        unsafe { std::mem::transmute(entry as usize) };
    let this = unsafe { ProcessRef::new(node_id(), process_id()) };

    // Register name
    let name = if let Some(name) = name {
        // Encode type information in name
        let name = format!("{} + ProcessRef + {}", name, std::any::type_name::<T>());
        unsafe {
            host::api::registry::put(
                name.as_ptr(),
                name.len(),
                this.process.node_id(),
                this.process.id(),
            )
        };
        Some(name)
    } else {
        None
    };

    let mut state = entry(this, capture);
    // Let parent know that the `init()` call finished
    parent.tag_send(tag, ());

    let mailbox: Mailbox<Sendable, Bincode, Catching> = unsafe { Mailbox::new() };
    // Run process forever and respond to requests.
    loop {
        let dispatcher = mailbox.tag_receive(&[]);
        match dispatcher {
            MailboxResult::Message(dispatcher) => match dispatcher {
                Sendable::Message(handler) => {
                    let handler: fn(state: &mut T::State) =
                        unsafe { std::mem::transmute(handler as usize) };
                    handler(&mut state);
                }
                Sendable::Request(handler, sender) => {
                    let handler: fn(state: &mut T::State, sender: Process<()>) =
                        unsafe { std::mem::transmute(handler as usize) };
                    handler(&mut state, sender);
                }
                Sendable::Shutdown(sender) => {
                    // Get tag out of message first
                    let tag = unsafe { host::api::message::get_tag() };
                    let tag = Tag::from(tag);
                    T::terminate(state);
                    sender.tag_send(tag, ());
                    break;
                }
            },
            MailboxResult::LinkDied(tag) => T::handle_link_trapped(&mut state, tag),
            _ => unreachable!(),
        }
    }

    // Unregister name
    if let Some(name) = name {
        unsafe { host::api::registry::remove(name.as_ptr(), name.len()) };
    }
}

pub trait Message<M, S>
where
    S: CanSerialize<M>,
{
    fn send(&self, message: M);
    fn send_after(&self, message: M, duration: Duration) -> TimerRef;
}

pub trait Request<M, S>
where
    S: CanSerialize<M>,
{
    type Result;

    fn request(&self, request: M) -> Self::Result {
        self.request_timeout_(request, None).unwrap()
    }

    fn request_timeout(&self, request: M, timeout: Duration) -> MailboxResult<Self::Result> {
        self.request_timeout_(request, Some(timeout))
    }

    #[doc(hidden)]
    fn request_timeout_(
        &self,
        request: M,
        timeout: Option<Duration>,
    ) -> MailboxResult<Self::Result>;
}

/// A reference to a running process.
///
/// `ProcessRef<T>` is different from a `Process` in the ability to handle
/// messages of different types, as long as the traits `MessageHandler<M>` or
/// `RequestHandler<R>` are implemented for T.
#[derive(serde::Serialize, serde::Deserialize)]
#[serde(bound = "")]
pub struct ProcessRef<T>
where
    T: ?Sized,
{
    pub(crate) process: Process<()>,
    phantom: PhantomData<T>,
}

impl<T> ProcessRef<T> {
    /// Construct a process from a raw ID.
    unsafe fn new(node_id: u64, process_id: u64) -> Self {
        let process = <Process<()>>::new(node_id, process_id);
        ProcessRef {
            process,
            phantom: PhantomData,
        }
    }

    /// Returns a globally unique process ID.
    pub fn id(&self) -> u64 {
        self.process.id()
    }

    pub fn lookup(name: &str) -> Option<Self> {
        let name = format!("{} + ProcessRef + {}", name, std::any::type_name::<T>());
        let mut id = 0;
        let mut node_id = 0;
        let result =
            unsafe { host::api::registry::get(name.as_ptr(), name.len(), &mut node_id, &mut id) };
        if result == 0 {
            unsafe { Some(Self::new(node_id, id)) }
        } else {
            None
        }
    }

    /// Returns `true` for processes on the local node that are running.
    ///
    /// Panics if called on a remote process.
    pub fn is_alive(&self) -> bool {
        assert_eq!(
            self.process.node_id(),
            host::node_id(),
            "is_alive() can only be used with local processes"
        );
        unsafe { host::api::process::exists(self.process.id()) != 0 }
    }

    /// Link process to the one currently running.
    pub fn link(&self) {
        // Don't use tags because a process' [`Mailbox`] can't differentiate between
        // regular messages and signals. Both processes should almost always die
        // when a link is broken.
        unsafe { host::api::process::link(0, self.process.id()) };
    }

    /// Unlink processes from the caller.
    pub fn unlink(&self) {
        unsafe { host::api::process::unlink(self.process.id()) };
    }

    /// Kill this process
    pub fn kill(&self) {
        unsafe { host::api::process::kill(self.process.id()) };
    }
}

impl<T> Clone for ProcessRef<T> {
    fn clone(&self) -> Self {
        ProcessRef {
            process: self.process,
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
        self.shutdown_timeout_(None).unwrap()
    }

    /// Shut down process with a timeout
    pub fn shutdown_timeout(&self, timeout: Duration) -> MailboxResult<()> {
        self.shutdown_timeout_(Some(timeout))
    }

    fn shutdown_timeout_(&self, timeout: Option<Duration>) -> MailboxResult<()> {
        // Create new message buffer.
        let tag = Tag::new();
        unsafe { host::api::message::create_data(tag.id(), 0) };

        // Create reference to self
        let this: Process<()> = Process::this();

        Bincode::encode(&Sendable::Shutdown(this)).unwrap();
        let timeout_ms = match timeout {
            Some(timeout) => timeout.as_millis() as u64,
            None => u64::MAX,
        };
        // Send the message and wait for response
        let result =
            host::send_receive_skip_search(self.process.node_id(), self.process.id(), timeout_ms);
        if result == 9027 {
            return MailboxResult::TimedOut;
        }
        MailboxResult::Message(())
    }
}

/// This is a wrapper around the message/request that is sent to a process.
///
/// The first `i32` value is a pointer
#[derive(serde::Serialize, serde::Deserialize)]
pub(crate) enum Sendable {
    Message(i32),
    // The process type can't be carried over as a generic and is set here to `()`, but
    // overwritten at the time of returning with the correct type.
    Request(i32, Process<()>),
    Shutdown(Process<()>),
}

impl<M, S, T> Message<M, S> for ProcessRef<T>
where
    T: AbstractProcess,
    T: MessageHandler<M, S>,
    S: CanSerialize<M>,
{
    /// Send message to the process.
    fn send(&self, message: M) {
        fn unpacker<TU, MU, SU>(this: &mut TU::State)
        where
            TU: MessageHandler<MU, SU>,
            SU: CanSerialize<MU>,
        {
            let message: MU = SU::decode().unwrap();
            <TU as MessageHandler<MU, SU>>::handle(this, message);
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
        host::send(self.process.node_id(), self.process.id());
    }

    /// Send message to the process after the specified duration has passed.
    fn send_after(&self, message: M, duration: Duration) -> TimerRef {
        fn unpacker<TU, MU, SU>(this: &mut TU::State)
        where
            TU: MessageHandler<MU, SU>,
            SU: CanSerialize<MU>,
        {
            let message: MU = SU::decode().unwrap();
            <TU as MessageHandler<MU, SU>>::handle(this, message);
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
        let timer_id =
            unsafe { host::api::timer::send_after(self.process.id(), duration.as_millis() as u64) };
        TimerRef::new(timer_id)
    }
}

impl<M, S, T> Request<M, S> for ProcessRef<T>
where
    T: AbstractProcess,
    T: RequestHandler<M, S>,
    S: CanSerialize<M> + CanSerialize<<T as RequestHandler<M, S>>::Response>,
{
    type Result = <T as RequestHandler<M, S>>::Response;

    fn request_timeout_(
        &self,
        request: M,
        timeout: Option<Duration>,
    ) -> MailboxResult<Self::Result> {
        fn unpacker<TU, MU, SU>(
            this: &mut TU::State,
            sender: Process<<TU as RequestHandler<MU, SU>>::Response, SU>,
        ) where
            TU: RequestHandler<MU, SU>,
            SU: CanSerialize<MU> + CanSerialize<<TU as RequestHandler<MU, SU>>::Response>,
        {
            // Get content out of message
            let message: MU = SU::decode().unwrap();
            // Get tag out of message before the handler function maybe manipulates it.
            let tag = unsafe { host::api::message::get_tag() };
            let tag = Tag::from(tag);
            let result = <TU as RequestHandler<MU, SU>>::handle(this, message);
            sender.tag_send(tag, result);
        }

        let tag = Tag::new();
        // Create new message buffer.
        unsafe { host::api::message::create_data(tag.id(), 0) };
        // Create reference to self
        let this: Process<()> = Process::new(node_id(), process_id());
        // First encode the handler inside the message buffer.
        let handler = unpacker::<T, M, S> as usize as i32;
        let handler_message = Sendable::Request(handler, this);
        Bincode::encode(&handler_message).unwrap();
        // Then the message itself.
        S::encode(&request).unwrap();
        // Send it & wait on a response!
        let timeout_ms = match timeout {
            Some(timeout) => timeout.as_millis() as u64,
            None => u64::MAX,
        };
        let result =
            host::send_receive_skip_search(self.process.node_id(), self.process.id(), timeout_ms);
        if result == 9027 {
            return MailboxResult::TimedOut;
        };
        MailboxResult::Message(S::decode().unwrap())
    }
}

/// Processes are equal if their process id and node id are equal
impl<T> PartialEq for ProcessRef<T> {
    fn eq(&self, other: &Self) -> bool {
        self.process == other.process
    }
}

/// Proccess equality comparison is an equivalance relation
impl<T> Eq for ProcessRef<T> {}

// Implement Hash explicitly to match the behavior of PartialEq
impl<T> std::hash::Hash for ProcessRef<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.process.hash(state);
    }
}

impl<T> std::fmt::Debug for ProcessRef<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ProcessRef")
            .field("uuid", &self.id())
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use lunatic_test::test;

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

    impl MessageHandler<Inc> for TestServer {
        fn handle(state: &mut Self::State, message: Inc) {
            state.0 += message.0;
        }
    }

    impl RequestHandler<Count> for TestServer {
        type Response = i32;

        fn handle(state: &mut Self::State, _: Count) -> Self::Response {
            state.0
        }
    }

    impl MessageHandler<Panic> for TestServer {
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
