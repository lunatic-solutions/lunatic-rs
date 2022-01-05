use lunatic::{spawn, Mailbox, Server};

#[lunatic::main]
fn main(_: Mailbox<()>) {
    // Spawn a process that gets two numbers as a request and can reply to the sender with the sum
    // of the numbers.
    let add_server = spawn::<Server<(i32, i32), _>, _>((), |_, (a, b)| a + b).unwrap();
    // Make specific requests to the `add_server` & ignore all messages in the mailbox that are not
    // responses to the request.
    assert_eq!(add_server.request((1, 1)), 2);
    assert_eq!(add_server.request((1, 2)), 3);
}
