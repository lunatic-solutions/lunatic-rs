use lunatic::{spawn, Mailbox, Task};

#[lunatic::main]
fn main(_: Mailbox<()>) {
    let child = spawn::<Task<()>, _>((), |_| {
        println!("Hello world from a process!");
    })
    .unwrap();
    // Wait for child to finish
    let _ignore = child.result();
}
