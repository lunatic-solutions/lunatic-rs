use lunatic::{process, Mailbox};

#[test]
fn spawn_link_test() {
    process::spawn(|m: Mailbox<()>| {
        let (_child, m) = process::spawn_link(m, |_: Mailbox<()>| panic!()).unwrap();
        // The child failure is captured as a message
        assert_eq!(m.receive().is_err(), true);
    })
    .unwrap()
    .join()
    .unwrap();
}

#[test]
fn spawn_link_unwrap_test() {
    let parent = process::spawn(|m: Mailbox<()>| {
        let (_child, m) = process::spawn_link_unwrap(m, |_: Mailbox<()>| panic!()).unwrap();
        // Will block until signal is received from child and this process fails
        assert_eq!(m.receive(), ());
    })
    .unwrap()
    .join();
    // Parent fails because of link
    assert_eq!(parent.is_err(), true);
}
