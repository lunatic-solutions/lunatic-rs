// To run this example you will need to have a remote node under the name "foo".
// You can start one with: `lunatic --node 0.0.0.0:8333 --node-name foo --no-entry`
//
// This example also requires some command line arguments to the runner:
// > cargo build --example environment_remote
// > lunatic --node 0.0.0.0:8334 --node-name bar --peer 0.0.0.0:8333 target/wasm32-wasi/debug/examples/environment_remote.wasm

use lunatic::{lookup, process, Config, Environment, Mailbox};

#[lunatic::main]
fn main(m: Mailbox<i64>) {
    let mut config = Config::new(0xA00000000, None);
    config.allow_namespace("");
    let mut env = Environment::new_remote("foo", config).unwrap();
    let module = env.add_this_module().unwrap();

    // Register parent in remote environment. In this case the parent could have been passed to the
    // child as part of the spawn context.
    env.register("parent", "1.0.0", process::this(&m)).unwrap();

    // Spawn child
    let child = module
        .spawn(|mailbox: Mailbox<(i64, i64)>| {
            let (a, b) = mailbox.receive().unwrap();
            println!("Adding {} + {}", a, b);
            let parent = lookup("parent", "^1").unwrap().unwrap();
            parent.send(a + b);
        })
        .unwrap();

    child.send((23, 4));
    let result = m.receive().unwrap();
    println!("Adding {} + {} = {}", 23, 4, result);
}
