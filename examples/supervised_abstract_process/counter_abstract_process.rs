use lunatic::ap::Config;
use lunatic::{abstract_process, Tag};

pub struct Counter(u32);

/// Abstract process using abstract_process macro.
/// `visibility = pub` Makes the generated traits public and usable.
#[abstract_process(visibility = pub)]
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
