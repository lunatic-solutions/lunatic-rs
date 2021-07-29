use lunatic::process;

fn main() {
    let proc = process::spawn(|mailbox| {
        let message = mailbox.receive();
        println!("Hello {}", message);
    })
    .unwrap();

    proc.send("World!".to_string());
    proc.join().unwrap();
}
