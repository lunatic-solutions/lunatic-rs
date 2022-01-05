use lunatic::{spawn_link, Mailbox, Task};

#[lunatic::main]
fn main(_: Mailbox<()>) {
    let child = spawn_link::<Task<()>, _>((), child).unwrap();
    // Wait on message or death
    child.result();
}

fn child(_: ()) {
    panic!("Error");
}
