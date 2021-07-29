use lunatic::{process, Mailbox};

fn main() {
    process::spawn(|mailbox: Mailbox<()>| {
        let (_child, mailbox) = process::spawn_link(mailbox, child).unwrap();
        // Wait on message
        assert_eq!(mailbox.receive().is_err(), true);
    })
    .unwrap()
    .join()
    .unwrap();
}

fn child(_: Mailbox<()>) {
    panic!("Error");
}
