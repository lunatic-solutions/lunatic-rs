use std::time::Duration;

use lunatic::{sleep, Mailbox, Process, ProcessConfig};

#[lunatic::main]
fn main(_: Mailbox<()>) {
    // Set process maximum memory to ~1.2Mb & fuel to ~100k instructions
    let mut config = ProcessConfig::new();
    config.set_max_memory(1_200_000);
    config.set_max_fuel(1);

    // This vector allocation will fail and the process will trap
    Process::spawn_config(&config, (), |_, _: Mailbox<()>| {
        vec![0; 150_000];
    });

    // This process will fail because it uses too much compute
    Process::spawn_config(&config, (), |_, _: Mailbox<()>| loop {});

    // Sleep for 1 sec
    sleep(Duration::from_millis(200));
}
