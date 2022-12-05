mod counter_abstract_process;

// Import the auto-generated `CounterHandler` public trait.
use counter_abstract_process::{Counter, CounterHandler};
use lunatic::process::{ProcessRef, StartProcess};
use lunatic::supervisor::{Supervisor, SupervisorConfig, SupervisorStrategy};
use lunatic::Mailbox;

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
        config.children_args((0, Some("hello".to_owned())));
    }
}

#[lunatic::main]
fn main(_: Mailbox<()>) {
    Sup::start((), None);

    // Get reference to named child.
    let hello = ProcessRef::<Counter>::lookup("hello").unwrap();

    // Accessible `increment` method.
    hello.increment();
    hello.increment();

    assert_eq!(hello.count(), 2);
}
