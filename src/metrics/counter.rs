use serde::Serialize;

use crate::host;

use super::{Meter, Span};

#[derive(Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Counter<'a> {
    meter: &'a Meter,
    id: u64,
}

impl<'a> Counter<'a> {
    pub fn add(&self, amount: impl Into<f64>) -> CounterAddition<'_, 'a, 'static> {
        CounterAddition {
            counter: self,
            amount: amount.into(),
            parent: None,
            attributes: None,
        }
    }

    pub fn meter(&self) -> &Meter {
        self.meter
    }

    pub fn id(&self) -> u64 {
        self.id
    }

    pub unsafe fn from_parts(meter: &Meter, id: u64) -> Counter<'_> {
        Counter { meter, id }
    }
}

impl<'a> Drop for Counter<'a> {
    fn drop(&mut self) {
        unsafe {
            host::api::metrics::drop_counter(self.id);
        }
    }
}

#[must_use]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CounterBuilder<'a, 'm> {
    meter: &'m Meter,
    name: &'a str,
    description: Option<&'a str>,
    unit: Option<&'a str>,
}

impl<'a, 'm> CounterBuilder<'a, 'm> {
    pub fn new(meter: &'m Meter, name: &'a str) -> Self {
        CounterBuilder {
            meter,
            name,
            description: None,
            unit: None,
        }
    }

    pub fn description(mut self, description: &'a str) -> Self {
        self.description = Some(description);
        self
    }

    pub fn unit(mut self, unit: &'a str) -> Self {
        self.unit = Some(unit);
        self
    }

    pub fn build(self) -> Counter<'m> {
        let description = self.description.unwrap_or("");
        let unit = self.unit.unwrap_or("");
        let id = unsafe {
            host::api::metrics::counter(
                self.meter.id(),
                self.name.as_ptr(),
                self.name.len(),
                description.as_ptr(),
                description.len(),
                unit.as_ptr(),
                unit.len(),
            )
        };
        Counter {
            meter: self.meter,
            id,
        }
    }
}

#[must_use]
#[derive(Debug)]
pub struct CounterAddition<'c, 'm, 's> {
    counter: &'c Counter<'m>,
    amount: f64,
    parent: Option<&'s Span>,
    attributes: Option<Vec<u8>>,
}

impl<'c, 'm, 's> CounterAddition<'c, 'm, 's> {
    pub fn parent<'a>(self, parent: &'a Span) -> CounterAddition<'c, 'm, 'a> {
        CounterAddition {
            counter: self.counter,
            amount: self.amount,
            parent: Some(parent),
            attributes: self.attributes,
        }
    }

    pub fn attributes<T>(mut self, attributes: &T) -> Result<Self, serde_json::Error>
    where
        T: Serialize,
    {
        let attributes = serde_json::to_vec(attributes)?;
        self.attributes = Some(attributes);
        Ok(self)
    }

    pub fn done(self) {
        let parent_id = self.parent.map(|span| span.id()).unwrap_or(u64::MAX);
        let attributes_bytes = self.attributes.unwrap_or(vec![]);
        unsafe {
            host::api::metrics::increment_counter(
                parent_id,
                self.counter.id,
                self.amount,
                attributes_bytes.as_ptr(),
                attributes_bytes.len(),
            )
        };
    }
}
