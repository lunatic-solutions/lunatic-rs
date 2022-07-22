use std::time::Duration;

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

    let add_server = Adder::start_node((), None, nodes[0]);

    println!("Nodes {nodes:?}");

    let msgs = [10, 582, 172, 45];
    let procs = nodes
        .into_iter()
        .map(|node| Process::spawn_node(node, 101, hello));

    for (i, proc) in procs.enumerate() {
        proc.send(msgs[i % msgs.len()]);
    }

    assert_eq!(add_server.request((1, 1)), 2);

    sleep(Duration::from_millis(5000));
}

fn hello(start: u32, mailbox: Mailbox<u32>) {
    println!("Hi from {}", node_id());
    let m = mailbox.receive();
    println!("{start} + {m} = {}", start + m);
    sleep(Duration::from_millis(2000));
}
