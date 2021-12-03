use std::{process::exit, time::Duration};

use lunatic::{
    process::{self, Process},
    Mailbox, ReceiveError, Request, Tag,
};

#[lunatic::test]
fn message_integer(m: Mailbox<u64>) {
    let this = process::this(&m);
    let _child = process::spawn_with(this, |parent, _: Mailbox<()>| {
        parent.send(127);
    })
    .unwrap();
    assert_eq!(m.receive().unwrap(), 127);
}

#[lunatic::test]
fn message_vector(m: Mailbox<Vec<i32>>) {
    let this = process::this(&m);
    let _child = process::spawn_with(this, |parent, _: Mailbox<()>| {
        parent.send(vec![1, 2, 3, 4, 5]);
    })
    .unwrap();
    let sum: i32 = m.receive().unwrap().iter().sum();
    assert_eq!(sum, 15);
}

#[lunatic::test]
fn message_custom_type(m: Mailbox<X>) {
    let this = process::this(&m);
    let _child = process::spawn_with(this, |parent, _: Mailbox<()>| {
        let x = X {
            y: Y {
                string: String::from("Hello!"),
            },
            m: M { hello: 1337 },
            v: vec![(1, 1.22), (55555, 3.14)],
            en: E::A(1, 2),
            enb: E::B("A longer string #$".to_string()),
            enc: E::C,
        };
        parent.send(x);
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
    assert_eq!(m.receive().unwrap(), expected);
}

#[lunatic::test]
fn message_resource(m: Mailbox<Process<i32>>) {
    let this = process::this(&m);
    let _child = process::spawn_with(this, |parent, _: Mailbox<()>| {
        let empty_proc = process::spawn(|_: Mailbox<i32>| {}).unwrap();
        parent.send(empty_proc);
        let panic_proc = process::spawn(|_: Mailbox<i32>| exit(1)).unwrap();
        parent.send(panic_proc);
    })
    .unwrap();
    // Receive first
    let _ = m.receive();
    // Receive second
    let _ = m.receive();
}

#[lunatic::test]
fn request_reply(m: Mailbox<u64>) {
    // Spawn a server that fills our mailbox with u64 messages.
    let this = process::this(&m);
    process::spawn_with(this, |parent, _: Mailbox<()>| loop {
        parent.send(1337);
    })
    .unwrap();
    // Spawn another process that can reply to us with an i32 message.
    let add_server = process::spawn(|mailbox: Mailbox<Request<(i32, i32), i32>>| loop {
        let request = mailbox.receive().unwrap();
        let (a, b) = *request.data();
        request.reply(a + b);
    })
    .unwrap();
    // Ignore all messages in the mailbox and make specific requests to the `add_server`.
    for _ in 0..1_000 {
        assert_eq!(add_server.request((1, 1)).unwrap(), 2);
        assert_eq!(add_server.request((1, 2)).unwrap(), 3);
        assert_eq!(add_server.request((8, 8)).unwrap(), 16);
        assert_eq!(add_server.request((16, 16)).unwrap(), 32);
        assert_eq!(add_server.request((128, -128)).unwrap(), 0);
    }
}

#[lunatic::test]
fn timeout(m: Mailbox<u64>) {
    let result = m.receive_timeout(Duration::new(0, 10_000)); // 10 us
    match result {
        Err(ReceiveError::Timeout) => (), // success
        _ => unreachable!(),
    };
}

#[lunatic::test]
fn filter_by_tags(m: Mailbox<u64>) {
    let tags = [
        (Tag::new(), 0),
        (Tag::new(), 1),
        (Tag::new(), 2),
        (Tag::new(), 3),
        (Tag::new(), 4),
        (Tag::new(), 5),
        (Tag::new(), 6),
        (Tag::new(), 7),
    ];
    let this = process::this(&m);
    process::spawn_with((this, tags), |(parent, tags), _: Mailbox<()>| {
        for tag in tags {
            parent.tag_send(tag.0, tag.1);
        }
    })
    .unwrap();

    let receive_tags = [tags[7].0, tags[1].0, tags[3].0, tags[6].0, tags[5].0];
    // First tag in the mailbox should be 1.
    assert_eq!(m.tag_receive(&receive_tags).unwrap(), 1);
    assert_eq!(m.tag_receive(&receive_tags).unwrap(), 3);
    assert_eq!(m.tag_receive(&receive_tags).unwrap(), 5);
    assert_eq!(m.tag_receive(&receive_tags).unwrap(), 6);
    assert_eq!(m.tag_receive(&receive_tags).unwrap(), 7);
    // Asking for messages that are not part of the mailbox should timeout.
    assert!(m
        .tag_receive_timeout(&receive_tags, Duration::new(0, 100_000)) // 100 us
        .is_err());
    // The first next message should be 0
    assert_eq!(m.receive().unwrap(), 0);
    assert_eq!(m.receive().unwrap(), 2);
    assert_eq!(m.receive().unwrap(), 4);
}

#[cfg(feature = "serde_messagepack")]
#[lunatic::test]
fn test_msgpack_wrap_serialization(m: Mailbox<lunatic::message::MessagePack<X>>) {
    use lunatic::message::MessagePack;

    let this = process::this(&m);
    let _child = process::spawn_with(this, |parent, _: Mailbox<()>| {
        let x = X::some_x();
        parent.send(MessagePack(x));
    })
    .unwrap();
    let expected = X::some_x();
    assert_eq!(m.receive().unwrap().0, expected);
}

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

impl X {
    pub fn some_x() -> Self {
        X {
            y: Y {
                string: String::from("Hello!"),
            },
            m: M { hello: 1337 },
            v: vec![(1, 1.22), (55555, 3.14)],
            en: E::A(1, 2),
            enb: E::B("A longer string #$".to_string()),
            enc: E::C,
        }
    }
}

impl lunatic::Msg for X {
    type Serializer = lunatic::message::MessagePack<X>;
}
