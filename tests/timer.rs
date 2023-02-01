use std::time::Duration;

use lunatic::ap::handlers::Message;
use lunatic::ap::{AbstractProcess, Config, MessageHandler, ProcessRef, State};
use lunatic::serializer::Bincode;
use lunatic_test::test;

struct P;
impl AbstractProcess for P {
    type Arg = ();
    type State = ProcessRef<Self>;
    type Serializer = Bincode;
    type Handlers = (Message<()>,);
    type StartupError = ();

    fn init(config: Config<Self>, _: ()) -> Result<Self::State, ()> {
        Ok(config.self_ref())
    }
}
impl MessageHandler<()> for P {
    fn handle(state: State<Self>, _: ()) {
        state.self_ref().kill();
    }
}

#[test]
#[should_panic]
fn send_after() {
    let process = P::link().start(()).unwrap();
    process.with_delay(Duration::from_millis(10)).send(());

    // give enough time to fail
    lunatic::sleep(Duration::from_millis(25));
}

#[test]
fn send_after_needs_more_time() {
    let process = P::link().start(()).unwrap();
    process.with_delay(Duration::from_millis(25)).send(());

    // sleep for some time, but not enough for the message to be sent
    lunatic::sleep(Duration::from_millis(10));
}

#[test]
fn cancel_send_after() {
    let process = P::link().start(()).unwrap();
    let timer_ref = process.with_delay(Duration::from_millis(10)).send(());
    timer_ref.cancel();

    // give enough time for the message to be sent if it wasn't canceled
    lunatic::sleep(Duration::from_millis(25));
}
