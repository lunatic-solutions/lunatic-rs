use std::time::Duration;

use lunatic::host::api::message::receive;
use lunatic::host::api::process::die_when_link_dies;
use lunatic::{spawn_link, Mailbox, Process, ProcessConfig};
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
    assert!(message.unwrap_err().is_timed_out());
}

#[test]
fn recursive_count(mailbox: Mailbox<i32>) {
    let mut config = ProcessConfig::new().unwrap();
    config.set_can_spawn_processes(true);
    Process::spawn_link_config(&config, (mailbox.this(), 1000), recursive_count_sub);
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

#[test]
fn lookup(mailbox: Mailbox<i32>) {
    // Register self under name "hello"
    let this = mailbox.this();
    this.register("hello");

    spawn_link!(|| {
        let parent = Process::<i32>::lookup("hello").unwrap();
        parent.send(1337);
    });

    assert_eq!(1337, mailbox.receive());
}

#[test]
fn spawn_config_doesnt_link() {
    let mut config = ProcessConfig::new().unwrap();
    config.set_max_memory(5_000_000);
    config.set_can_spawn_processes(true);

    Process::spawn_config(&config, (), |_, _: Mailbox<()>| panic!());

    // Give enough time to fail
    lunatic::sleep(Duration::from_millis(500));
}

#[test]
#[should_panic]
fn spawn_link_config_does_link() {
    let mut config = ProcessConfig::new().unwrap();
    config.set_max_memory(5_000_000);
    config.set_can_spawn_processes(true);

    Process::spawn_link_config(&config, (), |_, _: Mailbox<()>| panic!());

    // Give enough time to fail
    lunatic::sleep(Duration::from_millis(100));
}

#[test]
#[should_panic]
fn kill_process() {
    let process = Process::spawn_link((), |_, _: Mailbox<()>| {});
    process.kill();

    // Give enough time to fail
    lunatic::sleep(Duration::from_millis(100));
}

#[test]
fn unlink_shouldnt_fail_on_dead_process() {
    let child = Process::spawn_link((), |_, _: Mailbox<()>| {});
    // Give enough time for process to finish
    lunatic::sleep(Duration::from_millis(100));
    child.unlink();
}

#[test]
fn link_should_trigger_on_dead_process() {
    unsafe { die_when_link_dies(0) };
    let child = Process::spawn((), |_, _: Mailbox<()>| {});
    // Give enough time for process to finish
    lunatic::sleep(Duration::from_millis(100));
    child.link();
    let result = unsafe { receive(&0i64 as *const i64, 0, 100) };
    // Confirm it's a link broke message and not a timeout
    assert_ne!(result, 9027);
}

#[test]
fn is_alive() {
    let child = Process::spawn((), |_, _: Mailbox<()>| {
        lunatic::sleep(Duration::from_millis(100));
    });
    // Give enough time for process to be spawned
    lunatic::sleep(Duration::from_millis(10));
    assert_eq!(child.is_alive(), true);
    // Give enough time to process to finish
    lunatic::sleep(Duration::from_millis(150));
    assert_eq!(child.is_alive(), false);
}
