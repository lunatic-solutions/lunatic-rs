use std::time::Duration;

use lunatic::process::{AbstractProcess, Message, ProcessMessage, ProcessRef, StartProcess};
use lunatic_test::test;

struct P;
impl AbstractProcess for P {
    type Arg = ();
    type State = ProcessRef<Self>;

    fn init(this: ProcessRef<Self>, _: ()) -> Self::State {
        this
    }
}
impl ProcessMessage<()> for P {
    fn handle(this: &mut Self::State, _: ()) {
        this.kill();
    }
}

#[test]
#[should_panic]
fn send_after() {
    let process = P::start_link((), None);
    process.send_after((), Duration::from_millis(10));

    // give enough time to fail
    lunatic::sleep(Duration::from_millis(25));
}

#[test]
fn send_after_needs_more_time() {
    let process = P::start_link((), None);
    process.send_after((), Duration::from_millis(25));

    // sleep for some time, but not enough for the message to be sent
    lunatic::sleep(Duration::from_millis(10));
}

#[test]
fn cancel_send_after() {
    let process = P::start_link((), None);
    let timer_ref = process.send_after((), Duration::from_millis(10));
    timer_ref.cancel();

    // give enough time for the message to be sent if it wasn't canceled
    lunatic::sleep(Duration::from_millis(25));
}
