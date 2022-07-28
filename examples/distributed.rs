use std::time::Duration;

use lunatic::ProcessConfig;
use lunatic::{host::node_id, sleep, Mailbox, Process};

use lunatic::process::{AbstractProcess, ProcessRef, Request, RequestHandler, StartProcess};

struct Adder;
impl AbstractProcess for Adder {
    type Arg = ();
    type State = Self;

    fn init(_: ProcessRef<Self>, _: ()) -> Adder {
        Adder
    }
}
impl RequestHandler<(i32, i32)> for Adder {
    type Response = i32;

    fn handle(_: &mut Self::State, (a, b): (i32, i32)) -> i32 {
        println!("Got {a}, {b} to add");
        a + b
    }
}

fn main() {
    let nodes = lunatic::distributed::nodes();

    println!("Nodes {nodes:?}");

    let mut config = ProcessConfig::new();
    config.set_max_memory(1_500_000);
    config.set_max_fuel(1);

    if !nodes.is_empty() {
        let add_server = Adder::start_node_config((), None, nodes[0], &config);
        assert_eq!(add_server.request((1, 1)), 2);
    }

    let msgs = [10, 582, 172, 45];
    let procs = nodes
        .into_iter()
        .map(|node| Process::spawn_node_config(node, &config, 101, hello));

    for (i, proc) in procs.enumerate() {
        proc.send(msgs[i % msgs.len()]);
    }

    sleep(Duration::from_millis(5000));
}

fn hello(start: u32, mailbox: Mailbox<u32>) {
    println!("Hi from {}", node_id());
    let m = mailbox.receive();
    println!("{start} + {m} = {}", start + m);
    sleep(Duration::from_millis(2000));
}
