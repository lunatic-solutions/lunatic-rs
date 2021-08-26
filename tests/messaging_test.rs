use lunatic::{
    process::{self, Process},
    Mailbox,
};

#[lunatic::test]
fn message_integer_test(m: Mailbox<u64>) {
    let (this, m) = process::this(m);
    let _child = process::spawn_with(this, |parent, _: Mailbox<()>| {
        parent.send(127);
    })
    .unwrap();
    assert_eq!(m.receive().unwrap(), 127);
}

#[lunatic::test]
fn message_vector_test(m: Mailbox<Vec<i32>>) {
    let (this, m) = process::this(m);
    let _child = process::spawn_with(this, |parent, _: Mailbox<()>| {
        parent.send(vec![1, 2, 3, 4, 5]);
    })
    .unwrap();
    let sum: i32 = m.receive().unwrap().iter().sum();
    assert_eq!(sum, 15);
}

#[lunatic::test]
fn message_custom_type_test(m: Mailbox<X>) {
    let (this, m) = process::this(m);
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
fn message_resource_test(m: Mailbox<Proc>) {
    let (this, m) = process::this(m);
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
