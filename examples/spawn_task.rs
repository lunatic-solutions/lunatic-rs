use lunatic::{spawn_link, Mailbox};

#[lunatic::main]
fn main(_: Mailbox<()>) {
    let child = spawn_link!(@task || {
        println!("Hello world from a process!");
    });
    // Wait for child to finish
    let _ = child.result();
}
