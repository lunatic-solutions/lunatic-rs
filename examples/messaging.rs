use lunatic::{spawn, this_process, Mailbox, Process};

#[lunatic::main]
fn main(m: Mailbox<()>) {
    let this = this_process(&m);
    let proc = spawn::<Process<String>, _>(this, |parent, mailbox| {
        let message = mailbox.receive();
        println!("Hello {}", message);
        parent.send(());
    })
    .unwrap();
    proc.send("World!".to_string());
    // Wait for child to finish
    let _ignore = m.receive();
}
