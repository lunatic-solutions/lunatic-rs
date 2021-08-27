use lunatic::{
    process::{self, Process},
    Mailbox, Request,
};

#[lunatic::test]
fn message_integer(m: Mailbox<u64>) {
    let this = process::this(m);
    let _child = process::spawn_with(this, |parent, _: Mailbox<()>| {
        parent.send(127);
    })
    .unwrap();
    assert_eq!(m.receive().unwrap(), 127);
}

#[lunatic::test]
fn message_vector(m: Mailbox<Vec<i32>>) {
    let this = process::this(m);
    let _child = process::spawn_with(this, |parent, _: Mailbox<()>| {
        parent.send(vec![1, 2, 3, 4, 5]);
    })
    .unwrap();
    let sum: i32 = m.receive().unwrap().iter().sum();
    assert_eq!(sum, 15);
}

#[lunatic::test]
fn message_custom_type(m: Mailbox<X>) {
    let this = process::this(m);
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
fn message_resource(m: Mailbox<Proc>) {
    let this = process::this(m);
    let _child = process::spawn_with(this, |parent, _: Mailbox<()>| {
        let empty_proc = process::spawn(|_: Mailbox<i32>| {}).unwrap();
        parent.send(Proc(empty_proc));
        let panic_proc = process::spawn(|_: Mailbox<i32>| panic!()).unwrap();
        parent.send(Proc(panic_proc));
    })
    .unwrap();
    // Receive first
    let _ = m.receive();
    // Receive second
    let _ = m.receive();
}

#[lunatic::test]
fn request_replay(m: Mailbox<u64>) {
    // Spawn a server that fills our mailbox with u64 messages.
    let this = process::this(m);
    process::spawn_with(this, |parent, _: Mailbox<()>| loop {
        parent.send(1337);
    })
    .unwrap();
    // Spawn another process that can replay to us with an i32 message.
    let add_server = process::spawn(|mailbox: Mailbox<Request<(i32, i32), i32>>| loop {
        let request = mailbox.receive().unwrap();
        let (a, b) = *request.data();
        request.replay(a + b);
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
