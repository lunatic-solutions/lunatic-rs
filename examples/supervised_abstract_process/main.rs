/// Supervisor definition.

// Import the auto-generated `CounterHandler` public trait.
use counter_abstract_process::{Counter, CounterHandler}

use lunatic::supervisor::{Supervisor, SupervisorConfig, SupervisorStrategy};

// Supervisor definition.
struct Sup;
impl Supervisor for Sup {
    type Arg = ();
    // Start 1 child and monitor it for failures.
    type Children = Counter;

    fn init(config: &mut SupervisorConfig<Self>, _: ()) {
        // If the child fails, just restart it.
        config.set_strategy(SupervisorStrategy::OneForOne);
        // Start named child "hello".
        config.children_args((0, "hello".to_owned()));
    }
}

#[lunatic::main]
fn main(_: Mailbox<()>) {

    let sup = Sup::start((), None);

    // Get reference to named child.
    let hello = ProcessRef::<Counter>::lookup("hello").unwrap();

    // Accessible `increment` method.
    hello.increment();
}
