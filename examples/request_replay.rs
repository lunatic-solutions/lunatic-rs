use lunatic::{process, Mailbox, Request};

#[lunatic::main]
fn main(_: Mailbox<()>) {
    // Spawn a process that gets two numbers as a request and can reply to the sender with the sum
    // of the numbers.
    let add_server = process::spawn(|mailbox: Mailbox<Request<(i32, i32), i32>>| loop {
        let request = mailbox.receive().unwrap();
        let (a, b) = *request.data();
        request.reply(a + b);
    })
    .unwrap();
    // Make specific requests to the `add_server` & ignore all messages in the mailbox that are not
    // responses to the request.
    assert_eq!(add_server.request((1, 1)).unwrap(), 2);
    assert_eq!(add_server.request((1, 2)).unwrap(), 3);
}
