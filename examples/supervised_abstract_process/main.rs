use std::env;
use std::time::Duration;
mod counter_abstract_process;

// Import the auto-generated `CounterHandler` public trait.
use counter_abstract_process::{Counter, CounterMessages, CounterRequests};
use lunatic::ap::{AbstractProcess, ProcessRef};
use lunatic::supervisor::{Supervisor, SupervisorConfig, SupervisorStrategy};
use lunatic::{Mailbox, ProcessConfig, sleep};

// Supervisor definition.
struct Sup;
impl Supervisor for Sup {
    type Arg = ();
    // Start 1 child and monitor it for failures.
    type Children = (Counter,);

    fn init(config: &mut SupervisorConfig<Self>, _: ()) {
        // If the child fails, just restart it.
        config.set_strategy(SupervisorStrategy::OneForOne);
        // Start child with stat 0
        config.set_args((0,));
        // Name child 'hello'
        config.set_names((Some("hello".to_owned()),));
        let mut process_config = ProcessConfig::new().unwrap();
            process_config.add_environment_variable("PYTHONPATH", "/foo/bar");
            config.set_configs((Some(process_config),));
    }
}

#[lunatic::main]
fn main(_: Mailbox<()>) {
    Sup::start(()).unwrap();

    // Get reference to named child.
    let hello = ProcessRef::<Counter>::lookup(&"hello").unwrap();

    // Accessible `increment` method.
    hello.increment();
    hello.increment();

    assert_eq!(hello.count(), 2);

    // Give everything time to print.
    sleep(Duration::from_millis(1));
}
