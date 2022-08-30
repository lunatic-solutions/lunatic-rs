use lunatic::process::{ProcessRef, StartProcess};
use lunatic::{abstract_process, Mailbox, Tag};

struct Counter(u32);

#[abstract_process]
impl Counter {
    #[init]
    fn init(_: ProcessRef<Self>, start: u32) -> Self {
        Self(start)
    }

    #[terminate]
    fn terminate(self) {
        println!("Shutdown process");
    }

    #[handle_link_trapped]
    fn handle_link_trapped(&self, _tag: Tag) {
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
}

#[lunatic::main]
fn main(_: Mailbox<()>) {
    let counter = Counter::start_link(0, None);
    assert_eq!(counter.count(), 0);

    counter.increment();
    assert_eq!(counter.count(), 1);

    counter.increment();
    assert_eq!(counter.count(), 2);
}
