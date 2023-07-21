use std::time::Duration;

use lunatic::ap::handlers::{Message, Request};
use lunatic::ap::{AbstractProcess, Config, MessageHandler, ProcessRef, RequestHandler, State};
use lunatic::serializer::{Json, MessagePack};
use lunatic::supervisor::{Supervisor, SupervisorConfig, SupervisorStrategy};
use lunatic::{sleep, spawn, test, ProcessConfig};

const LOGGER_NAME: &'static str = "logger/assert_order";

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, PartialEq, Eq)]
enum LogEvent {
    Init(char),
    Panic(char),
    Shutdown(char),
}

struct Logger {
    logs: Vec<LogEvent>,
}

impl AbstractProcess for Logger {
    type Arg = ();
    type State = Logger;
    type Serializer = Json;
    type Handlers = (Request<LogEvent>, Request<TakeLogs>);
    type StartupError = ();

    fn init(_: Config<Logger>, _arg: Self::Arg) -> Result<Self::State, ()> {
        Ok(Logger { logs: vec![] })
    }
}

impl RequestHandler<LogEvent> for Logger {
    type Response = ();

    fn handle(mut state: State<Self>, request: LogEvent) -> Self::Response {
        state.logs.push(request);
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
struct TakeLogs;
impl RequestHandler<TakeLogs> for Logger {
    type Response = Vec<LogEvent>;

    fn handle(mut state: State<Self>, _request: TakeLogs) -> Self::Response {
        std::mem::replace(&mut state.logs, vec![])
    }
}

struct A {
    count: u32,
    name: char,
}

impl AbstractProcess for A {
    type Arg = (u32, char);
    type State = A;
    type Serializer = MessagePack;
    type Handlers = (
        Message<Inc>,
        Request<Count>,
        Message<Panic>,
        Request<GetEnvVar>,
    );
    type StartupError = ();

    fn init(_: Config<Self>, (count, name): Self::Arg) -> Result<A, ()> {
        if let Some(logger) = ProcessRef::<Logger>::lookup(&LOGGER_NAME) {
            let log = LogEvent::Init(name);
            logger.request(log);
        }
        Ok(A { count, name })
    }

    fn terminate(state: Self::State) {
        if let Some(logger) = ProcessRef::<Logger>::lookup(&LOGGER_NAME) {
            let log = LogEvent::Shutdown(state.name);
            logger.request(log);
        }
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
struct Inc;
impl MessageHandler<Inc> for A {
    fn handle(mut state: State<Self>, _: Inc) {
        state.count += 1;
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
struct Count;
impl RequestHandler<Count> for A {
    type Response = u32;

    fn handle(state: State<Self>, _: Count) -> u32 {
        state.count
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
struct Panic;
impl MessageHandler<Panic> for A {
    fn handle(state: State<Self>, _: Panic) {
        if let Some(logger) = ProcessRef::<Logger>::lookup(&LOGGER_NAME) {
            let log = LogEvent::Panic(state.name);
            logger.request(log);
        }
        panic!();
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
struct GetEnvVar(String);
impl RequestHandler<GetEnvVar> for A {
    type Response = Option<String>;

    fn handle(_: State<Self>, env_var: GetEnvVar) -> Option<String> {
        // Look up environment variable inside the process and return it.
        let mut vars = std::env::vars();
        vars.find(|(key, _)| key == &env_var.0)
            .map(|(_, value)| value)
    }
}

#[test]
fn one_failing_process() {
    struct Sup;
    impl Supervisor for Sup {
        type Arg = ();
        type Children = (A,);

        fn init(config: &mut SupervisorConfig<Self>, _: ()) {
            config.set_strategy(SupervisorStrategy::OneForOne);
            let starting_state = (4, ' ');
            config.set_args((starting_state,));
        }
    }

    let sup = Sup::link().start(()).unwrap();

    let child = sup.children().0;

    // Starting state should be 4
    for i in 4..30 {
        assert_eq!(i, child.request(Count));
        child.send(Inc);
    }

    // Panicking is going to restart the count
    child.send(Panic);
    // We need to re-acquire reference to child and give a bit of time to the
    // supervisor to re-spawn it.
    sleep(Duration::from_millis(10));
    let child = sup.children().0;

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
            let starting_state_a = (33, 'a');
            let starting_state_b = (44, 'b');
            config.set_args((starting_state_a, starting_state_b));
        }
    }

    let logger = Logger::link().start_as(&LOGGER_NAME, ()).unwrap();
    let sup = Sup::link().start(()).unwrap();

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
    sleep(Duration::from_millis(10));

    let log = logger.request(TakeLogs);
    assert_eq!(
        log,
        vec![
            // initial spawn
            LogEvent::Init('a'),
            LogEvent::Init('b'),
            // panic
            LogEvent::Panic('b'),
            // restart
            LogEvent::Init('b'),
        ]
    );

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

    // Panicking is going to restart the count
    a.send(Panic);
    sleep(Duration::from_millis(10));

    let log = logger.request(TakeLogs);
    assert_eq!(
        log,
        vec![
            // panic
            LogEvent::Panic('a'),
            // restart
            LogEvent::Init('a'),
        ]
    );

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
            let starting_state_a = (33, 'a');
            let starting_state_b = (44, 'b');
            config.set_args((starting_state_a, starting_state_b));
        }
    }

    let logger = Logger::link().start_as(&LOGGER_NAME, ()).unwrap();
    let sup = Sup::link().start(()).unwrap();

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
    sleep(Duration::from_millis(10));

    let log = logger.request(TakeLogs);
    assert_eq!(
        log,
        vec![
            // initial spawn
            LogEvent::Init('a'),
            LogEvent::Init('b'),
            // panic
            LogEvent::Panic('b'),
            // shutdown
            LogEvent::Shutdown('a'),
            // restart
            LogEvent::Init('a'),
            LogEvent::Init('b'),
        ]
    );

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

    // Panicking is going to restart the count
    a.send(Panic);
    sleep(Duration::from_millis(10));

    let log = logger.request(TakeLogs);
    assert_eq!(
        log,
        vec![
            // panic
            LogEvent::Panic('a'),
            // shutdown
            LogEvent::Shutdown('b'),
            // restart
            LogEvent::Init('a'),
            LogEvent::Init('b'),
        ]
    );

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
            let starting_state_a = (33, 'a');
            let starting_state_b = (44, 'b');
            let starting_state_c = (55, 'c');
            let starting_state_d = (66, 'd');
            config.set_args((
                starting_state_a,
                starting_state_b,
                starting_state_c,
                starting_state_d,
            ));
        }
    }

    let logger = Logger::link().start_as(&LOGGER_NAME, ()).unwrap();
    let sup = Sup::link().start(()).unwrap();

    let (_, b, _, _) = sup.children();

    // Panicking `b` is going to shut down `c` and `d` in reverse order and start
    // them up again.
    b.send(Panic);
    sleep(Duration::from_millis(10));

    let logs = logger.request(TakeLogs);
    assert_eq!(
        logs,
        vec![
            // initial spawn
            LogEvent::Init('a'),
            LogEvent::Init('b'),
            LogEvent::Init('c'),
            LogEvent::Init('d'),
            // panic
            LogEvent::Panic('b'),
            // shutdown
            LogEvent::Shutdown('d'),
            LogEvent::Shutdown('c'),
            // restart
            LogEvent::Init('b'),
            LogEvent::Init('c'),
            LogEvent::Init('d'),
        ]
    );
    println!("wroks");
    // Panicking the first child should restart all children
    let (a, _, _, _) = sup.children();
    a.send(Panic);
    sleep(Duration::from_millis(10));

    let logs = logger.request(TakeLogs);
    assert_eq!(
        logs,
        vec![
            // panic
            LogEvent::Panic('a'),
            // shutdown
            LogEvent::Shutdown('d'),
            LogEvent::Shutdown('c'),
            LogEvent::Shutdown('b'),
            // restart
            LogEvent::Init('a'),
            LogEvent::Init('b'),
            LogEvent::Init('c'),
            LogEvent::Init('d'),
        ]
    );
    println!("wroks");
    // Panicking the last child
    let (_, _, _, d) = sup.children();
    println!("wroks");
    d.send(Panic);
    sleep(Duration::from_millis(10));

    let logs = logger.request(TakeLogs);
    assert_eq!(
        logs,
        vec![
            // panic
            LogEvent::Panic('d'),
            // no shutdown only restart
            LogEvent::Init('d'),
        ]
    );
}

#[test]
fn ten_children_sup() {
    struct Sup;
    impl Supervisor for Sup {
        type Arg = ();
        type Children = (A, A, A, A, A, A, A, A, A, A);

        fn init(config: &mut SupervisorConfig<Self>, _: ()) {
            config.set_strategy(SupervisorStrategy::OneForOne);
            config.set_args((
                (0, ' '),
                (0, ' '),
                (0, ' '),
                (0, ' '),
                (0, ' '),
                (0, ' '),
                (0, ' '),
                (0, ' '),
                (0, ' '),
                (0, ' '),
            ));
        }
    }

    Sup::link().start(()).unwrap();
}

#[test]
#[should_panic]
fn children_args_not_called() {
    struct Sup;
    impl Supervisor for Sup {
        type Arg = ();
        type Children = (A,);

        fn init(config: &mut SupervisorConfig<Self>, _: ()) {
            config.set_strategy(SupervisorStrategy::OneForOne);
            // config.children_args(0);
        }
    }

    Sup::link().start(()).unwrap();
}

#[test]
fn shutdown() {
    struct Sup;
    impl Supervisor for Sup {
        type Arg = ();
        type Children = (A, A, A, A);

        fn init(config: &mut SupervisorConfig<Self>, _: ()) {
            config.set_strategy(SupervisorStrategy::OneForOne);
            config.set_args(((0, 'a'), (0, 'b'), (0, 'c'), (0, 'd')));
        }
    }

    let logger = Logger::link().start_as(&LOGGER_NAME, ()).unwrap();
    let sup = Sup::link().start(()).unwrap();
    sup.shutdown();
    let log = logger.request(TakeLogs);
    assert_eq!(
        log,
        vec![
            LogEvent::Init('a'),
            LogEvent::Init('b'),
            LogEvent::Init('c'),
            LogEvent::Init('d'),
            LogEvent::Shutdown('d'),
            LogEvent::Shutdown('c'),
            LogEvent::Shutdown('b'),
            LogEvent::Shutdown('a'),
        ],
    );
}

#[test]
fn lookup_children() {
    struct Sup;
    impl Supervisor for Sup {
        type Arg = ();
        type Children = (A, A, A);

        fn init(config: &mut SupervisorConfig<Self>, _: ()) {
            config.set_strategy(SupervisorStrategy::OneForOne);
            config.set_args(((0, ' '), (1, ' '), (2, ' ')));
            config.set_names((
                Some("first".to_owned()),
                Some("second".to_owned()),
                Some("third".to_owned()),
            ))
        }
    }

    Sup::link().start(()).unwrap();

    let first = ProcessRef::<A>::lookup(&"first").unwrap();
    assert_eq!(first.request(Count), 0);
    let second = ProcessRef::<A>::lookup(&"second").unwrap();
    assert_eq!(second.request(Count), 1);
    let third = ProcessRef::<A>::lookup(&"third").unwrap();
    assert_eq!(third.request(Count), 2);

    // Kill third and inc count to 4
    third.send(Panic);
    sleep(Duration::from_millis(10));
    let third = ProcessRef::<A>::lookup(&"third").unwrap();
    third.send(Inc);
    third.send(Inc);
    assert_eq!(third.request(Count), 4);
    // Holding multiple references is ok
    let third = ProcessRef::<A>::lookup(&"third").unwrap();
    assert_eq!(third.request(Count), 4);
}

#[test]
fn wait_on_shutdown() {
    struct Sup;
    impl Supervisor for Sup {
        type Arg = ();
        type Children = (A,);

        fn init(config: &mut SupervisorConfig<Self>, _: ()) {
            config.set_strategy(SupervisorStrategy::OneForOne);
            config.set_args(((0, ' '),));
        }
    }

    let sup = Sup::link().start(()).unwrap();
    let sup_cloned = sup.clone();

    // Shutdown supervisor process after a delay
    spawn!(|sup, _mailbox: Mailbox<()>| {
        sleep(Duration::from_millis(10));
        sup.shutdown();
    });

    // block main process until supervisor shuts down
    // the test will hang if block_until_shutdown() fails
    sup_cloned.wait_on_shutdown()
}

#[test]
fn env_var_config() {
    struct Sup;
    impl Supervisor for Sup {
        type Arg = ();
        type Children = (A,);

        fn init(config: &mut SupervisorConfig<Self>, _: ()) {
            config.set_strategy(SupervisorStrategy::OneForOne);
            config.set_args(((0, ' '),));
            config.set_names((Some("named".to_owned()),));
            let mut process_config = ProcessConfig::new().unwrap();
            process_config.add_environment_variable("Hello", "world");
            config.set_configs((Some(process_config),));
        }
    }

    Sup::link().start(()).unwrap();

    let named = ProcessRef::<A>::lookup(&"named").unwrap();
    assert_eq!(
        named.request(GetEnvVar("Hello".to_string())),
        Some("world".to_string())
    );
    assert_eq!(named.request(GetEnvVar("no".to_string())), None);
    // Kill
    named.send(Panic);
    sleep(Duration::from_millis(10));
    let named = ProcessRef::<A>::lookup(&"named").unwrap();
    assert_eq!(
        named.request(GetEnvVar("Hello".to_string())),
        Some("world".to_string())
    );
    assert_eq!(named.request(GetEnvVar("no".to_string())), None);
}
