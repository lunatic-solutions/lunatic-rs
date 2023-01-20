use lunatic::ap::handlers::Request;
use lunatic::ap::{AbstractProcess, Config, RequestHandler, State};
use lunatic::serializer::MessagePack;
use lunatic::Mailbox;

struct Adder;
impl AbstractProcess for Adder {
    type Arg = ();
    type State = Self;
    type Handlers = (Request<(i32, i32)>,);
    type Serializer = MessagePack;
    type StartupError = ();

    fn init(_: Config<Self>, _: ()) -> Result<Adder, ()> {
        Ok(Adder)
    }
}
impl RequestHandler<(i32, i32)> for Adder {
    type Response = i32;

    fn handle(_: State<Self>, (a, b): (i32, i32)) -> i32 {
        a + b
    }
}

#[lunatic::main]
fn main(_: Mailbox<()>) {
    let add_server = Adder::link().start(()).unwrap();
    assert_eq!(add_server.request((1, 1), None).unwrap(), 2);
    assert_eq!(add_server.request((1, 2), None).unwrap(), 3);
}
