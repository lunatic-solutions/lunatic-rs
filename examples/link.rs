use lunatic::{process, Mailbox};

#[lunatic::main]
fn main(mailbox: Mailbox<()>) {
    let (_child, _tag, mailbox) = process::spawn_link(mailbox, child).unwrap();
    // Wait on message or death
    assert!(mailbox.receive().is_signal());
}

fn child(_: Mailbox<()>) {
    panic!("Error");
}
