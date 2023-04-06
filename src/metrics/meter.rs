use crate::host;

use super::{CounterBuilder, CounterType, HistogramBuilder};

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
        CounterBuilder::new(self, name, CounterType::Accumulative)
    }

    pub fn up_down_counter<'a, 'm>(&'m self, name: &'a str) -> CounterBuilder<'a, 'm> {
        CounterBuilder::new(self, name, CounterType::UpDown)
    }

    pub fn histogram<'a, 'm>(&'m self, name: &'a str) -> HistogramBuilder<'a, 'm> {
        HistogramBuilder::new(self, name)
    }
}

impl Drop for Meter {
    fn drop(&mut self) {
        unsafe {
            host::api::metrics::meter_drop(self.id);
        }
    }
}
