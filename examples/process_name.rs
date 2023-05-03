use std::time::Duration;

use lunatic::{spawn, Process, ProcessName};

#[derive(ProcessName)]
struct MyProcess;

fn main() {
    let process = spawn!(|mailbox: Mailbox<()>| {
        mailbox.receive();
        println!("Received message!");
    });
    process.register(&MyProcess);

    spawn!(|| {
        let process = Process::<()>::lookup(&MyProcess).unwrap();
        process.send(());
    });

    lunatic::sleep(Duration::from_millis(50));
}
