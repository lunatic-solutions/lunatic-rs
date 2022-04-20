use std::time::Duration;

use lunatic::{sleep, spawn_link};

fn main() {
    let local = "I'm a string.".to_string();
    let proc =
        spawn_link!(|local, mailbox: Mailbox<String>| println!("{} {}", mailbox.receive(), local));
    proc.send("Hello!".to_string());

    let proc = spawn_link!(|mailbox: Mailbox<String>| println!("{}", mailbox.receive()));
    proc.send("Hello non-capturing closure!".to_string());

    let input = "Hello function!".to_string();
    let _ = spawn_link!(function_process(input));

    sleep(Duration::from_millis(100));
}

fn function_process(input: String) {
    assert_eq!(input, "Hello function");
}
