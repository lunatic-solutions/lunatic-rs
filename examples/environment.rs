use lunatic::{Config, Environment, Mailbox};

#[lunatic::main]
fn main(m: Mailbox<()>) {
    // Create a new environment where processes can use maximum 17 Wasm pages of
    // memory (17 * 64KB) & 1 gallon of instructions (~=100k CPU cycles).
    let mut config = Config::new(1_200_000, Some(1));
    // Allow only syscalls under the "wasi_snapshot_preview1::environ*" namespace
    config.allow_namespace("wasi_snapshot_preview1::environ");
    let mut env = Environment::new(config).unwrap();
    let module = env.add_this_module().unwrap();

    // This process will fail because it can't uses syscalls for std i/o
    let (_, m) = module
        .spawn_link(m, |_: Mailbox<()>| println!("Hi from different env"))
        .unwrap();
    assert!(m.receive().is_err());

    // This process will fail because it uses too much memory
    let (_, m) = module
        .spawn_link(m, |_: Mailbox<()>| {
            vec![0; 150_000];
        })
        .unwrap();
    assert!(m.receive().is_err());

    // This process will fail because it uses too much compute
    let (_, m) = module.spawn_link(m, |_: Mailbox<()>| loop {}).unwrap();
    assert!(m.receive().is_err());
}
