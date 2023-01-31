use std::time::Duration;

use lunatic::ap::{AbstractProcess, Config, DeferredResponse};
use lunatic::{abstract_process, Mailbox, Tag};

struct Counter(u32);

#[abstract_process]
impl Counter {
    #[init]
    fn init(_: Config<Self>, start: u32) -> Result<Self, ()> {
        Ok(Self(start))
    }

    #[terminate]
    fn terminate(self) {
        println!("Shutdown process");
    }

    #[handle_link_death]
    fn handle_link_death(&self, _tag: Tag) {
        println!("Link trapped");
    }

    #[handle_message]
    fn increment(&mut self) {
        self.0 += 1;
    }

    #[handle_request]
    fn count(&self) -> u32 {
        self.0
    }

    #[handle_deferred_request]
    fn add_to_count(&self, a: u32, b: u32, dr: DeferredResponse<u32, Self>) {
        dr.send_response(self.0 + a + b)
    }
}

#[lunatic::main]
fn main(_: Mailbox<()>) {
    let counter = Counter::link().start(0).unwrap();
    assert_eq!(counter.count(), 0);

    counter.increment();
    assert_eq!(counter.count(), 1);

    counter.increment();
    assert_eq!(counter.count(), 2);

    assert_eq!(
        counter
            .with_timeout(Duration::from_millis(10))
            .add_to_count(1, 1)
            .unwrap(),
        4
    );
}
