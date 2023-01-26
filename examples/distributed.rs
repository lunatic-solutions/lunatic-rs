use std::time::Duration;

use lunatic::ap::handlers::Request;
use lunatic::ap::{AbstractProcess, Config, RequestHandler, State};
use lunatic::host::node_id;
use lunatic::serializer::MessagePack;
use lunatic::{sleep, Mailbox, Process, ProcessConfig};

struct Adder;
impl AbstractProcess for Adder {
    type Arg = ();
    type State = Self;
    type Handlers = (Request<(i32, i32)>,);
    type Serializer = MessagePack;
    type StartupError = ();

    fn init(_: Config<Self>, _: ()) -> Result<Adder, ()> {
        Ok(Adder)
    }
}
impl RequestHandler<(i32, i32)> for Adder {
    type Response = i32;

    fn handle(_: State<Self>, (a, b): (i32, i32)) -> i32 {
        println!("Got {a}, {b} to add");
        a + b
    }
}

fn main() {
    let nodes = lunatic::distributed::nodes();

    println!("Nodes {nodes:?}");

    let mut config = ProcessConfig::new().unwrap();
    config.set_max_memory(1_500_000);
    config.set_max_fuel(1);

    if !nodes.is_empty() {
        let add_server = Adder::on_node(nodes[0])
            .configure(&config)
            .start(())
            .unwrap();
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
