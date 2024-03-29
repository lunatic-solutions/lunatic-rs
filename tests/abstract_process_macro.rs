use std::f32::consts::PI;
use std::time::Duration;

use lunatic::ap::{AbstractProcess, Config};
use lunatic::{abstract_process, host, sleep, spawn_link, test, Tag};

#[test]
fn init() {
    struct A;

    #[abstract_process]
    impl A {
        #[init]
        fn init(_config: Config<Self>, _count: (u32, String)) -> Result<Self, ()> {
            Ok(Self {})
        }
    }

    A::link()
        .start((
            42,
            "the meaning of life, the universe, and everything".to_owned(),
        ))
        .unwrap();
}

#[test]
fn shutdown() {
    struct A;

    #[abstract_process]
    impl A {
        #[init]
        fn init(_: Config<Self>, _: ()) -> Result<A, ()> {
            Ok(A)
        }

        #[terminate]
        fn terminate(self) {
            println!("Exiting");
        }
    }

    let a = A::link().start(()).unwrap();
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
        fn init(_config: Config<Self>, _arg: ()) -> Result<Self, ()> {
            unsafe { host::api::process::die_when_link_dies(0) };
            spawn_link!(|| panic!());
            Ok(Self {
                link_trapped: false,
            })
        }

        #[handle_link_death]
        fn handle_link_trapped(&mut self, tag: Tag) {
            println!("Link trapped: {:?}", tag);
            self.link_trapped = true;
        }

        #[handle_request]
        fn is_link_trapped(&self) -> bool {
            self.link_trapped
        }
    }

    let a = A::start(()).unwrap();
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
        fn init(_config: Config<Self>, count: u32) -> Result<Self, ()> {
            Ok(Self { count })
        }

        #[handle_message]
        fn increment(&mut self) {
            self.count += 1;
            self.check_count();
        }

        #[handle_message]
        fn decrement(&mut self) {
            self.count -= 1;
            self.check_count();
        }

        #[handle_request]
        fn count(&self) -> u32 {
            self.count
        }

        fn check_count(&self) {
            if self.count > 5 {
                println!("count exceeded 5!");
            }
        }
    }

    let counter = Counter::link().start(2).unwrap();
    counter.increment();
    assert_eq!(3, counter.count());
    counter.decrement();
    assert_eq!(2, counter.count());
}

#[test]
fn handle_single_argument() {
    struct A;

    #[derive(serde::Serialize, serde::Deserialize)]
    pub struct Name(String);

    #[abstract_process]
    impl A {
        #[init]
        fn init(_: Config<Self>, _: ()) -> Result<A, ()> {
            Ok(A)
        }

        #[handle_message]
        fn say_hello(&self, message: String) {
            assert_eq!(message, "Hello");
        }

        #[handle_request]
        fn say_hello_to(&self, name: Name) -> String {
            format!("Hello {}", name.0)
        }
    }

    let a = A::link().start(()).unwrap();
    a.say_hello("Hello".to_owned());
    let greeting = a.say_hello_to(Name("Mark".to_owned()));
    assert_eq!("Hello Mark", greeting);
}

#[test]
fn handle_multiple_arguments() {
    struct A;

    #[derive(serde::Serialize, serde::Deserialize)]
    pub struct Num(u32);

    #[abstract_process]
    impl A {
        #[init]
        fn init(_: Config<Self>, _: ()) -> Result<A, ()> {
            Ok(A)
        }

        #[handle_message]
        fn say_hello(&self, arg1: String, arg2: bool, arg3: Num) {
            assert_eq!(arg1, "Hello");
            assert_eq!(arg2, false);
            assert_eq!(arg3.0, 666);
        }

        #[handle_request]
        fn say_hello_to(&self, arg1: String, arg2: bool, arg3: Num) -> String {
            format!("{} {} {}", arg1, arg2, arg3.0)
        }
    }

    let a = A::link().start(()).unwrap();
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
        fn init(_: Config<Self>, _: ()) -> Result<A, ()> {
            Ok(A)
        }

        #[handle_message]
        fn one_mut_arg(&self, mut _a: String) {}

        #[handle_request]
        fn two_mut_arg(&self, mut _a: String, _b: bool) -> () {}
    }

    let a = A::link().start(()).unwrap();
    a.one_mut_arg("Hello".to_owned());
    a.two_mut_arg("Hello".to_owned(), true);
}

#[test]
fn handle_destructuring() {
    struct A;

    #[derive(serde::Serialize, serde::Deserialize)]
    pub struct Person {
        name: String,
        age: u16,
    }

    #[abstract_process]
    impl A {
        #[init]
        fn init(_: Config<Self>, _: ()) -> Result<A, ()> {
            Ok(A)
        }

        #[handle_message]
        fn unpack_tuples(&self, (a, (mut b, c)): (u8, (bool, char))) {
            assert_eq!(a, 5);
            b = !b;
            assert_eq!(b, true);
            assert_eq!(c, 'a');
        }

        #[handle_message]
        fn unpack_slice(&self, [a, b, c]: [u32; 3]) {
            assert_eq!(a, 0);
            assert_eq!(b, 1);
            assert_eq!(c, 2);
        }

        #[handle_request]
        fn unpack_struct(&self, Person { name, mut age }: Person) -> () {
            assert_eq!(name, "Mark");
            age += 1;
            assert_eq!(age, 5);
        }
    }

    let a = A::link().start(()).unwrap();
    a.unpack_tuples((5, (false, 'a')));
    a.unpack_slice([0, 1, 2]);
    a.unpack_struct(Person {
        name: "Mark".to_owned(),
        age: 4,
    });
}

#[test]
fn handle_comments() {
    struct Counter {
        count: u32,
    }

    /// Some comments on the counter.
    #[abstract_process]
    impl Counter {
        /// Some comments on the init method.
        #[init]
        fn init(_config: Config<Self>, count: u32) -> Result<Self, ()> {
            Ok(Self { count })
        }

        /// Some comments on the terminate method.
        #[terminate]
        fn terminate(self) {}

        /// Some comments on the handle_link_trapped method.
        #[handle_link_death]
        fn handle_link_trapped(&mut self, _tag: Tag) {}

        /// Some comments on the increment method.
        #[handle_message]
        fn increment(&mut self) {
            self.count += 1;
        }

        /// Some comments on the init method.
        #[handle_request]
        fn count(&self) -> u32 {
            self.count
        }
    }

    let counter = Counter::link().start(2).unwrap();
    counter.increment();
    assert_eq!(3, counter.count());
}

#[test]
fn handle_differing_names() {
    struct Counter {
        count: u32,
    }

    /// Some comments on the counter.
    #[abstract_process]
    impl Counter {
        /// Some comments on the init method.
        #[init]
        fn initialize(_config: Config<Self>, count: u32) -> Result<Self, ()> {
            Ok(Self { count })
        }

        /// Some comments on the terminate method.
        #[terminate]
        fn terminator(self) {}

        /// Some comments on the handle_link_trapped method.
        #[handle_link_death]
        fn link_trapped(&mut self, _tag: Tag) {}

        /// Some comments on the increment method.
        #[handle_message]
        fn increment(&mut self) {
            self.count += 1;
        }

        /// Some comments on the init method.
        #[handle_request]
        fn count(&self) -> u32 {
            self.count
        }
    }

    let counter = Counter::link().start(2).unwrap();
    counter.increment();
    assert_eq!(3, counter.count());
}

#[test]
fn reply_types() {
    struct A;

    #[derive(Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
    pub struct CustomReply;

    #[abstract_process]
    impl A {
        #[init]
        fn init(_: Config<Self>, _: ()) -> Result<A, ()> {
            Ok(A)
        }

        #[handle_request]
        fn empty_struct(&self) -> () {}

        #[handle_request]
        fn builtin_type(&self) -> bool {
            true
        }

        #[handle_request]
        fn nested_types(&self) -> (bool, u8) {
            (true, 9)
        }

        #[handle_request]
        fn custom_type(&self) -> CustomReply {
            CustomReply
        }
    }

    let a = A::link().start(()).unwrap();
    assert_eq!(a.empty_struct(), ());
    assert_eq!(a.builtin_type(), true);
    assert_eq!(a.nested_types(), (true, 9));
    assert_eq!(a.custom_type(), CustomReply);
}

#[test]
fn send_with_delay() {
    struct Counter {
        count: u32,
    }

    #[abstract_process]
    impl Counter {
        #[init]
        fn init(_config: Config<Self>, count: u32) -> Result<Self, ()> {
            Ok(Self { count })
        }

        #[handle_message]
        fn increment(&mut self) {
            self.count += 1;
        }

        #[handle_request]
        fn count(&self) -> u32 {
            self.count
        }
    }

    let counter = Counter::link().start(2).unwrap();
    counter.with_delay(Duration::from_millis(10)).increment();
    assert_eq!(2, counter.count());
    sleep(Duration::from_millis(15));
    assert_eq!(3, counter.count());
}

#[test]
fn request_timeout() {
    struct A;

    #[abstract_process]
    impl A {
        #[init]
        fn init(_config: Config<Self>, _: ()) -> Result<Self, ()> {
            Ok(Self)
        }

        #[handle_request]
        fn respond_fast(&self) -> u32 {
            sleep(Duration::from_millis(5));
            0
        }

        #[handle_request]
        fn respond_slow(&self) -> u32 {
            sleep(Duration::from_millis(15));
            0
        }
    }

    let counter = A::link().start(()).unwrap();
    assert!(counter
        .with_timeout(Duration::from_millis(10))
        .respond_fast()
        .is_ok());
    assert!(counter
        .with_timeout(Duration::from_millis(10))
        .respond_slow()
        .is_err());
}

#[test]
fn visibility() {
    mod m {
        use super::*;

        pub struct Counter {
            count: u32,
        }

        #[abstract_process(visibility = pub)]
        impl Counter {
            #[init]
            fn init(_config: Config<Self>, count: u32) -> Result<Self, ()> {
                Ok(Self { count })
            }

            #[handle_message]
            fn increment(&mut self) {
                self.count += 1;
            }

            #[handle_request]
            fn count(&self) -> u32 {
                self.count
            }
        }
    }

    use m::{Counter, CounterMessages, CounterRequests};
    let counter = Counter::link().start(2).unwrap();
    counter.with_delay(Duration::from_millis(10)).increment();
    assert_eq!(2, counter.count());
    sleep(Duration::from_millis(15));
    assert_eq!(3, counter.count());
}

#[test]
fn wrapper_rename() {
    pub struct Counter {
        count: u32,
    }

    #[abstract_process(message_trait_name = "CounterMsgExt",
                       request_trait_name = "CounterReqExt",
                       visibility = pub)]
    impl Counter {
        #[init]
        fn init(_config: Config<Self>, count: u32) -> Result<Self, ()> {
            Ok(Self { count })
        }

        #[handle_message]
        fn increment(&mut self) {
            self.count += 1;
        }

        #[handle_request]
        fn count(&self) -> u32 {
            self.count
        }
    }

    let counter = Counter::link().start(2).unwrap();
    counter.with_delay(Duration::from_millis(10)).increment();
    assert_eq!(2, counter.count());
    sleep(Duration::from_millis(15));
    assert_eq!(3, counter.count());
}

#[test]
fn generics() {
    use std::ops::{Add, AddAssign};

    use serde::de::Deserialize;
    use serde::ser::Serialize;

    struct GenAdder<T> {
        count: T,
    }

    #[abstract_process]
    impl<T> GenAdder<T>
    where
        T: Add + AddAssign + Default + Clone + Serialize + for<'de> Deserialize<'de> + 'static,
    {
        #[init]
        fn init(_: Config<Self>, _: ()) -> Result<Self, ()> {
            Ok(Self {
                count: T::default(),
            })
        }

        #[handle_message]
        fn add(&mut self, value: T) {
            self.count += value;
        }

        #[handle_request]
        fn sum(&self) -> T {
            self.count.clone()
        }
    }

    let counter = GenAdder::<f32>::link().start(()).unwrap();
    assert_eq!(0f32, counter.sum());
    counter.add(PI);
    assert_eq!(PI, counter.sum());
    counter.with_delay(Duration::from_millis(10)).add(PI);
    sleep(Duration::from_millis(15));
    let s = counter
        .with_timeout(Duration::from_millis(10))
        .sum()
        .unwrap();
    assert_eq!(PI * 2f32, s);
}
