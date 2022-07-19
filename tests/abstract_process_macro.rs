use lunatic::{
    host,
    process::{Message, ProcessRef, Request, StartProcess},
    sleep, spawn_link, test, Tag,
};
use lunatic_macros::{abstract_process, process_message, process_request};
use std::time::Duration;

#[test]
fn init() {
    struct A;

    #[abstract_process]
    impl A {
        #[init]
        fn init(_process: ProcessRef<Self>, _count: (u32, String)) -> Self {
            Self {}
        }
    }

    A::start_link(
        (
            42,
            "the meaning of life, the universe, and everything".to_owned(),
        ),
        None,
    );
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
fn handle_link_trapped() {
    struct A {
        link_trapped: bool,
    }

    #[abstract_process]
    impl A {
        #[init]
        fn init(_process: ProcessRef<Self>, _arg: ()) -> Self {
            unsafe { host::api::process::die_when_link_dies(0) };
            spawn_link!(|| panic!());
            Self {
                link_trapped: false,
            }
        }

        #[handle_link_trapped]
        fn handle_link_trapped(&mut self, tag: Tag) {
            println!("Link trapped: {:?}", tag);
            self.link_trapped = true;
        }

        #[process_request]
        fn is_link_trapped(&self) -> bool {
            self.link_trapped
        }
    }

    let a = A::start((), None);
    sleep(Duration::from_millis(10));
    assert!(a.is_link_trapped());
}

#[test]
fn handle_zero_argument() {
    struct Counter {
        count: u32,
    }

    #[abstract_process]
    impl Counter {
        #[init]
        fn init(_process: ProcessRef<Self>, count: u32) -> Self {
            Self { count }
        }

        #[process_message]
        fn increment(&mut self, num: u32) {
            self.count += num;
            self.check_count();
        }

        #[process_message]
        fn decrement(&mut self, num: u32) {
            self.count -= num;
            self.check_count();
        }

        #[process_request]
        fn count(&self) -> u32 {
            self.count
        }

        fn check_count(&self) {
            if self.count > 5 {
                println!("count exceeded 5!");
            }
        }
    }

    let counter = Counter::start_link(2, None);
    counter.increment(1);
    assert_eq!(3, counter.count());
    counter.decrement(3);
    assert_eq!(0, counter.count());
}

#[test]
fn handle_single_argument() {
    struct A;

    #[derive(serde::Serialize, serde::Deserialize)]
    struct Name(String);

    #[abstract_process]
    impl A {
        #[init]
        fn init(_: ProcessRef<Self>, _: ()) -> A {
            A
        }

        #[process_message]
        fn say_hello(&self, message: String) {
            assert_eq!(message, "Hello");
        }

        #[process_request]
        fn say_hello_to(&self, name: Name) -> String {
            format!("Hello {}", name.0)
        }
    }

    let a = A::start_link((), None);
    a.say_hello("Hello".to_owned());
    let greeting = a.say_hello_to(Name("Mark".to_owned()));
    assert_eq!("Hello Mark", greeting);
}

#[test]
fn handle_more_than_one_argument() {
    struct A;

    #[derive(serde::Serialize, serde::Deserialize)]
    struct Num(u32);

    #[abstract_process]
    impl A {
        #[init]
        fn init(_: ProcessRef<Self>, _: ()) -> A {
            A
        }

        #[process_message]
        fn say_hello(&self, arg1: String, arg2: bool, arg3: Num) {
            assert_eq!(arg1, "Hello");
            assert_eq!(arg2, false);
            assert_eq!(arg3.0, 666);
        }

        #[process_request]
        fn say_hello_to(&self, arg1: String, arg2: bool, arg3: Num) -> String {
            format!("{} {} {}", arg1, arg2, arg3.0)
        }
    }

    let a = A::start_link((), None);
    a.say_hello("Hello".to_owned(), false, Num(666));
    let greeting = a.say_hello_to("Mark".to_owned(), true, Num(777));
    assert_eq!("Mark true 777", greeting);
}

#[test]
fn handle_mut_types() {
    struct A;

    #[abstract_process]
    impl A {
        #[init]
        fn init(_: ProcessRef<Self>, _: ()) -> A {
            A
        }

        #[process_message]
        fn one_mut_arg(&self, mut _a: String) {}

        #[process_request]
        fn two_mut_arg(&self, mut _a: String, _b: bool) -> () {}
    }

    let a = A::start_link((), None);
    a.one_mut_arg("Hello".to_owned());
    a.two_mut_arg("Hello".to_owned(), true);
}

#[test]
fn handle_destructuring() {
    struct A;

    #[derive(serde::Serialize, serde::Deserialize)]
    struct Person {
        name: String,
        age: u16,
    }

    #[abstract_process]
    impl A {
        #[init]
        fn init(_: ProcessRef<Self>, _: ()) -> A {
            A
        }

        #[process_message]
        fn unpack_tuples(&self, (a, (mut b, c)): (u8, (bool, char))) {
            assert_eq!(a, 5);
            b = !b;
            assert_eq!(b, true);
            assert_eq!(c, 'a');
        }

        #[process_message]
        fn unpack_slice(&self, [a, b, c]: [u32; 3]) {
            assert_eq!(a, 0);
            assert_eq!(b, 1);
            assert_eq!(c, 2);
        }

        #[process_request]
        fn unpack_struct(&self, Person { name, mut age }: Person) -> () {
            assert_eq!(name, "Mark");
            age += 1;
            assert_eq!(age, 5);
        }
    }

    let a = A::start_link((), None);
    a.unpack_tuples((5, (false, 'a')));
    a.unpack_slice([0, 1, 2]);
    a.unpack_struct(Person {
        name: "Mark".to_owned(),
        age: 4,
    });
}

#[test]
fn reply_types() {
    struct A;

    #[derive(Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
    struct CustomReply;

    #[abstract_process]
    impl A {
        #[init]
        fn init(_: ProcessRef<Self>, _: ()) -> A {
            A
        }

        #[process_request]
        fn empty_struct(&self) -> () {}

        #[process_request]
        fn builtin_type(&self) -> bool {
            true
        }

        #[process_request]
        fn nested_types(&self) -> (bool, u8) {
            (true, 9)
        }

        #[process_request]
        fn custom_type(&self) -> CustomReply {
            CustomReply
        }
    }

    let a = A::start_link((), None);
    assert_eq!(a.empty_struct(), ());
    assert_eq!(a.builtin_type(), true);
    assert_eq!(a.nested_types(), (true, 9));
    assert_eq!(a.custom_type(), CustomReply);
}
