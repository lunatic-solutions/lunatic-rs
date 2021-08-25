use lunatic::{process, Mailbox};

#[lunatic::main]
fn main(mailbox: Mailbox<()>) {
    let (_child, mailbox) = process::spawn_link(mailbox, child).unwrap();
    // Wait on message or death
    assert_eq!(mailbox.receive().is_err(), true);
}

fn child(_: Mailbox<()>) {
    panic!("Error");
}
