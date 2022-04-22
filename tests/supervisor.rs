use std::time::Duration;

use lunatic::{
    process::{
        AbstractProcess, Message, ProcessMessage, ProcessRef, ProcessRequest, Request, StartProcess,
    },
    sleep,
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
            config.children_args(starting_state);
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
fn two_failing_process() {
    struct Sup;
    impl Supervisor for Sup {
        type Arg = ();
        type Children = (A, A);

        fn init(config: &mut SupervisorConfig<Self>, _: ()) {
            config.set_strategy(SupervisorStrategy::OneForOne);
            let starting_state_a = 33;
            let starting_state_b = 44;
            config.children_args((starting_state_a, starting_state_b));
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
fn ten_children_sup() {
    struct Sup;
    impl Supervisor for Sup {
        type Arg = ();
        type Children = (A, A, A, A, A, A, A, A, A, A);

        fn init(config: &mut SupervisorConfig<Self>, _: ()) {
            config.set_strategy(SupervisorStrategy::OneForOne);
            config.children_args((0, 0, 0, 0, 0, 0, 0, 0, 0, 0));
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
