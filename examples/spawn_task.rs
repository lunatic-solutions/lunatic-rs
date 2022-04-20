use lunatic::{Mailbox, Task};

#[lunatic::main]
fn main(_: Mailbox<()>) {
    let child = Task::spawn_link((), |_| {
        println!("Hello world from a process!");
    });
    // Wait for child to finish
    let _ = child.result();
}
