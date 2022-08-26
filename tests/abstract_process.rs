use std::time::Duration;

use lunatic::{
    host,
    process::{
        AbstractProcess, Message, MessageHandler, ProcessRef, Request, RequestHandler,
        SelfReference, StartProcess,
    },
    sleep, spawn_link, test, Tag,
};

#[test]
fn shutdown() {
    struct A;

    impl AbstractProcess for A {
        type Arg = ();
        type State = A;

        fn init(_: ProcessRef<Self>, _: ()) -> A {
            A
        }

        fn terminate(_: Self::State) {
            println!("Exiting");
        }
    }

    let a = A::start_link((), None);
    a.shutdown();
}

#[test]
fn handle_link_trapped() {
    struct A {
        link_trapped: bool,
    }

    impl AbstractProcess for A {
        type Arg = ();
        type State = A;

        fn init(_process: ProcessRef<Self>, _arg: ()) -> Self {
            unsafe { host::api::process::die_when_link_dies(0) };
            spawn_link!(|| panic!());
            Self {
                link_trapped: false,
            }
        }

        fn handle_link_trapped(state: &mut Self::State, tag: Tag) {
            println!("Link trapped: {:?}", tag);
            state.link_trapped = true;
        }
    }

    #[derive(serde::Serialize, serde::Deserialize)]
    struct IsLinkTrapped;
    impl RequestHandler<IsLinkTrapped> for A {
        type Response = bool;

        fn handle(state: &mut Self::State, _request: IsLinkTrapped) -> bool {
            state.link_trapped
        }
    }

    let a = A::start((), None);
    sleep(Duration::from_millis(10));
    assert!(a.request(IsLinkTrapped));
}

#[test]
fn handle_message() {
    struct A;

    impl AbstractProcess for A {
        type Arg = ();
        type State = A;

        fn init(_: ProcessRef<Self>, _: ()) -> A {
            A
        }
    }

    impl MessageHandler<String> for A {
        fn handle(_state: &mut Self::State, message: String) {
            assert_eq!(message, "Hello world");
        }
    }

    let a = A::start_link((), None);
    a.send("Hello world".to_owned());

    sleep(Duration::from_millis(100));
}

#[test]
fn handle_request() {
    struct A;

    impl AbstractProcess for A {
        type Arg = ();
        type State = A;

        fn init(_: ProcessRef<Self>, _: ()) -> A {
            A
        }
    }

    impl RequestHandler<String> for A {
        type Response = String;

        fn handle(_state: &mut Self::State, mut request: String) -> String {
            request.push_str(" world");
            request
        }
    }

    let a = A::start_link((), None);
    let response = a.request("Hello".to_owned());

    assert_eq!(response, "Hello world");
}

#[test]
fn init_args() {
    struct A(Vec<f64>);

    impl AbstractProcess for A {
        type Arg = Vec<f64>;
        type State = A;

        fn init(_: ProcessRef<Self>, arg: Vec<f64>) -> A {
            A(arg)
        }
    }

    #[derive(serde::Serialize, serde::Deserialize)]
    struct Add(f64);
    impl MessageHandler<Add> for A {
        fn handle(state: &mut Self::State, message: Add) {
            state.0.push(message.0);
        }
    }

    #[derive(serde::Serialize, serde::Deserialize)]
    struct Sum;
    impl RequestHandler<Sum> for A {
        type Response = f64;

        fn handle(state: &mut Self::State, _: Sum) -> f64 {
            state.0.iter().sum()
        }
    }

    let init = vec![0.1, 0.1, 0.1, 0.2];
    let a = A::start_link(init, None);

    a.send(Add(0.2));
    a.send(Add(0.2));
    a.send(Add(0.1));
    a.send(Add(1.0));
    assert_eq!(a.request(Sum), 2.0);
    a.send(Add(0.1));
    assert_eq!(a.request(Sum), 2.1);
    a.send(Add(0.1));
    assert_eq!(a.request(Sum), 2.2);
    a.send(Add(0.3));
    assert_eq!(a.request(Sum), 2.5);
    a.send(Add(0.1));
    assert_eq!(a.request(Sum), 2.6);
}

#[test]
fn self_reference() {
    struct A(u32);

    impl AbstractProcess for A {
        type Arg = u32;
        type State = A;

        fn init(this: ProcessRef<Self>, start: u32) -> A {
            // Start incrementing state state.
            this.send(Inc);

            A(start)
        }
    }

    #[derive(serde::Serialize, serde::Deserialize)]
    struct Inc;
    impl MessageHandler<Inc> for A {
        fn handle(state: &mut Self::State, _: Inc) {
            // Increment state until 10
            if state.0 < 10 {
                state.0 += 1;
                // Increment state again
                state.process().send(Inc);
            }
        }
    }

    #[derive(serde::Serialize, serde::Deserialize)]
    struct Count;
    impl RequestHandler<Count> for A {
        type Response = u32;

        fn handle(state: &mut Self::State, _: Count) -> u32 {
            state.0
        }
    }

    let a = A::start_link(0, None);
    // Give enough time to increment state.
    sleep(Duration::from_millis(20));

    assert_eq!(a.request(Count), 10);
}

#[test]
fn different_state_type() {
    struct A(u32);
    struct B;

    impl AbstractProcess for B {
        type Arg = u32;
        type State = A;

        fn init(_: ProcessRef<Self>, start: u32) -> A {
            A(start)
        }
    }

    #[derive(serde::Serialize, serde::Deserialize)]
    struct Inc;
    impl MessageHandler<Inc> for B {
        fn handle(state: &mut Self::State, _: Inc) {
            state.0 += 1;
        }
    }

    #[derive(serde::Serialize, serde::Deserialize)]
    struct Count;
    impl RequestHandler<Count> for B {
        type Response = u32;

        fn handle(state: &mut Self::State, _: Count) -> u32 {
            state.0
        }
    }

    let b = B::start_link(0, None);

    for i in 0..100 {
        assert_eq!(b.request(Count), i);
        b.send(Inc);
    }
}

#[test]
fn lookup() {
    struct A;

    impl AbstractProcess for A {
        type Arg = ();
        type State = A;

        fn init(_: ProcessRef<Self>, _: ()) -> A {
            A
        }
    }

    impl RequestHandler<()> for A {
        type Response = ();
        fn handle(_: &mut Self::State, _: ()) {}
    }

    let a = A::start_link((), Some("a"));
    drop(a);

    let a = ProcessRef::<A>::lookup("a").unwrap();
    a.request(());
    drop(a);

    let a = ProcessRef::<A>::lookup("b");
    assert!(a.is_none());
}

#[test]
#[should_panic]
fn linked_process_fails() {
    struct A;

    impl AbstractProcess for A {
        type Arg = ();
        type State = A;

        fn init(_: ProcessRef<Self>, _: ()) -> A {
            A
        }
    }

    #[derive(serde::Serialize, serde::Deserialize)]
    struct Panic;
    impl MessageHandler<Panic> for A {
        fn handle(_state: &mut Self::State, _: Panic) {
            panic!();
        }
    }

    let a = A::start_link((), None);
    a.link();
    a.send(Panic);
    sleep(Duration::from_millis(100));
}

#[test]
fn unlinked_process_doesnt_fail() {
    struct A;

    impl AbstractProcess for A {
        type Arg = ();
        type State = A;

        fn init(_: ProcessRef<Self>, _: ()) -> A {
            A
        }
    }

    #[derive(serde::Serialize, serde::Deserialize)]
    struct Panic;
    impl MessageHandler<Panic> for A {
        fn handle(_state: &mut Self::State, _: Panic) {
            panic!();
        }
    }

    let a = A::start_link((), None);
    a.link();
    a.unlink();
    a.send(Panic);
    sleep(Duration::from_millis(100));
}

#[test]
fn request_timeout() {
    struct A;

    impl AbstractProcess for A {
        type Arg = ();
        type State = A;

        fn init(_: ProcessRef<Self>, _: ()) -> A {
            A
        }
    }

    impl RequestHandler<String> for A {
        type Response = String;

        fn handle(_state: &mut Self::State, mut request: String) -> String {
            sleep(Duration::from_millis(25));
            request.push_str(" world");
            request
        }
    }

    let a = A::start_link((), None);
    let response = a.request_timeout("Hello".to_owned(), Duration::from_millis(10));

    assert!(response.is_timed_out());
}

#[test]
fn shutdown_timeout() {
    struct A;

    impl AbstractProcess for A {
        type Arg = ();
        type State = A;

        fn init(_: ProcessRef<Self>, _: ()) -> A {
            A
        }

        fn terminate(_: Self::State) {
            sleep(Duration::from_millis(25));
        }
    }

    let a = A::start_link((), None);
    let response = a.shutdown_timeout(Duration::from_millis(10));

    assert!(response.is_timed_out());
}
