use lunatic::{Config, Environment, Mailbox};

fn main() {
    // Create a new environment where processes can use maximum 17 Wasm pages of
    // memory (17 * 64KB) & 1 gallon of instructions (~=100k CPU cycles).
    let config = Config::new(17, Some(1));
    // Allow only syscalls under the "wasi_snapshot_preview1::environ*" namespace
    config.allow_namespace("wasi_snapshot_preview1::environ");
    let env = Environment::new(config).unwrap();
    let module = env.add_this_module().unwrap();

    // This process will fail because it can't uses syscalls for std i/o
    let proc = module
        .spawn(|_: Mailbox<()>| {
            println!("Hi from different env");
        })
        .unwrap()
        .join();
    assert_eq!(proc.is_err(), true);

    // This process will fail because it uses too much memory
    let proc = module
        .spawn(|_: Mailbox<()>| {
            vec![0; 15_000];
        })
        .unwrap()
        .join();
    assert_eq!(proc.is_err(), true);

    // This process will fail because it uses too much compute
    let proc = module
        .spawn(|_: Mailbox<()>| loop {
            let _ = 1 + 1;
        })
        .unwrap()
        .join();
    assert_eq!(proc.is_err(), true);
}
