use lunatic::{Mailbox, Process};

#[lunatic::main]
fn main(m: Mailbox<()>) {
    let proc = Process::<String>::spawn_link(m.this(), |parent, mailbox| {
        let message = mailbox.receive();
        println!("Hello {}", message);
        parent.send(());
    });
    proc.send("World!".to_string());
    // Wait for child to finish
    let _ = m.receive();
}
