use std::time::Duration;

use lunatic::ap::handlers::{DeferredRequest, Message, Request};
use lunatic::ap::{
    AbstractProcess, Config, DeferredRequestHandler, DeferredResponse, MessageHandler, ProcessRef,
    RequestHandler, StartupError, State,
};
use lunatic::serializer::Bincode;
use lunatic::time::Timeout;
use lunatic::{sleep, spawn_link, test};

/// This `AbstractProcess` always panics on `init`.
struct InitPanicksAP;

impl AbstractProcess for InitPanicksAP {
    type State = ();
    type Serializer = Bincode;
    type Arg = ();
    type Handlers = ();
    type StartupError = ();

    fn init(_: Config<Self>, _: Self::Arg) -> Result<(), ()> {
        panic!("Startup failed");
    }
}

#[test]
fn init_failure() {
    assert_eq!(InitPanicksAP::start(()), Err(StartupError::InitPanicked));
}

/// This `AbstractProcess` returns an error on `init`.
struct InitErrorAP;

impl AbstractProcess for InitErrorAP {
    type State = ();
    type Serializer = Bincode;
    type Arg = ();
    type Handlers = ();
    type StartupError = String;

    fn init(_: Config<Self>, _: Self::Arg) -> Result<(), String> {
        Err("Failed".to_owned())
    }
}

#[test]
fn init_error() {
    assert_eq!(
        InitErrorAP::start(()),
        Err(StartupError::Custom("Failed".to_owned()))
    );
}

/// `AbstractProcess` that starts normally.
struct InitOkAP;

impl AbstractProcess for InitOkAP {
    type State = ();
    type Serializer = Bincode;
    type Arg = ();
    type Handlers = ();
    type StartupError = ();

    fn init(_: Config<Self>, _: Self::Arg) -> Result<(), ()> {
        Ok(())
    }
}

#[test]
fn init_ok() {
    assert!(InitOkAP::start(()).is_ok());
}

#[test]
fn shutdown_ok() {
    let ap = InitOkAP::start(()).unwrap();
    ap.shutdown();
}

/// `AbstractProcess` that fails to shut down in time.
struct ShutdownTimeoutAP;

impl AbstractProcess for ShutdownTimeoutAP {
    type State = ();
    type Serializer = Bincode;
    type Arg = ();
    type Handlers = ();
    type StartupError = ();

    fn init(_: Config<Self>, _: Self::Arg) -> Result<(), ()> {
        Ok(())
    }

    fn terminate(_state: Self::State) {
        sleep(Duration::from_millis(100));
    }
}

#[test]
fn shutdown_timeout() {
    let ap = ShutdownTimeoutAP::start(()).unwrap();
    assert!(ap
        .with_timeout(Duration::from_millis(10))
        .shutdown()
        .is_err());
}

/// `AbstractProcess` with float array as `init` arguments.
struct FloatsServerAP(Vec<f64>);

impl AbstractProcess for FloatsServerAP {
    type State = Self;
    type Serializer = Bincode;
    type Arg = Vec<f64>;
    type Handlers = (Message<Add>, Request<Sum>);
    type StartupError = ();

    fn init(_: Config<Self>, arg: Self::Arg) -> Result<Self, ()> {
        Ok(Self(arg))
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
struct Add(f64);
impl MessageHandler<Add> for FloatsServerAP {
    fn handle(mut state: State<Self>, add: Add) {
        state.0.push(add.0);
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
struct Sum;
impl RequestHandler<Sum> for FloatsServerAP {
    type Response = f64;

    fn handle(state: State<Self>, _: Sum) -> Self::Response {
        state.0.iter().sum()
    }
}

#[test]
fn float_message_and_request_handling() {
    let init = vec![0.1, 0.1, 0.1, 0.2];
    let ap = FloatsServerAP::link().start(init).unwrap();

    ap.send(Add(0.2));
    ap.send(Add(0.2));
    ap.send(Add(0.1));
    ap.send(Add(1.0));
    assert_eq!(ap.request(Sum), 2.0);
    ap.send(Add(0.1));
    assert_eq!(ap.request(Sum), 2.1);
    ap.send(Add(0.1));
    assert_eq!(ap.request(Sum), 2.2);
    ap.send(Add(0.3));
    assert_eq!(ap.request(Sum), 2.5);
    ap.send(Add(0.1));
    assert_eq!(ap.request(Sum), 2.6);
}

/// `AbstractProcess` that self-references itself during `init` and in handlers.
struct SelfRefAP(u32);

impl AbstractProcess for SelfRefAP {
    type State = Self;
    type Serializer = Bincode;
    type Arg = u32;
    type Handlers = (Message<Inc>, Request<Count>);
    type StartupError = ();

    fn init(config: Config<Self>, start: Self::Arg) -> Result<Self, ()> {
        // Send increment message before constructing state.
        config.self_ref().send(Inc);
        Ok(Self(start))
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
struct Inc;
impl MessageHandler<Inc> for SelfRefAP {
    fn handle(mut state: State<Self>, _: Inc) {
        // Increment state until 10
        if state.0 < 10 {
            state.0 += 1;
            // Increment state again
            state.self_ref().send(Inc);
        }
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
struct Count;
impl RequestHandler<Count> for SelfRefAP {
    type Response = u32;

    fn handle(state: State<Self>, _: Count) -> Self::Response {
        state.0
    }
}

#[test]
fn self_ref() {
    let ap = SelfRefAP::link().start(0).unwrap();
    // Give enough time to increment state.
    sleep(Duration::from_millis(20));
    assert_eq!(ap.request(Count), 10);
}

/// `AbstractProcess` that is registered under a well-known name.
struct RegisteredAP;

impl AbstractProcess for RegisteredAP {
    type State = ();
    type Serializer = Bincode;
    type Arg = ();
    type Handlers = ();
    type StartupError = ();

    fn init(_: Config<Self>, _: Self::Arg) -> Result<(), ()> {
        // Doing a lookup in `init` should not deadlock.
        let _ = ProcessRef::<InitOkAP>::lookup("_");
        Ok(())
    }
}

#[test]
fn lookup() {
    let ap = RegisteredAP::start_as("AP", ()).unwrap();
    let lookup = ProcessRef::<RegisteredAP>::lookup("AP").unwrap();
    assert_eq!(ap, lookup);
    let exists = RegisteredAP::start_as("AP", ());
    assert_eq!(exists, Err(StartupError::NameAlreadyRegistered(ap)));
    // Registering a different process type under the same name will work.
    let doesnt_exist = InitOkAP::start_as("AP", ());
    assert!(doesnt_exist.is_ok());
}

/// `AbstractProcess` that can panic on message.
struct PanicOnMessageAP;

impl AbstractProcess for PanicOnMessageAP {
    type State = ();
    type Serializer = Bincode;
    type Arg = ();
    type Handlers = (Message<Panick>,);
    type StartupError = ();

    fn init(_: Config<Self>, _: Self::Arg) -> Result<(), ()> {
        Ok(())
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
struct Panick;

impl MessageHandler<Panick> for PanicOnMessageAP {
    fn handle(_: State<Self>, _: Panick) {
        panic!();
    }
}

#[test]
#[should_panic]
fn linked_process_fails() {
    let ap = PanicOnMessageAP::start(()).unwrap();
    ap.link();
    ap.send(Panick);
    sleep(Duration::from_millis(10));
}

#[test]
fn unlinked_process_doesnt_fail() {
    let ap = PanicOnMessageAP::link().start(()).unwrap();
    ap.unlink();
    ap.send(Panick);
    sleep(Duration::from_millis(10));
}

/// `AbstractProcess` that handles failed links
struct HandleLinkPanicAP {
    panicked: bool,
}

impl AbstractProcess for HandleLinkPanicAP {
    type State = Self;
    type Serializer = Bincode;
    type Arg = ();
    type Handlers = (Request<DidPanick>,);
    type StartupError = ();

    fn init(config: Config<Self>, _: Self::Arg) -> Result<Self, ()> {
        config.die_if_link_dies(false);
        spawn_link!(|| panic!());
        Ok(Self { panicked: false })
    }

    fn handle_link_death(mut state: State<Self>, tag: lunatic::Tag) {
        println!("Link trapped: {:?}", tag);
        state.panicked = true;
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
struct DidPanick;

impl RequestHandler<DidPanick> for HandleLinkPanicAP {
    type Response = bool;

    fn handle(state: State<Self>, _: DidPanick) -> Self::Response {
        state.panicked
    }
}

#[test]
fn handle_link_panic() {
    let ap = HandleLinkPanicAP::start(()).unwrap();
    sleep(Duration::from_millis(10));
    assert!(ap.request(DidPanick));
}

/// `AbstractProcess` that handles `String` message
struct StringHandlerAP;

impl AbstractProcess for StringHandlerAP {
    type State = Self;
    type Serializer = Bincode;
    type Arg = ();
    type Handlers = (Message<String>,);
    type StartupError = ();

    fn init(_: Config<Self>, _: Self::Arg) -> Result<Self, ()> {
        Ok(Self)
    }
}

impl MessageHandler<String> for StringHandlerAP {
    fn handle(_: State<Self>, message: String) {
        println!("what");
        assert_eq!(message, "Hello process");
    }
}

#[test]
fn handle_message() {
    let ap = StringHandlerAP::link().start(()).unwrap();
    ap.send("Hello process".to_owned());
    sleep(Duration::from_millis(10));
}

/// `AbstractProcess` that handles a `String` request/response
struct StringRequestHandlerAP;

impl AbstractProcess for StringRequestHandlerAP {
    type State = Self;
    type Serializer = Bincode;
    type Arg = ();
    type Handlers = (Request<String>,);
    type StartupError = ();

    fn init(_: Config<Self>, _: Self::Arg) -> Result<Self, ()> {
        Ok(Self)
    }
}

impl RequestHandler<String> for StringRequestHandlerAP {
    type Response = String;

    fn handle(_: State<Self>, mut request: String) -> Self::Response {
        request.push_str(" world");
        request
    }
}

#[test]
fn handle_request() {
    let ap = StringRequestHandlerAP::link().start(()).unwrap();
    let response = ap.request("Hello".to_owned());
    assert_eq!(response, "Hello world");
}

/// `AbstractProcess` that times out on requests
struct RequestHandlerTimeoutAP;

impl AbstractProcess for RequestHandlerTimeoutAP {
    type State = Self;
    type Serializer = Bincode;
    type Arg = ();
    type Handlers = (Request<()>,);
    type StartupError = ();

    fn init(_: Config<Self>, _: Self::Arg) -> Result<Self, ()> {
        Ok(Self)
    }
}

impl RequestHandler<()> for RequestHandlerTimeoutAP {
    type Response = ();

    fn handle(_: State<Self>, _: ()) -> Self::Response {
        sleep(Duration::from_millis(25));
    }
}

#[test]
fn request_timeout() {
    let ap = RequestHandlerTimeoutAP::link().start(()).unwrap();
    let response = ap.with_timeout(Duration::from_millis(10)).request(());
    assert_eq!(response, Err(Timeout));
}

/// `AbstractProcess` that handles a deferred `String` request/response
struct DeferredStringRequestHandlerAP;

impl AbstractProcess for DeferredStringRequestHandlerAP {
    type State = Self;
    type Serializer = Bincode;
    type Arg = ();
    type Handlers = (DeferredRequest<String>,);
    type StartupError = ();

    fn init(_: Config<Self>, _: Self::Arg) -> Result<Self, ()> {
        Ok(Self)
    }
}

impl DeferredRequestHandler<String> for DeferredStringRequestHandlerAP {
    type Response = String;

    fn handle(
        _: State<Self>,
        request: String,
        deferred_response: DeferredResponse<Self::Response, Self>,
    ) {
        spawn_link!(|request, deferred_response| {
            request.push_str(" world");
            deferred_response.send_response(request);
        });
    }
}

#[test]
fn deferred_handle_request() {
    let ap = DeferredStringRequestHandlerAP::link().start(()).unwrap();
    let response = ap.deferred_request("Hello".to_owned());
    assert_eq!(response, "Hello world");
}

/// `AbstractProcess` that times out on a deferred request/response
struct DeferredRequestTimeoutAP;

impl AbstractProcess for DeferredRequestTimeoutAP {
    type State = Self;
    type Serializer = Bincode;
    type Arg = ();
    type Handlers = (DeferredRequest<String>,);
    type StartupError = ();

    fn init(_: Config<Self>, _: Self::Arg) -> Result<Self, ()> {
        Ok(Self)
    }
}

impl DeferredRequestHandler<String> for DeferredRequestTimeoutAP {
    type Response = String;

    fn handle(_: State<Self>, _: String, _: DeferredResponse<Self::Response, Self>) {
        // Never return response
    }
}

#[test]
fn deferred_request_timeout() {
    let ap = DeferredRequestTimeoutAP::link().start(()).unwrap();
    let response = ap
        .with_timeout(Duration::from_millis(10))
        .deferred_request("Hello".to_owned());
    assert_eq!(response, Err(Timeout));
}
