use lunatic::process::SelfReference;
use lunatic::{
    process::{Message, ProcessRef, Request, StartProcess},
    sleep, test, Tag,
};
use lunatic_macros::{abstract_process, process_message, process_request};
use std::time::Duration;

#[derive(Debug)]
struct Counter {
    count: u32,
}

#[derive(serde::Serialize, serde::Deserialize)]
struct Panic;
#[derive(serde::Serialize, serde::Deserialize)]
struct Inc;
#[derive(serde::Serialize, serde::Deserialize)]
struct Count;
#[derive(serde::Serialize, serde::Deserialize, Debug)]
struct CountReply(u32);

#[abstract_process]
impl Counter {
    #[init]
    fn init(_process: ProcessRef<Self>, count: u32) -> Self {
        Self { count }
    }

    #[terminate]
    fn terminate(self) {
        println!("Shutting down with state {}", self.count);
    }

    #[handle_link_trapped]
    fn handle_link_trapped(&mut self, tag: Tag) {
        println!("{:?}", tag);
    }

    #[process_message]
    fn increment(&mut self, _: Inc) {
        self.count += 1;
        self.check_count();
    }

    #[process_message]
    fn panic(&mut self, _: Panic) {
        panic!();
    }

    #[process_request]
    fn count(&self, _: Count) -> CountReply {
        CountReply(self.count)
    }

    fn check_count(&self) {
        if self.count > 5 {
            println!("count exceeded 5!");
        }
    }
}

#[test]
fn init_and_terminate() {
    let counter = Counter::start_link(2, None);
    counter.increment(Inc);
    counter.increment(Inc);
    counter.increment(Inc);
    counter.increment(Inc);
    counter.increment(Inc);
    dbg!(counter.count(Count));
    counter.panic(Panic);
}

#[test]
fn shutdown() {
    struct A;

    #[abstract_process]
    impl A {
        #[init]
        fn init(_: ProcessRef<Self>, _: ()) -> A {
            A
        }

        #[terminate]
        fn terminate(self) {
            println!("Exiting");
        }
    }

    let a = A::start_link((), None);
    a.shutdown();
}

#[test]
fn handle_message() {
    struct A;

    #[abstract_process]
    impl A {
        #[init]
        fn init(_: ProcessRef<Self>, _: ()) -> A {
            A
        }

        #[process_message]
        fn handle(&self, message: String) {
            assert_eq!(message, "Hello world");
        }
    }

    let a = A::start_link((), None);
    a.handle("Hello world".to_owned());

    sleep(Duration::from_millis(100));
}

#[test]
fn handle_request() {
    struct A;

    #[abstract_process]
    impl A {
        #[init]
        fn init(_: ProcessRef<Self>, _: ()) -> A {
            A
        }

        #[process_request]
        fn handle(&mut self, mut request: String) -> String {
            request.push_str(" world");
            request
        }
    }

    let a = A::start_link((), None);
    let response = a.handle("Hello".to_owned());

    assert_eq!(response, "Hello world");
}

#[test]
fn init_args() {
    struct A(Vec<f64>);

    #[derive(serde::Serialize, serde::Deserialize)]
    struct Sum;

    #[abstract_process]
    impl A {
        #[init]
        fn init(_: ProcessRef<Self>, arg: Vec<f64>) -> A {
            A(arg)
        }

        #[process_message]
        fn add(&mut self, message: f64) {
            self.0.push(message);
        }

        #[process_request]
        fn sum(&mut self, _: Sum) -> f64 {
            self.0.iter().sum()
        }
    }

    let init = vec![0.1, 0.1, 0.1, 0.2];
    let a = A::start_link(init, None);

    a.add(0.2);
    a.add(0.2);
    a.add(0.1);
    a.add(1.0);
    assert_eq!(a.sum(Sum), 2.0);
    a.add(0.1);
    assert_eq!(a.sum(Sum), 2.1);
    a.add(0.1);
    assert_eq!(a.sum(Sum), 2.2);
    a.add(0.3);
    assert_eq!(a.sum(Sum), 2.5);
    a.add(0.1);
    assert_eq!(a.sum(Sum), 2.6);
}

#[test]
fn self_reference() {
    struct A(u32);

    #[derive(serde::Serialize, serde::Deserialize)]
    struct Inc;
    #[derive(serde::Serialize, serde::Deserialize)]
    struct Count;

    #[abstract_process]
    impl A {
        #[init]
        fn init(this: ProcessRef<Self>, start: u32) -> A {
            // Start incrementing state state.
            this.increment(Inc);

            A(start)
        }

        #[process_message]
        fn increment(&mut self, _: Inc) {
            // Increment state until 10
            if self.0 < 10 {
                self.0 += 1;
                // Increment state again
                self.process().increment(Inc);
            }
        }

        #[process_request]
        fn count(&self, _: Count) -> u32 {
            self.0
        }
    }

    let a = A::start_link(0, None);
    // Give enough time to increment state.
    sleep(Duration::from_millis(20));

    assert_eq!(a.count(Count), 10);
}

#[test]
fn lookup() {
    struct A;

    #[abstract_process]
    impl A {
        #[init]
        fn init(_: ProcessRef<Self>, _: ()) -> A {
            A
        }

        #[process_request]
        fn handle(&mut self, _: ()) {}
    }

    let a = A::start_link((), Some("a"));
    drop(a);

    let a = ProcessRef::<A>::lookup("a").unwrap();
    a.handle(());
    drop(a);

    let a = ProcessRef::<A>::lookup("b");
    assert!(a.is_none());
}

#[test]
#[should_panic]
fn linked_process_fails() {
    struct A;

    #[derive(serde::Serialize, serde::Deserialize)]
    struct Panic;

    #[abstract_process]
    impl A {
        #[init]
        fn init(_: ProcessRef<Self>, _: ()) -> A {
            A
        }

        #[process_message]
        fn panic(&mut self, _: Panic) {
            panic!();
        }
    }

    let a = A::start_link((), None);
    a.link();
    a.panic(Panic);
    sleep(Duration::from_millis(100));
}

#[test]
fn unlinked_process_doesnt_fail() {
    struct A;

    #[derive(serde::Serialize, serde::Deserialize)]
    struct Panic;

    #[abstract_process]
    impl A {
        #[init]
        fn init(_: ProcessRef<Self>, _: ()) -> A {
            A
        }

        #[process_message]
        fn panic(&mut self, _: Panic) {
            panic!();
        }
    }

    let a = A::start_link((), None);
    a.link();
    a.unlink();
    a.panic(Panic);
    sleep(Duration::from_millis(100));
}

#[test]
fn request_timeout() {
    struct A;

    #[abstract_process]
    impl A {
        #[init]
        fn init(_: ProcessRef<Self>, _: ()) -> A {
            A
        }

        #[process_request]
        fn handle(&mut self, mut request: String) -> String {
            sleep(Duration::from_millis(25));
            request.push_str(" world");
            request
        }
    }

    let a = A::start_link((), None);
    let response = a.request_timeout("Hello".to_owned(), Duration::from_millis(10));

    assert!(response.is_err());
}

#[test]
fn shutdown_timeout() {
    struct A;

    #[abstract_process]
    impl A {
        #[init]
        fn init(_: ProcessRef<Self>, _: ()) -> A {
            A
        }

        #[terminate]
        fn terminate(self) {
            sleep(Duration::from_millis(25));
        }
    }

    let a = A::start_link((), None);
    let response = a.shutdown_timeout(Duration::from_millis(10));

    assert!(response.is_err());
}
