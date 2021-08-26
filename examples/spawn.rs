use lunatic::{process, Mailbox};

#[lunatic::main]
fn main(m: Mailbox<()>) {
    let (this, m) = process::this(m);
    process::spawn_with(this, |parent, _: Mailbox<()>| {
        println!("Hello world from a process!");
        parent.send(());
    })
    .unwrap();
    // Wait for child to finish
    let _ignore = m.receive();
}
