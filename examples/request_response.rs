use lunatic::{Mailbox, Request, Server, ServerRequest, StartServer};

struct Adder;
impl Server for Adder {
    type Arg = ();
    type State = Self;

    fn init(_: ()) -> Adder {
        Adder
    }
}
impl ServerRequest<(i32, i32)> for Adder {
    type Response = i32;

    fn handle(&mut self, (a, b): (i32, i32)) -> i32 {
        a + b
    }
}

#[lunatic::main]
fn main(_: Mailbox<()>) {
    let add_server = Adder::start((), None);
    assert_eq!(add_server.request((1, 1)), 2);
    assert_eq!(add_server.request((1, 2)), 3);
}
