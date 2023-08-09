use crate::host;

use super::{CounterBuilder, CounterType, HistogramBuilder};

/// Represents a meter in the OpenTelemetry system.
#[derive(Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Meter {
    id: u64,
}

impl Meter {
    /// Create a new meter with the given `name`.
    pub fn new(name: &str) -> Self {
        let id = unsafe { host::api::metrics::meter(name.as_ptr(), name.len()) };
        Meter { id }
    }

    /// Get the identifier for this meter.
    pub fn id(&self) -> u64 {
        self.id
    }

    /// Create a `Meter` from a given identifier.
    pub const unsafe fn from_id(id: u64) -> Self {
        Meter { id }
    }

    /// Create a new cumulative counter with the given `name` on this meter.
    pub fn counter<'a, 'm>(&'m self, name: &'a str) -> CounterBuilder<'a, 'm> {
        CounterBuilder::new(self, name, CounterType::Accumulative)
    }

    /// Create a new up-down counter with the given `name` on this meter.
    pub fn up_down_counter<'a, 'm>(&'m self, name: &'a str) -> CounterBuilder<'a, 'm> {
        CounterBuilder::new(self, name, CounterType::UpDown)
    }

    /// Create a new histogram with the given `name` on this meter.
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
