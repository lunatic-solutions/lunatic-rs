use lunatic::{
    process::{self, Process},
    Mailbox,
};

#[test]
fn message_integer_test() {
    process::spawn(|m: Mailbox<i32>| {
        let (parent, m) = process::this(m);
        let _child = process::spawn_with(parent, |parent, _: Mailbox<()>| {
            parent.send(127);
        })
        .unwrap();
        assert_eq!(m.receive(), 127);
    })
    .unwrap()
    .join()
    .unwrap();
}

#[test]
fn message_vector_test() {
    process::spawn(|m: Mailbox<Vec<i32>>| {
        let (parent, m) = process::this(m);
        let _child = process::spawn_with(parent, |parent, _: Mailbox<()>| {
            parent.send(vec![1, 2, 3, 4, 5]);
        })
        .unwrap();
        let sum: i32 = m.receive().iter().sum();
        assert_eq!(sum, 15);
    })
    .unwrap()
    .join()
    .unwrap();
}

#[test]
fn message_custom_type_test() {
    use lunatic::derive::Message;

    #[derive(serde::Serialize, serde::Deserialize, Debug, PartialEq)]
    struct Y {
        string: String,
    }

    #[derive(Message, Debug, PartialEq)]
    struct M {
        hello: u64,
    }

    #[derive(Message, Debug, PartialEq)]
    struct X {
        y: Y,
        m: M,
        v: Vec<(i32, f64)>,
    }

    process::spawn(|m: Mailbox<X>| {
        let (parent, m) = process::this(m);
        let _child = process::spawn_with(parent, |parent, _: Mailbox<()>| {
            let x = X {
                y: Y {
                    string: String::from("Hello!"),
                },
                m: M { hello: 1337 },
                v: vec![(1, 1.22), (55555, 3.14)],
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
        };
        assert_eq!(m.receive(), expected);
    })
    .unwrap()
    .join()
    .unwrap();
}

#[test]
fn message_resource_test() {
    use lunatic::derive::Message;

    #[derive(Message)]
    struct Proc(Process<i32>);

    process::spawn(|m: Mailbox<Proc>| {
        let (parent, m) = process::this(m);
        let _child = process::spawn_with(parent, |parent, _: Mailbox<()>| {
            let empty_proc = process::spawn(|_: Mailbox<i32>| {}).unwrap();
            parent.send(Proc(empty_proc));
            let panic_proc = process::spawn(|_: Mailbox<i32>| panic!()).unwrap();
            parent.send(Proc(panic_proc));
        })
        .unwrap();
        // First succeeds
        assert_eq!(m.receive().0.join().is_ok(), true);
        // Second panics
        assert_eq!(m.receive().0.join().is_err(), true);
    })
    .unwrap()
    .join()
    .unwrap();
}
