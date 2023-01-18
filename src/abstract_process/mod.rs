mod builder;
mod lifecycles;
mod messages;
mod tag;

pub mod handlers;

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
use crate::protocol::ProtocolCapture;
use crate::{host, serializer, Process, ProcessConfig, Tag};

pub trait AbstractProcess: Sized
where
    // The serializer needs to be able to serialize the arguments used
    // for initialization
    Self::Serializer: serializer::Serializer<Self::Arg>,
    // and errors that happen during the startup and need to be communicated to the parent
    Self::Serializer: serializer::Serializer<Result<(), StartupError<Self>>>,
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
    /// They are defined as a tuple and wrapped into `Message`, `Request`
    /// wrappers. E.g.
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
    type StartupError;

    /// Entry function of the new process.
    ///
    /// This function is executed inside the new process. It will receive the
    /// arguments passed to the [`start`](AbstractProcess::start) or
    /// [`start_link`](AbstractProcess::start_link) function by the parent. And
    /// will return the starting state of the newly spawned process.
    ///
    /// The parent will block on the call of `start` or `start_link` until this
    /// function finishes. This allows startups to be synchronized.
    fn init(config: Config<Self>, arg: Self::Arg) -> Result<Self::State, Self::StartupError>;

    /// Called when a `shutdown` command is received.
    fn terminate(_state: Self::State) {}

    /// This function will be called if another linked process dies.
    fn handle_link_death(_state: &mut Self::State, _tag: Tag) {}

    /// Starts a new `AbstractProcess` and returns a reference to it.
    fn start(arg: Self::Arg) -> Result<ProcessRef<Self>, StartupError<Self>>
    where
        // TODO: Clean up serialization dependencies
        Self::Serializer: serializer::Serializer<()>,
        Self::Serializer: serializer::Serializer<(
            Process<Result<(), StartupError<Self>>, Self::Serializer>,
            Tag,
            Self::Arg,
        )>,
        Self::Serializer: serializer::Serializer<ProtocolCapture<Self::Arg>>,
        Self::Serializer: serializer::Serializer<
            ProtocolCapture<(
                Process<Result<(), StartupError<Self>>, Self::Serializer>,
                Tag,
                Self::Arg,
            )>,
        >,
        Self::Serializer: serializer::Serializer<ProtocolCapture<ProtocolCapture<Self::Arg>>>,
        Self::Serializer: serializer::Serializer<ShutdownMessage<(), Self::Serializer>>,
    {
        AbstractProcessBuilder::<Self>::new().start(arg)
    }

    /// Starts the process and registers it under `name`. If another process is
    /// already registered under the same name, it will return a
    /// `Err(StartupError::NameAlreadyRegistered(proc))` with a reference to the
    /// existing process.
    ///
    /// If used in combination with the [`on_node`](Self::on_node) option, the
    /// name registration will be performed on the local and not the remote
    /// node.
    fn start_as<S: AsRef<str>>(
        name: S,
        arg: Self::Arg,
    ) -> Result<ProcessRef<Self>, StartupError<Self>>
    where
        // TODO: Clean up serialization dependencies
        Self::Serializer: serializer::Serializer<()>,
        Self::Serializer: serializer::Serializer<(
            Process<Result<(), StartupError<Self>>, Self::Serializer>,
            Tag,
            Self::Arg,
        )>,
        Self::Serializer: serializer::Serializer<ProtocolCapture<Self::Arg>>,
        Self::Serializer: serializer::Serializer<
            ProtocolCapture<(
                Process<Result<(), StartupError<Self>>, Self::Serializer>,
                Tag,
                Self::Arg,
            )>,
        >,
        Self::Serializer: serializer::Serializer<ProtocolCapture<ProtocolCapture<Self::Arg>>>,
        Self::Serializer: serializer::Serializer<ShutdownMessage<(), Self::Serializer>>,
    {
        AbstractProcessBuilder::<Self>::new().start_as(name, arg)
    }

    fn link() -> AbstractProcessBuilder<Self> {
        AbstractProcessBuilder::new().link()
    }

    fn link_with(tag: Tag) -> AbstractProcessBuilder<Self> {
        AbstractProcessBuilder::new().link_with(tag)
    }

    fn configure(config: ProcessConfig) -> AbstractProcessBuilder<Self> {
        AbstractProcessBuilder::new().configure(config)
    }

    fn on_node(node: u64) -> AbstractProcessBuilder<Self> {
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
        let process = Process::this();
        ProcessRef { process }
    }
}

pub trait MessageHandler<Message>: AbstractProcess
where
    Self::Serializer: serializer::Serializer<Message>,
{
    fn handle(state: State<Self>, message: Message);
}

pub trait RequestHandler<Request>: AbstractProcess
where
    Self::Serializer: serializer::Serializer<Request>,
    Self::Serializer: serializer::Serializer<Self::Response>,
{
    type Response;

    fn handle(state: State<Self>, request: Request) -> Self::Response;
}

pub trait DeferredRequestHandler<Request>: AbstractProcess
where
    Self::Serializer: serializer::Serializer<Request>,
    Self::Serializer: serializer::Serializer<Self::Response>,
{
    type Response;

    fn handle(
        state: State<Self>,
        request: Request,
        deferred_response: DeferredResponse<Self::Response, Self::Serializer>,
    );
}

/// A reference to the state inside [`AbstractProcess`] handlers.
pub struct State<'a, AP: AbstractProcess> {
    state: &'a mut AP::State,
}

impl<'a, AP: AbstractProcess> State<'a, AP> {
    /// Get a reference to the running [`AbstractProcess`].
    pub fn self_ref(&self) -> ProcessRef<AP> {
        let process = Process::this();
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
pub struct DeferredResponse<Response, Serializer> {
    tag: Tag,
    return_address: ReturnAddress<Response, Serializer>,
}

impl<Response, Serializer> DeferredResponse<Response, Serializer>
where
    Serializer: serializer::Serializer<Response>,
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
#[derive(Copy, serde::Serialize, serde::Deserialize)]
#[serde(bound = "")]
pub struct ProcessRef<T>
where
    T: AbstractProcess,
{
    // The `Process` generic value is set to `()` here. For every request, the value is going to
    // be different and will be transmuted just in time before the request is sent out.
    process: Process<(), T::Serializer>,
}

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

    /// Returns the node ID .
    pub fn node_id(&self) -> u64 {
        self.process.node_id()
    }

    /// Returns a process registered under `name` if it exists and the signature
    /// matches.
    pub fn lookup<S: AsRef<str>>(name: S) -> Option<Self> {
        let name: &str = name.as_ref();
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

    /// Registers process under `name`.
    pub fn register<S: AsRef<str>>(&self, name: S) {
        let name: &str = name.as_ref();
        let name = format!("{} + ProcessRef + {}", name, std::any::type_name::<T>());
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
    ///
    /// If a timeout is specified the function will only block for the timeout
    /// period before returning `Err(Timeout)`.
    pub fn shutdown(&self, timeout: Option<Duration>) -> Result<(), Timeout>
    where
        // The serializer needs to be able to serialize values of `ShutdownMessage` & `()` for the
        // return value.
        T::Serializer: serializer::Serializer<ShutdownMessage<(), T::Serializer>>,
        T::Serializer: serializer::Serializer<()>,
    {
        let return_address = ReturnAddress::from_self();
        let message = ShutdownMessage(return_address);
        let send_tag = AbstractProcessTag::from_u6(SHUTDOWN_HANDLER);
        let (receive_tag, _) = AbstractProcessTag::extract_u6_data(send_tag);
        unsafe {
            // Cast into the right type for sending.
            let process: Process<ShutdownMessage<(), T::Serializer>, T::Serializer> =
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
        T::Serializer: serializer::Serializer<M>,
    {
        let handler_id = T::Handlers::handler_id::<Message<M>>();
        let tag = AbstractProcessTag::from_u6(handler_id);
        // Cast into the right type for sending.
        let process: Process<M, T::Serializer> = unsafe { std::mem::transmute(self.process) };
        process.tag_send(tag, message);
    }

    /// Make a request to the process.
    //
    /// If a timeout is specified the function will only block for the timeout
    /// period before returning `Err(Timeout)`.
    #[track_caller]
    pub fn request<R: 'static>(
        &self,
        request: R,
        timeout: Option<Duration>,
    ) -> Result<T::Response, Timeout>
    where
        T: RequestHandler<R>,
        T::Serializer: serializer::Serializer<R>,
        T::Serializer: serializer::Serializer<T::Response>,
        T::Serializer: serializer::Serializer<RequestMessage<R, T::Response, T::Serializer>>,
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
    //
    /// If a timeout is specified the function will only block for the timeout
    /// period before returning `Err(Timeout)`.
    #[track_caller]
    pub fn deferred_request<R: 'static>(
        &self,
        request: R,
        timeout: Option<Duration>,
    ) -> Result<T::Response, Timeout>
    where
        T: DeferredRequestHandler<R>,
        T::Serializer: serializer::Serializer<R>,
        T::Serializer: serializer::Serializer<T::Response>,
        T::Serializer: serializer::Serializer<RequestMessage<R, T::Response, T::Serializer>>,
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

/// Error result of [`ProcessRef::shutdown`] & [`ProcessRef::request`].
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Timeout;
