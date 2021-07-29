use lunatic::{process, Mailbox};

fn main() {
    process::spawn(|_: Mailbox<()>| {
        println!("Hello world from a process!");
    })
    .unwrap()
    .join()
    .unwrap();
}
