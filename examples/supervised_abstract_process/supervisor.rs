/// Supervisor definition.

struct Sup;
impl Supervisor for Sup {
    type Arg = ();
    // Start 3 `Counters` and monitor them for failures.
    type Children = (Counter, Counter, Counter);

    fn init(config: &mut SupervisorConfig<Self>, _: ()) {
        // If a child fails, just restart it.
        config.set_strategy(SupervisorStrategy::OneForOne);
        // Start each `Counter` with a state of `0` & name last child "hello".
        config.children_args((0, None),(0, None),(0, "hello".to_owned()));
    }
}

let sup = Sup::start((), None);
let children = sup.children();
let count1 = children.2.request(Count);
// Get reference to named child.
let hello = ProcessRef::<Counter>::lookup("hello").unwrap();
let count2 = hello.request(Count);
assert_eq!(count1, count2);
