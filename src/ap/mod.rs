//! Contains the [`AbstractProcess`] abstraction.

mod builder;
mod lifecycles;
mod tag;

pub mod handlers;
pub(crate) mod messages;

use std::any::type_name;
use std::fmt::Debug;
use std::marker::PhantomData;
use std::mem;
use std::ops::{Deref, DerefMut};
use std::time::Duration;

use self::builder::AbstractProcessBuilder;
use self::handlers::{DeferredRequest, Handlers, Message, Request};
use self::messages::{RequestMessage, ReturnAddress, ShutdownMessage, SHUTDOWN_HANDLER};
use self::tag::AbstractProcessTag;
use crate::function::process::{process_name, ProcessType};
use crate::protocol::ProtocolCapture;
use crate::serializer::CanSerialize;
use crate::time::{Timeout, TimerRef, WithDelay, WithTimeout};
use crate::{host, Process, ProcessConfig, Tag};

/// Building block for processes that act as a server of a client-server
/// relation.
///
/// An `AbstractProcess` is like any other process in lunatic, it can hold
/// state, receive messages and so on. Their main advantage is that they
/// provide a type-safe interface for dealing with requests.
///
/// ### Startup
///
/// `AbstractProcesses` can be started using the [`Self::start`] function, or
/// [`Self::start_as`] for a named process. Calls to these functions will block
/// until the process is started and the [`Self::init`] function finishes. A
/// custom return error can be specified using the [`Self::StartupError`] type.
/// If the `init` function panics, the start functions will return a
/// [`StartupError::InitPanicked`] error.
///
/// ### Handlers
///
/// Handlers are used to define the types of messages that can be handled by
/// abstract processes. Handlers are defined using the traits
/// [`MessageHandler`], [`RequestHandler`] and [`DeferredRequestHandler`].
///
/// [`MessageHandler`] are used to handle asynchronous messages sent to the
/// abstract process. This means that the sender doesn't wait for an answer.
///
/// The following example shows a `Counter` abstract process that is able to
/// handle `Increment` messages. During the handling of a message the handler
/// has access to the internal state of the abstract process.
///
/// ```rust
/// #[derive(serde::Serialize, serde::Deserialize)]
/// struct Increment;
/// impl MessageHandler<Increment> for Counter {
///     fn handle(mut state: State<Self>, _: Increment) {
///         state.0 += 1;
///     }
/// }
/// ```
///
/// [`RequestHandler`] and [`DeferredRequestHandler`] expect a return value and
/// the requests are made synchronous, this means that the sender waits for a
/// response.
///
/// ```rust
/// #[derive(serde::Serialize, serde::Deserialize)]
/// struct Count;
/// impl RequestHandler<Count> for Counter {
///     type Response = u32;
///     fn handle(state: State<Self>, _: Count) -> Self::Response {
///         state.0
///     }
/// }
/// ```
///
/// In case of a [`DeferredRequestHandler`], the response doesn't need to be
/// immediate and can be even delegated to a 3rd process.
///
/// ```rust
/// impl DeferredRequestHandler<Count> for Counter {
///     type Response = u32;
///     fn handle(_: State<Self>, _: String, dr: DeferredResponse<Self::Response, Self>) {
///         dr.send_response(u32);
///     }
/// }
/// ```
///
/// _It is not enough just to define the handlers, they also need to be
/// associated with the `AbstractProcess` using the [`Self::Handlers`] type:_
///
/// ```rust
/// type Handlers = (Message<Increment>, Request<Count>, DeferredRequest<Count>);
/// ```
///
/// ### Shutdown
///
/// An abstract process can be shut down using the [`ProcessRef::shutdown`]
/// call. This function will block, until the [`Self::terminate`] function
/// finishes.
pub trait AbstractProcess: Sized
where
    // The serializer needs to be able to serialize types that are used
    // for starting up, shutting down and internal implementation
    // details. The following section lists all requirements:
    //
    // Arguments that are sent from parent to the `init` function
    Self::Serializer: CanSerialize<Self::Arg>,
    // Errors that can be returned during startup to the parent
    Self::Serializer: CanSerialize<Result<(), StartupError<Self>>>,
    // Every `AbstractProcess` needs to be able to receive a shutdown
    // message
    Self::Serializer: CanSerialize<ShutdownMessage<Self::Serializer>>,
    // This is more of an implementation detail. The internal reference
    // to the `AbstractProcess` will be held in the shape of a
    // `Process<(), Self::Serializer>` type. This requires the serializer
    // to work with `()`
    Self::Serializer: CanSerialize<()>,
    // Similar to the previous requirement, the next two are inherited
    // from the `Process::spawn_*` family of functions
    Self::Serializer: CanSerialize<(
        Process<Result<(), StartupError<Self>>, Self::Serializer>,
        Tag,
        Self::Arg,
    )>,
    Self::Serializer: CanSerialize<
        ProtocolCapture<(
            Process<Result<(), StartupError<Self>>, Self::Serializer>,
            Tag,
            Self::Arg,
        )>,
    >,
{
    /// The state of the process.
    ///
    /// This value is usually set to `Self`.
    type State;

    /// The serializer used for all messages sent to and responses sent from
    /// the abstract process.
    type Serializer;

    /// The argument received by the `init` function.
    ///
    /// This argument is sent from the parent to the child and needs to be
    /// serializable by `Self::Serializer`.
    type Arg;

    /// Handlers for incoming messages, requests and deferred requests.
    ///
    /// They are defined as a tuple and wrapped into `Message`, `Request` and
    /// `DeferredRequest` wrappers.
    /// ```
    /// type Handlers = (Message<Handler1>, Message<Handler2>, Request<Handler3>);
    /// ```
    ///
    /// Even if there is only one handler, it needs to be defined as a tuple.
    /// ```
    /// type Handlers = (Message<Handler1>,);
    /// ```
    type Handlers: Handlers<Self>;

    /// Errors that can be returned from the `init` call to the spawner.
    type StartupError: Debug;

    /// Entry function of the new process.
    ///
    /// This function is executed inside the new process. It will receive the
    /// arguments passed to the [`start`](AbstractProcess::start) or
    /// [`start_as`](AbstractProcess::start_as) function by the parent. And
    /// will return the starting state of the newly spawned process.
    ///
    /// The parent will block on the call of `start` or `start_as` until this
    /// function finishes. This allows startups to be synchronized.
    fn init(config: Config<Self>, arg: Self::Arg) -> Result<Self::State, Self::StartupError>;

    /// Called when a `shutdown` command is received.
    fn terminate(_state: Self::State) {}

    /// This function will be called if another linked process dies.
    fn handle_link_death(_state: State<Self>, _tag: Tag) {}

    /// Starts a new `AbstractProcess` and returns a reference to it.
    ///
    /// This call will block until the `init` function finishes. If the `init`
    /// function returns an error, it will be returned as
    /// `StartupError::Custom(error)`. If the `init` function panics during
    /// execution, it will return [`StartupError::InitPanicked`].
    #[track_caller]
    fn start(arg: Self::Arg) -> Result<ProcessRef<Self>, StartupError<Self>> {
        AbstractProcessBuilder::<Self>::new().start(arg)
    }

    /// Starts the process and registers it under `name`. If another process is
    /// already registered under the same name, it will return a
    /// `Err(StartupError::NameAlreadyRegistered(proc))` with a reference to the
    /// existing process.
    ///
    /// This call will block until the `init` function finishes. If the `init`
    /// function returns an error, it will be returned as
    /// `StartupError::Custom(error)`. If the `init` function panics during
    /// execution, it will return [`StartupError::InitPanicked`].
    ///
    /// If used in combination with the [`on_node`](Self::on_node) option, the
    /// name registration will be performed on the local node and not the remote
    /// one.
    #[track_caller]
    fn start_as<S: AsRef<str>>(
        name: S,
        arg: Self::Arg,
    ) -> Result<ProcessRef<Self>, StartupError<Self>> {
        AbstractProcessBuilder::<Self>::new().start_as(name, arg)
    }

    /// Links the to be spawned process to the parent.
    fn link() -> AbstractProcessBuilder<'static, Self> {
        AbstractProcessBuilder::new().link()
    }

    /// Links the to be spawned process to the parent with a specific [`Tag`].
    fn link_with(tag: Tag) -> AbstractProcessBuilder<'static, Self> {
        AbstractProcessBuilder::new().link_with(tag)
    }

    /// Allows for spawning the process with a specific configuration.
    fn configure(config: &ProcessConfig) -> AbstractProcessBuilder<Self> {
        AbstractProcessBuilder::new().configure(config)
    }

    /// Sets the node on which the process will be spawned.
    fn on_node(node: u64) -> AbstractProcessBuilder<'static, Self> {
        AbstractProcessBuilder::new().on_node(node)
    }
}

/// [`AbstractProcess`] startup configuration.
///
/// Available configuration options:
/// - [`die_if_link_dies`](Config::die_if_link_dies) - Sets if link deaths
///   should be caught.
///
/// The `Config` struct can also be used to acquire a self reference with
/// [`self_ref`](Config::self_ref) to send messages to itself during the
/// initialization process.
pub struct Config<AP: AbstractProcess> {
    phantom: PhantomData<AP>,
}

impl<AP: AbstractProcess> Config<AP> {
    /// Create a new configuration.
    pub(crate) fn new() -> Self {
        Config {
            phantom: PhantomData,
        }
    }

    /// If set to `true`, each link death will trigger the
    /// [`handle_link_death`](AbstractProcess::handle_link_death) handler.
    ///
    /// If set to `false` and a linked process dies, the [`AbstractProcess`]
    /// will die too.
    ///
    /// Default value is `false`.
    pub fn die_if_link_dies(&self, die: bool) {
        unsafe { host::api::process::die_when_link_dies(die as u32) };
    }

    /// Get a reference to the running [`AbstractProcess`].
    pub fn self_ref(&self) -> ProcessRef<AP> {
        let process = unsafe { Process::this() };
        ProcessRef { process }
    }
}

pub trait MessageHandler<Message>: AbstractProcess
where
    Self::Serializer: CanSerialize<Message>,
{
    fn handle(state: State<Self>, message: Message);
}

pub trait RequestHandler<Request>: AbstractProcess
where
    Self::Serializer: CanSerialize<Request>,
    Self::Serializer: CanSerialize<Self::Response>,
{
    type Response;

    fn handle(state: State<Self>, request: Request) -> Self::Response;
}

pub trait DeferredRequestHandler<Request>: AbstractProcess
where
    Self::Serializer: CanSerialize<Request>,
    Self::Serializer: CanSerialize<Self::Response>,
{
    type Response;

    fn handle(
        state: State<Self>,
        request: Request,
        deferred_response: DeferredResponse<Self::Response, Self>,
    );
}

/// A reference to the state inside handlers.
pub struct State<'a, AP: AbstractProcess> {
    state: &'a mut AP::State,
}

impl<'a, AP: AbstractProcess> State<'a, AP> {
    /// Get a reference to the running [`AbstractProcess`].
    pub fn self_ref(&self) -> ProcessRef<AP> {
        let process = unsafe { Process::this() };
        ProcessRef { process }
    }
}

impl<'a, AP: AbstractProcess> Deref for State<'a, AP> {
    type Target = AP::State;

    fn deref(&self) -> &Self::Target {
        self.state
    }
}

impl<'a, AP: AbstractProcess> DerefMut for State<'a, AP> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.state
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
#[serde(bound = "")]
pub struct DeferredResponse<Response, AP: AbstractProcess> {
    tag: Tag,
    return_address: ReturnAddress<Response, AP::Serializer>,
}

impl<Response, AP: AbstractProcess> DeferredResponse<Response, AP>
where
    AP::Serializer: CanSerialize<Response>,
{
    pub fn send_response(self, response: Response) {
        self.return_address.send_response(response, self.tag);
    }
}

/// A reference to a running [`AbstractProcess`].
///
/// `ProcessRef<T>` is different from a `Process` in the ability to handle
/// messages of different types, as long as the traits
/// `MessageHandler<Message>`, `RequestHandler<Request>` or
/// `DeferredRequestHandler<Request>` are implemented for `T`.
#[derive(serde::Serialize, serde::Deserialize)]
#[serde(bound = "")]
pub struct ProcessRef<T>
where
    T: AbstractProcess,
{
    // The `Process` generic value is set to `()` here. For every request, the value is going to
    // be different and will be transmuted just in time before the request is sent out.
    process: Process<(), T::Serializer>,
}

impl<T: AbstractProcess> Copy for ProcessRef<T> {}

impl<T> ProcessRef<T>
where
    T: AbstractProcess,
{
    /// Construct a process from a raw ID.
    unsafe fn new(node_id: u64, process_id: u64) -> Self {
        let process = Process::new(node_id, process_id);
        ProcessRef { process }
    }

    /// Returns the process ID.
    pub fn id(&self) -> u64 {
        self.process.id()
    }

    /// Returns the node ID.
    pub fn node_id(&self) -> u64 {
        self.process.node_id()
    }

    /// Returns a process registered under `name` if it exists and the signature
    /// matches.
    pub fn lookup<S: AsRef<str>>(name: S) -> Option<Self> {
        let name: &str = name.as_ref();
        let name = process_name::<T, T::Serializer>(ProcessType::ProcessRef, name);
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

    /// Registers process under `name`.
    pub fn register<S: AsRef<str>>(&self, name: S) {
        let name: &str = name.as_ref();
        let name = process_name::<T, T::Serializer>(ProcessType::ProcessRef, name);
        unsafe { host::api::registry::put(name.as_ptr(), name.len(), self.node_id(), self.id()) };
    }

    /// Returns `true` for processes on the local node that are running.
    ///
    /// Panics if called on a remote process.
    #[track_caller]
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
        self.link_with(Tag::new());
    }

    /// Link process to the one currently running with tag.
    pub fn link_with(&self, tag: Tag) {
        unsafe { host::api::process::link(tag.id(), self.process.id()) };
    }

    /// Unlink processes from the caller.
    pub fn unlink(&self) {
        unsafe { host::api::process::unlink(self.process.id()) };
    }

    /// Kill process
    pub fn kill(&self) {
        unsafe { host::api::process::kill(self.process.id()) };
    }

    /// Shuts the [`AbstractProcess`] down.
    #[track_caller]
    pub fn shutdown(&self)
    where
        // The serializer needs to be able to serialize values of `ShutdownMessage` & `()` for the
        // return value.
        T::Serializer: CanSerialize<ShutdownMessage<T::Serializer>>,
        T::Serializer: CanSerialize<()>,
    {
        self.shutdown_timeout(None).unwrap();
    }

    /// Shuts the [`AbstractProcess`] down.
    ///
    /// If a timeout is specified the function will only block for the timeout
    /// period before returning `Err(Timeout)`.
    #[track_caller]
    pub(crate) fn shutdown_timeout(&self, timeout: Option<Duration>) -> Result<(), Timeout>
    where
        // The serializer needs to be able to serialize values of `ShutdownMessage` & `()` for the
        // return value.
        T::Serializer: CanSerialize<ShutdownMessage<T::Serializer>>,
        T::Serializer: CanSerialize<()>,
    {
        let return_address = ReturnAddress::from_self();
        let message = ShutdownMessage(return_address);
        let send_tag = AbstractProcessTag::from_u6(SHUTDOWN_HANDLER);
        let (receive_tag, _) = AbstractProcessTag::extract_u6_data(send_tag);
        unsafe {
            // Cast into the right type for sending.
            let process: Process<ShutdownMessage<T::Serializer>, T::Serializer> =
                mem::transmute(self.process);
            match process.tag_send_receive(send_tag, receive_tag, message, timeout) {
                crate::MailboxResult::Message(()) => Ok(()),
                crate::MailboxResult::TimedOut => Err(Timeout),
                _ => unreachable!("send_receive should panic in case of other errors"),
            }
        }
    }

    /// Send message to the process.
    #[track_caller]
    pub fn send<M: 'static>(&self, message: M)
    where
        T::Serializer: CanSerialize<M>,
    {
        let handler_id = T::Handlers::handler_id::<Message<M>>();
        let tag = AbstractProcessTag::from_u6(handler_id);
        // Cast into the right type for sending.
        let process: Process<M, T::Serializer> = unsafe { std::mem::transmute(self.process) };
        process.tag_send(tag, message);
    }

    /// Send message to the process after the specified duration has passed.
    #[track_caller]
    pub(crate) fn delayed_send<M: 'static>(&self, message: M, duration: Duration) -> TimerRef
    where
        T::Serializer: CanSerialize<M>,
    {
        let handler_id = T::Handlers::handler_id::<Message<M>>();
        let tag = AbstractProcessTag::from_u6(handler_id);
        // Cast into the right type for sending.
        let process: Process<M, T::Serializer> = unsafe { std::mem::transmute(self.process) };
        process.tag_send_after(tag, message, duration)
    }

    /// Make a request to the process.
    #[track_caller]
    pub fn request<R: 'static>(&self, request: R) -> T::Response
    where
        T: RequestHandler<R>,
        T::Serializer: CanSerialize<R>,
        T::Serializer: CanSerialize<T::Response>,
        T::Serializer: CanSerialize<RequestMessage<R, T::Response, T::Serializer>>,
    {
        self.request_timeout(request, None).unwrap()
    }

    /// Make a request to the process.
    //
    /// If a timeout is specified the function will only block for the timeout
    /// period before returning `Err(Timeout)`.
    #[track_caller]
    pub(crate) fn request_timeout<R: 'static>(
        &self,
        request: R,
        timeout: Option<Duration>,
    ) -> Result<T::Response, Timeout>
    where
        T: RequestHandler<R>,
        T::Serializer: CanSerialize<R>,
        T::Serializer: CanSerialize<T::Response>,
        T::Serializer: CanSerialize<RequestMessage<R, T::Response, T::Serializer>>,
    {
        let return_address = ReturnAddress::from_self();
        let message = RequestMessage(request, return_address);
        let handler_id = T::Handlers::handler_id::<Request<R>>();
        let send_tag = AbstractProcessTag::from_u6(handler_id);
        let (receive_tag, _) = AbstractProcessTag::extract_u6_data(send_tag);
        unsafe {
            // Cast into the right type for sending.
            let process: Process<RequestMessage<R, T::Response, T::Serializer>, T::Serializer> =
                mem::transmute(self.process);
            match process.tag_send_receive(send_tag, receive_tag, message, timeout) {
                crate::MailboxResult::Message(message) => Ok(message),
                crate::MailboxResult::TimedOut => Err(Timeout),
                _ => unreachable!("send_receive should panic in case of other errors"),
            }
        }
    }

    /// Make a deferred request to the process.
    #[track_caller]
    pub fn deferred_request<R: 'static>(&self, request: R) -> T::Response
    where
        T: DeferredRequestHandler<R>,
        T::Serializer: CanSerialize<R>,
        T::Serializer: CanSerialize<T::Response>,
        T::Serializer: CanSerialize<RequestMessage<R, T::Response, T::Serializer>>,
    {
        self.deferred_request_timeout(request, None).unwrap()
    }

    /// Make a deferred request to the process.
    //
    /// If a timeout is specified the function will only block for the timeout
    /// period before returning `Err(Timeout)`.
    #[track_caller]
    pub(crate) fn deferred_request_timeout<R: 'static>(
        &self,
        request: R,
        timeout: Option<Duration>,
    ) -> Result<T::Response, Timeout>
    where
        T: DeferredRequestHandler<R>,
        T::Serializer: CanSerialize<R>,
        T::Serializer: CanSerialize<T::Response>,
        T::Serializer: CanSerialize<RequestMessage<R, T::Response, T::Serializer>>,
    {
        let return_address = ReturnAddress::from_self();
        let message = RequestMessage(request, return_address);
        let handler_id = T::Handlers::handler_id::<DeferredRequest<R>>();
        let send_tag = AbstractProcessTag::from_u6(handler_id);
        let (receive_tag, _) = AbstractProcessTag::extract_u6_data(send_tag);
        unsafe {
            // Cast into the right type for sending.
            let process: Process<RequestMessage<R, T::Response, T::Serializer>, T::Serializer> =
                mem::transmute(self.process);
            match process.tag_send_receive(send_tag, receive_tag, message, timeout) {
                crate::MailboxResult::Message(message) => Ok(message),
                crate::MailboxResult::TimedOut => Err(Timeout),
                _ => unreachable!("send_receive should panic in case of other errors"),
            }
        }
    }

    /// Set a timeout on the next action performed on this process.
    ///
    /// Timeouts affect [`ProcessRef::shutdown`], [`ProcessRef::request`] and
    /// [`ProcessRef::deferred_request`] functions.
    pub fn with_timeout(self, timeout: Duration) -> WithTimeout<ProcessRef<T>> {
        WithTimeout::from(timeout, self)
    }

    /// Set a delay on the next [`ProcessRef::send`] performed on this process.
    ///
    /// This is a non-blocking function, meaning that `send` is going to be
    /// performed in the background while the execution continues. The `send`
    /// call will return a reference to the timer allowing you to cancel it.
    pub fn with_delay(self, timeout: Duration) -> WithDelay<ProcessRef<T>> {
        WithDelay::from(timeout, self)
    }
}

impl<T> Debug for ProcessRef<T>
where
    T: AbstractProcess,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = format!("ProcessRef<{}>", type_name::<T>());
        f.debug_struct(&name)
            .field("id", &self.process.id())
            .finish()
    }
}

impl<T> Clone for ProcessRef<T>
where
    T: AbstractProcess,
{
    fn clone(&self) -> Self {
        ProcessRef {
            process: self.process,
        }
    }
}

impl<T> PartialEq for ProcessRef<T>
where
    T: AbstractProcess,
{
    fn eq(&self, other: &Self) -> bool {
        self.process == other.process
    }
}

impl<T> Eq for ProcessRef<T> where T: AbstractProcess {}

/// Result of [`AbstractProcess::start`].
#[derive(serde::Serialize, serde::Deserialize)]
pub enum StartupError<AP: AbstractProcess> {
    /// The `init` function of the `AbstractProcess` panicked.
    InitPanicked,
    /// The name supplied to `start_as` is already registered.
    #[serde(bound(serialize = "", deserialize = ""))]
    NameAlreadyRegistered(ProcessRef<AP>),
    /// A custom error.
    Custom(AP::StartupError),
}

impl<AP: AbstractProcess> Debug for StartupError<AP>
where
    AP::StartupError: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InitPanicked => write!(f, "InitPanicked"),
            Self::NameAlreadyRegistered(arg0) => {
                f.debug_tuple("NameAlreadyRegistered").field(arg0).finish()
            }
            Self::Custom(arg0) => f.debug_tuple("Custom").field(arg0).finish(),
        }
    }
}

impl<AP: AbstractProcess> Clone for StartupError<AP>
where
    AP::StartupError: Clone,
{
    fn clone(&self) -> Self {
        match self {
            Self::InitPanicked => Self::InitPanicked,
            Self::NameAlreadyRegistered(arg0) => Self::NameAlreadyRegistered(arg0.clone()),
            Self::Custom(arg0) => Self::Custom(arg0.clone()),
        }
    }
}

impl<AP: AbstractProcess> PartialEq for StartupError<AP>
where
    AP::StartupError: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::NameAlreadyRegistered(l0), Self::NameAlreadyRegistered(r0)) => l0 == r0,
            (Self::Custom(l0), Self::Custom(r0)) => l0 == r0,
            _ => core::mem::discriminant(self) == core::mem::discriminant(other),
        }
    }
}

impl<AP: AbstractProcess> Eq for StartupError<AP> where AP::StartupError: Eq {}
