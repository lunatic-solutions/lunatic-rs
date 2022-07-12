use std::time::Duration;

use lunatic::{
    process::{
        AbstractProcess, Message, ProcessMessage, ProcessRef, ProcessRequest, Request, StartProcess,
    },
    sleep, spawn,
    supervisor::{Supervisor, SupervisorConfig, SupervisorStrategy},
    test,
};

struct A(u32);

impl AbstractProcess for A {
    type Arg = u32;
    type State = A;

    fn init(_: ProcessRef<Self>, start: u32) -> A {
        A(start)
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
struct Inc;
impl ProcessMessage<Inc> for A {
    fn handle(state: &mut Self::State, _: Inc) {
        state.0 += 1;
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
struct Count;
impl ProcessRequest<Count> for A {
    type Response = u32;

    fn handle(state: &mut Self::State, _: Count) -> u32 {
        state.0
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
struct Panic;
impl ProcessMessage<Panic> for A {
    fn handle(_: &mut Self::State, _: Panic) {
        panic!();
    }
}

#[test]
fn one_failing_process() {
    struct Sup;
    impl Supervisor for Sup {
        type Arg = ();
        type Children = A;

        fn init(config: &mut SupervisorConfig<Self>, _: ()) {
            config.set_strategy(SupervisorStrategy::OneForOne);
            let starting_state = 4;
            config.children_args((starting_state, None));
        }
    }

    let sup = Sup::start((), None);

    let child = sup.children();

    // Starting state should be 4
    for i in 4..30 {
        assert_eq!(i, child.request(Count));
        child.send(Inc);
    }

    // Panicking is going to restart the count
    child.send(Panic);
    // We need to re-acquire reference to child and give a bit of time to the supervisor to re-spawn it.
    sleep(Duration::from_millis(10));
    let child = sup.children();

    // Starting state should be 4 again
    for i in 4..30 {
        assert_eq!(i, child.request(Count));
        child.send(Inc);
    }
}

#[test]
fn two_failing_process_one_for_one() {
    struct Sup;
    impl Supervisor for Sup {
        type Arg = ();
        type Children = (A, A);

        fn init(config: &mut SupervisorConfig<Self>, _: ()) {
            config.set_strategy(SupervisorStrategy::OneForOne);
            let starting_state_a = 33;
            let starting_state_b = 44;
            config.children_args(((starting_state_a, None), (starting_state_b, None)));
        }
    }

    let sup = Sup::start((), None);

    let (a, b) = sup.children();

    // Starting state should be 33 for a
    for i in 33..36 {
        assert_eq!(i, a.request(Count));
        a.send(Inc);
    }
    // Starting state should be 44 for b
    for i in 44..88 {
        assert_eq!(i, b.request(Count));
        b.send(Inc);
    }

    // Panicking b is going to restart the count
    b.send(Panic);

    // We need to re-acquire reference to child and give a bit of time to the supervisor to re-spawn it.
    sleep(Duration::from_millis(10));
    let (a, b) = sup.children();

    // The state for a shouldn't be restarted.
    for i in 36..99 {
        assert_eq!(i, a.request(Count));
        a.send(Inc);
    }
    // But b should
    for i in 44..66 {
        assert_eq!(i, b.request(Count));
        b.send(Inc);
    }

    // Panicking a is going to restart the count
    a.send(Panic);

    // We need to re-acquire reference to child and give a bit of time to the supervisor to re-spawn it.
    sleep(Duration::from_millis(10));
    let (a, b) = sup.children();

    // The state for a shouldn't be restarted.
    for i in 33..50 {
        assert_eq!(i, a.request(Count));
        a.send(Inc);
    }
    // But b should
    for i in 66..100 {
        assert_eq!(i, b.request(Count));
        b.send(Inc);
    }
}

#[test]
fn two_failing_process_one_for_all() {
    struct Sup;
    impl Supervisor for Sup {
        type Arg = ();
        type Children = (A, A);

        fn init(config: &mut SupervisorConfig<Self>, _: ()) {
            config.set_strategy(SupervisorStrategy::OneForAll);
            let starting_state_a = 33;
            let starting_state_b = 44;
            config.children_args(((starting_state_a, None), (starting_state_b, None)));
        }
    }

    let sup = Sup::start((), None);

    let (a, b) = sup.children();

    // Starting state should be 33 for a
    for i in 33..36 {
        assert_eq!(i, a.request(Count));
        a.send(Inc);
    }
    // Starting state should be 44 for b
    for i in 44..88 {
        assert_eq!(i, b.request(Count));
        b.send(Inc);
    }

    // Panicking b is going to restart the count
    b.send(Panic);

    // We need to re-acquire reference to child and give a bit of time to the supervisor to re-spawn it.
    sleep(Duration::from_millis(10));
    let (a, b) = sup.children();

    // The state for a should be restarted.
    for i in 33..36 {
        assert_eq!(i, a.request(Count));
        a.send(Inc);
    }
    // So should b
    for i in 44..66 {
        assert_eq!(i, b.request(Count));
        b.send(Inc);
    }

    // Panicking a is going to restart the count
    a.send(Panic);

    // We need to re-acquire reference to child and give a bit of time to the supervisor to re-spawn it.
    sleep(Duration::from_millis(10));
    let (a, b) = sup.children();

    // The state for a should be restarted.
    for i in 33..50 {
        assert_eq!(i, a.request(Count));
        a.send(Inc);
    }
    // So should a
    for i in 44..66 {
        assert_eq!(i, b.request(Count));
        b.send(Inc);
    }
}
#[test]
fn four_failing_process_rest_for_all() {
    struct Sup;
    impl Supervisor for Sup {
        type Arg = ();
        type Children = (A, A, A, A);

        fn init(config: &mut SupervisorConfig<Self>, _: ()) {
            config.set_strategy(SupervisorStrategy::RestForOne);
            let starting_state_a = 33;
            let starting_state_b = 44;
            let starting_state_c = 55;
            let starting_state_d = 66;
            config.children_args((
                (starting_state_a, None),
                (starting_state_b, None),
                (starting_state_c, None),
                (starting_state_d, None),
            ));
        }
    }

    let sup = Sup::start((), None);

    let (a, b, c, d) = sup.children();

    // Starting state should be 33 for a
    for i in 33..36 {
        assert_eq!(i, a.request(Count));
        a.send(Inc);
    }
    // Starting state should be 44 for b
    for i in 44..48 {
        assert_eq!(i, b.request(Count));
        b.send(Inc);
    }
    // Starting state should be 55 for c
    for i in 55..59 {
        assert_eq!(i, c.request(Count));
        c.send(Inc);
    }
    // Starting state should be 66 for d
    for i in 66..70 {
        assert_eq!(i, d.request(Count));
        d.send(Inc);
    }

    // Panicking b is going to restart the count
    b.send(Panic);

    // We need to re-acquire reference to child and give a bit of time to the supervisor to re-spawn it.
    sleep(Duration::from_millis(10));
    let (a, b, c, d) = sup.children();

    // The state for a shouldn't be restarted.
    for i in 36..99 {
        assert_eq!(i, a.request(Count));
        a.send(Inc);
    }
    // But b, c, d should
    for i in 44..48 {
        assert_eq!(i, b.request(Count));
        b.send(Inc);
    }
    for i in 55..59 {
        assert_eq!(i, c.request(Count));
        c.send(Inc);
    }
    for i in 66..70 {
        assert_eq!(i, d.request(Count));
        d.send(Inc);
    }

    // Panicking the first child should restart all children
    a.send(Panic);

    // We need to re-acquire reference to child and give a bit of time to the supervisor to re-spawn it.
    sleep(Duration::from_millis(10));
    let (a, b, c, d) = sup.children();

    // All children should have restarted
    for i in 33..36 {
        assert_eq!(i, a.request(Count));
        a.send(Inc);
    }
    for i in 44..48 {
        assert_eq!(i, b.request(Count));
        b.send(Inc);
    }
    for i in 55..59 {
        assert_eq!(i, c.request(Count));
        c.send(Inc);
    }
    for i in 66..70 {
        assert_eq!(i, d.request(Count));
        d.send(Inc);
    }

    // Panicking the last child
    d.send(Panic);
    sleep(Duration::from_millis(10));
    let (a, b, c, d) = sup.children();

    // Only the last child should have restarted
    for i in 36..40 {
        assert_eq!(i, a.request(Count));
        a.send(Inc);
    }
    for i in 48..52 {
        assert_eq!(i, b.request(Count));
        b.send(Inc);
    }
    for i in 59..63 {
        assert_eq!(i, c.request(Count));
        c.send(Inc);
    }
    for i in 66..70 {
        assert_eq!(i, d.request(Count));
        d.send(Inc);
    }
}

#[test]
fn ten_children_sup() {
    struct Sup;
    impl Supervisor for Sup {
        type Arg = ();
        type Children = (A, A, A, A, A, A, A, A, A, A);

        fn init(config: &mut SupervisorConfig<Self>, _: ()) {
            config.set_strategy(SupervisorStrategy::OneForOne);
            config.children_args((
                (0, None),
                (0, None),
                (0, None),
                (0, None),
                (0, None),
                (0, None),
                (0, None),
                (0, None),
                (0, None),
                (0, None),
            ));
        }
    }

    Sup::start_link((), None);
}

#[test]
#[should_panic]
fn children_args_not_called() {
    struct Sup;
    impl Supervisor for Sup {
        type Arg = ();
        type Children = A;

        fn init(config: &mut SupervisorConfig<Self>, _: ()) {
            config.set_strategy(SupervisorStrategy::OneForOne);
            // config.children_args(0);
        }
    }

    Sup::start_link((), None);
}

#[test]
fn shutdown() {
    struct A;

    impl AbstractProcess for A {
        type Arg = ();
        type State = A;

        fn init(proc: ProcessRef<Self>, _: ()) -> A {
            println!("{}", proc.uuid());
            A
        }

        fn terminate(_: Self::State) {
            println!("Exit");
        }
    }

    struct Sup;
    impl Supervisor for Sup {
        type Arg = ();
        type Children = (A, A, A, A);

        fn init(config: &mut SupervisorConfig<Self>, _: ()) {
            config.set_strategy(SupervisorStrategy::OneForOne);
            config.children_args((((), None), ((), None), ((), None), ((), None)));
        }
    }

    let sup = Sup::start((), None);
    sup.shutdown();

    sleep(Duration::from_millis(100));
}

#[test]
fn lookup_children() {
    struct Sup;
    impl Supervisor for Sup {
        type Arg = ();
        type Children = (A, A, A);

        fn init(config: &mut SupervisorConfig<Self>, _: ()) {
            config.set_strategy(SupervisorStrategy::OneForOne);
            config.children_args((
                (0, Some("first".to_owned())),
                (1, Some("second".to_owned())),
                (2, Some("third".to_owned())),
            ));
        }
    }

    Sup::start_link((), None);

    let first = ProcessRef::<A>::lookup("first").unwrap();
    assert_eq!(first.request(Count), 0);
    let second = ProcessRef::<A>::lookup("second").unwrap();
    assert_eq!(second.request(Count), 1);
    let third = ProcessRef::<A>::lookup("third").unwrap();
    assert_eq!(third.request(Count), 2);

    // Kill third and inc count to 4
    third.send(Panic);
    // We need to re-acquire reference to child and give a bit of time to the supervisor to re-spawn it.
    sleep(Duration::from_millis(10));
    let third = ProcessRef::<A>::lookup("third").unwrap();
    third.send(Inc);
    third.send(Inc);
    assert_eq!(third.request(Count), 4);
    // Holding multiple references is ok
    let third = ProcessRef::<A>::lookup("third").unwrap();
    assert_eq!(third.request(Count), 4);
}

#[test]
fn block_until_shutdown() {
    struct Sup;
    impl Supervisor for Sup {
        type Arg = ();
        type Children = A;

        fn init(config: &mut SupervisorConfig<Self>, _: ()) {
            config.set_strategy(SupervisorStrategy::OneForOne);
            config.children_args((0, None));
        }
    }

    let sup = Sup::start_link((), None);
    let sup_cloned = sup.clone();

    // Shutdown supervisor process after a delay
    spawn!(|sup, _mailbox: Mailbox<()>| {
        sleep(Duration::from_millis(10));
        sup.shutdown();
    });

    // block main process until supervisor shuts down
    // the test will hang if block_until_shutdown() fails
    sup_cloned.block_until_shutdown()
}
