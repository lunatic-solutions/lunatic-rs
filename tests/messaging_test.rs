use std::{process::exit, time::Duration};

use lunatic::{spawn, this_process, Mailbox, Process, ReceiveError, Server, Task};
use lunatic_test::test;

#[test]
fn message_integer() {
    let child = spawn::<Task<_>, _>(127, |input| input).unwrap();
    assert_eq!(child.result(), 127);
}

#[test]
fn message_vector() {
    let child = spawn::<Task<_>, _>(vec![1, 2, 3, 4, 5], |input| input).unwrap();
    assert_eq!(child.result(), vec![1, 2, 3, 4, 5]);
}

#[test]
fn message_custom_type() {
    let child = spawn::<Task<_>, _>((), |_| X {
        y: Y {
            string: String::from("Hello!"),
        },
        m: M { hello: 1337 },
        v: vec![(1, 1.22), (55555, 3.14)],
        en: E::A(1, 2),
        enb: E::B("A longer string #$".to_string()),
        enc: E::C,
    })
    .unwrap();

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
fn message_resource() {
    let this_mailbox = unsafe { Mailbox::new() };
    let this: Process<Proc> = this_process(&this_mailbox);
    spawn::<Process<_>, _>(this, |parent, _: Mailbox<()>| {
        let empty_proc = spawn::<Process<_>, _>((), |_, _: Mailbox<i32>| {}).unwrap();
        parent.send(Proc(empty_proc));
        let panic_proc = spawn::<Process<_>, _>((), |_, _: Mailbox<i32>| exit(1)).unwrap();
        parent.send(Proc(panic_proc));
    })
    .unwrap();
    // Receive first
    let _ = this_mailbox.receive();
    // Receive second
    let _ = this_mailbox.receive();
}

#[test]
fn request_reply() {
    // Spawn a server that fills our mailbox with u64 messages.
    let this_mailbox = unsafe { Mailbox::new() };
    let this: Process<u64> = this_process(&this_mailbox);
    spawn::<Process<_>, _>(this, |parent, _: Mailbox<()>| loop {
        parent.send(1337);
    })
    .unwrap();

    // Spawn another process that can reply to us with an i32 message.
    let add_server = spawn::<Server<_, _>, _>((), |_, (a, b)| a + b).unwrap();

    // Ignore all messages in the mailbox and make specific requests to the `add_server`.
    for _ in 0..1_000 {
        assert_eq!(add_server.request((1, 1)), 2);
        assert_eq!(add_server.request((1, 2)), 3);
        assert_eq!(add_server.request((8, 8)), 16);
        assert_eq!(add_server.request((16, 16)), 32);
        assert_eq!(add_server.request((128, -128)), 0);
    }
}

// TODO: All tests run in same process and the mailbox is already full of messages.
//       Once the testing story around processes is solved this can be used.
// #[test]
fn _timeout() {
    let this_mailbox = unsafe { Mailbox::<u64>::new() };
    let result = this_mailbox.receive_timeout(Duration::new(0, 10_000)); // 10 us
    match result {
        Err(ReceiveError::Timeout) => (), // success
        _ => unreachable!(),
    };
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
