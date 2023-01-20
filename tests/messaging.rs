use std::time::Duration;

use lunatic::ap::handlers::Request;
use lunatic::ap::{AbstractProcess, Config, RequestHandler, State};
use lunatic::serializer::Json;
use lunatic::{spawn_link, Mailbox, Process};
use lunatic_test::test;

#[test]
fn message_integer() {
    let child = spawn_link!(@task |input = 127| input);
    assert_eq!(child.result(), 127);
}

#[test]
fn message_vector() {
    let child = spawn_link!(@task |input = { vec![1, 2, 3, 4, 5] }| input);
    assert_eq!(child.result(), vec![1, 2, 3, 4, 5]);
}

#[test]
fn message_custom_type() {
    let child = spawn_link!(@task || X {
        y: Y {
            string: String::from("Hello!"),
        },
        m: M { hello: 1337 },
        v: vec![(1, 1.22), (55555, 3.14)],
        en: E::A(1, 2),
        enb: E::B("A longer string #$".to_string()),
        enc: E::C,
    });

    let expected = X {
        y: Y {
            string: String::from("Hello!"),
        },
        m: M { hello: 1337 },
        v: vec![(1, 1.22), (55555, 3.14)],
        en: E::A(1, 2),
        enb: E::B("A longer string #$".to_string()),
        enc: E::C,
    };
    assert_eq!(child.result(), expected);
}

#[test]
fn message_resource(mailbox: Mailbox<Proc>) {
    let this = mailbox.this();
    Process::spawn(this, |parent, _: Mailbox<()>| {
        let empty_proc = Process::spawn((), |_, _: Mailbox<i32>| {});
        parent.send(Proc(empty_proc));
        let panic_proc = Process::spawn((), |_, _: Mailbox<i32>| panic!());
        parent.send(Proc(panic_proc));
    });
    // Receive first
    let _ = mailbox.receive();
    // Receive second
    let _ = mailbox.receive();
}

#[test]
fn message_dead_process() {
    let child = Process::spawn((), |_, _: Mailbox<()>| {});
    // Give enough time to finish
    lunatic::sleep(Duration::from_millis(100));
    child.send(());
}

#[test]
fn request_reply(mailbox: Mailbox<u64>) {
    struct Adder;
    impl AbstractProcess for Adder {
        type Arg = ();
        type State = Self;
        type Serializer = Json;
        type Handlers = (Request<(i32, i32)>,);
        type StartupError = ();

        fn init(_: Config<Self>, _: ()) -> Result<Adder, ()> {
            Ok(Adder)
        }
    }
    impl RequestHandler<(i32, i32)> for Adder {
        type Response = i32;

        fn handle(_: State<Self>, (a, b): (i32, i32)) -> i32 {
            a + b
        }
    }

    // Spawn a server that fills our mailbox with u64 messages.
    Process::spawn(mailbox.this(), |parent, _: Mailbox<()>| loop {
        parent.send(1337);
    });

    // Spawn another process that can reply to us with an i32 message.
    let add_server = Adder::link().start(()).unwrap();

    // Ignore all messages in the mailbox and make specific requests to the
    // `add_server`.
    for _ in 0..1_000 {
        assert_eq!(add_server.request((1, 1), None).unwrap(), 2);
        assert_eq!(add_server.request((1, 2), None).unwrap(), 3);
        assert_eq!(add_server.request((8, 8), None).unwrap(), 16);
        assert_eq!(add_server.request((16, 16), None).unwrap(), 32);
        assert_eq!(add_server.request((128, -128), None).unwrap(), 0);
    }
}

#[test]
fn timeout(mailbox: Mailbox<u64>) {
    let result = mailbox.receive_timeout(Duration::new(0, 10_000)); // 10 us
    assert!(result.is_timed_out())
}

#[derive(serde::Serialize, serde::Deserialize)]
struct Proc(Process<i32>);

#[derive(serde::Serialize, serde::Deserialize, Debug, PartialEq)]
struct Y {
    string: String,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, PartialEq)]
struct M {
    hello: u64,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, PartialEq)]
enum E {
    A(u32, u32),
    B(String),
    C,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, PartialEq)]
struct X {
    y: Y,
    m: M,
    v: Vec<(i32, f64)>,
    en: E,
    enb: E,
    enc: E,
}
