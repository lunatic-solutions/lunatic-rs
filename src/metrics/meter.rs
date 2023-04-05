use crate::host;

use super::CounterBuilder;

#[derive(Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Meter {
    id: u64,
}

impl Meter {
    pub fn new(name: &str) -> Self {
        let id = unsafe { host::api::metrics::meter(name.as_ptr(), name.len()) };
        Meter { id }
    }

    pub fn id(&self) -> u64 {
        self.id
    }

    pub const unsafe fn from_id(id: u64) -> Self {
        Meter { id }
    }

    pub fn counter<'a, 'm>(&'m self, name: &'a str) -> CounterBuilder<'a, 'm> {
        CounterBuilder::new(self, name)
    }
}

impl Drop for Meter {
    fn drop(&mut self) {
        unsafe {
            host::api::metrics::drop_meter(self.id);
        }
    }
}
