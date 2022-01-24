use std::time::Duration;

use lunatic::{sleep, spawn, End, HasDual, Mailbox, Protocol, Send};

#[lunatic::main]
fn main(_: Mailbox<()>) {
    let local = "I'm a string.".to_string();
    let proc = spawn!(@process
        |local, mailbox: Mailbox<String>| println!("{} {}", mailbox.receive(), local));
    proc.send("Hello!".to_string());

    let proc = spawn!(@process |mailbox: Mailbox<String>| println!("{}", mailbox.receive()));
    proc.send("Hello non-capturing closure!".to_string());

    let proc = spawn!(@process function_process);
    proc.send("Hello function!".to_string());

    let proto: Protocol<<Proto as HasDual>::Dual> = spawn!(@protocol function_protocol);
    let (proto, first) = proto.receive();
    let (_, second) = proto.receive();
    assert_eq!("Hello! From a protocol.", format!("{} {}", first, second));

    let local = "I'm a string.".to_string();
    let task = spawn!(@task |local| format!("Hello task! {}", local));
    assert_eq!("Hello task! I'm a string.", task.result());

    spawn!(@background || println!("Hello from the background!"));

    sleep(Duration::from_millis(100));
}

fn function_process(_: (), mailbox: Mailbox<String>) {
    println!("{}", mailbox.receive())
}

type Proto = Send<String, Send<String, End>>;

fn function_protocol(_: (), protocol: Protocol<Proto>) {
    let protocol = protocol.send("Hello!".to_string());
    let _ = protocol.send("From a protocol.".to_string());
}
