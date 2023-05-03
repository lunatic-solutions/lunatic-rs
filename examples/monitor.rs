use std::time::Duration;

use lunatic::{Mailbox, MessageSignal, Process, ProcessDiedSignal};

#[lunatic::main]
fn main(mailbox: Mailbox<String>) {
    let mailbox = mailbox.monitorable();

    let process = Process::spawn(mailbox.this(), child_process);
    mailbox.monitor(process);

    loop {
        match mailbox.receive() {
            MessageSignal::Message(msg) => {
                println!("{msg}");
            }
            MessageSignal::Signal(ProcessDiedSignal(id)) => {
                println!("Process {id} died");
                break;
            }
        }
    }
}

fn child_process(parent: Process<String>, _: Mailbox<()>) {
    parent.send("Hello".to_string());
    lunatic::sleep(Duration::from_secs(3));
}
