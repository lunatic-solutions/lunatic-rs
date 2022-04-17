use std::time::Duration;

use lunatic::{Mailbox, Process};
use lunatic_test::test;

#[test]
fn spawn_non_capturing_child() {
    Process::spawn((), |_, _: Mailbox<()>| {});
}

#[test]
fn wait_for_child(m: Mailbox<()>) {
    Process::spawn(m.this(), |parent, _: Mailbox<()>| parent.send(()));
    m.receive();
}

#[test]
fn failing_child_doesnt_kill_parent() {
    Process::spawn((), |_, _: Mailbox<()>| panic!("panics"));
    // Give a chance for the kill to propagate.
    lunatic::sleep(Duration::from_millis(100));
}

#[test]
#[should_panic]
fn failing_child_kills_linked_parent() {
    Process::spawn_link((), |_, _: Mailbox<()>| panic!("panics"));
    // Give a chance for the kill to propagate.
    lunatic::sleep(Duration::from_millis(100));
}

#[test]
fn parent_and_child_exchange_messages(parent_mailbox: Mailbox<i32>) {
    let parent = parent_mailbox.this();
    let adder = Process::spawn_link(parent, |parent, child_mailbox: Mailbox<i32>| {
        let a = child_mailbox.receive();
        let b = child_mailbox.receive();
        parent.send(a + b)
    });

    adder.send(2);
    adder.send(2);
    assert_eq!(4, parent_mailbox.receive());
}

#[test]
fn mailbox_timeout(m: Mailbox<i32>) {
    let message = m.receive_timeout(Duration::from_millis(10));
    assert!(message.is_err());
}

#[test]
fn recursive_count(mailbox: Mailbox<i32>) {
    Process::spawn_link((mailbox.this(), 1000), recursive_count_sub);
    assert_eq!(500500, mailbox.receive());
}

fn recursive_count_sub((parent, n): (Process<i32>, i32), mailbox: Mailbox<i32>) {
    if n > 0 {
        Process::spawn_link((mailbox.this(), n - 1), recursive_count_sub);
        parent.send(n + mailbox.receive());
    } else {
        parent.send(0);
    }
}
