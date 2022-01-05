// To run this example you will need to have a remote node under the name "foo".
// You can start one with: `lunatic --node 0.0.0.0:8333 --node-name foo --no-entry`
//
// This example also requires some command line arguments to the runner:
// > cargo build --example environment_remote
// > lunatic --node 0.0.0.0:8334 --node-name bar --peer 0.0.0.0:8333 target/wasm32-wasi/debug/examples/environment_remote.wasm

use lunatic::{lookup, this_process, Config, Environment, Mailbox, Process, Server};

#[lunatic::main]
fn main(m: Mailbox<i32>) {
    let mut config = Config::new(0xA00000000, None);
    config.allow_namespace("");
    let mut env = Environment::new_remote("foo", config).unwrap();
    let module = env.add_this_module().unwrap();

    // Register parent in remote environment. In this case the parent could have been passed to the
    // child as part of the spawn context.
    env.register("parent", "1.0.0", this_process(&m)).unwrap();

    // Spawn child
    let child = module
        .spawn::<Server<(i32, i32), _>, _>((), |_, (a, b)| {
            println!("Adding {} + {}", a, b);
            let parent: Process<i32> = lookup("parent", "^1").unwrap().unwrap();
            // Send back result as message through lookup
            parent.send(a + b);
            // Also send it back as part of the response.
            a + b
        })
        .unwrap();

    let response = child.request((23, 4));
    assert_eq!(response, 27);

    let result = m.receive();
    println!("Adding {} + {} = {}", 23, 4, result);
}
