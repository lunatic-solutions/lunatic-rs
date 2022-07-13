use std::time::Duration;

use lunatic::{process::process_id, sleep, Mailbox, Process, ProcessConfig};

#[lunatic::main]
fn main(_: Mailbox<()>) {
    // Set process maximum memory to ~1.2Mb & fuel to ~100k instructions
    let mut config = ProcessConfig::new();
    config.set_max_memory(1_500_000);
    config.set_max_fuel(1);

    // This vector allocation will fail and the process will trap
    Process::spawn_config(&config, (), |_, _: Mailbox<()>| {
        println!("Process {} trying to allocate 2MB", process_id());
        let temp: Vec<u8> = vec![0; 2_000_000];
        println!("Succeeded. Value at index 0: {}", temp[0]);
    });

    // This process will fail because it uses too much compute
    Process::spawn_config(&config, (), |_, _: Mailbox<()>| {
        println!("Process {} going into infinite loop", process_id());
        loop {}
    });

    sleep(Duration::from_millis(200));
}
