use lunatic::{process, Mailbox};

#[lunatic::main]
fn main(m: Mailbox<()>) {
    let (this, m) = process::this(m);
    let proc = process::spawn_with(this, |parent, mailbox| {
        let message = mailbox.receive();
        println!("Hello {}", message);
        parent.send(());
    })
    .unwrap();
    proc.send("World!".to_string());
    // Wait for child to finish
    m.receive()
}
