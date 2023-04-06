use serde::{Deserialize, Serialize};

use crate::host;

use super::{Meter, Span};

#[derive(Debug, PartialEq, Eq)]
pub struct Counter<'a> {
    meter: &'a Meter,
    id: u64,
    counter_type: CounterType,
}

impl<'a> Counter<'a> {
    pub fn add(&self, amount: impl Into<f64>) -> CounterAddition<'_, 'a, 'static> {
        CounterAddition::new(self, amount)
    }

    pub fn meter(&self) -> &Meter {
        self.meter
    }

    pub fn id(&self) -> u64 {
        self.id
    }

    pub fn counter_type(&self) -> CounterType {
        self.counter_type
    }

    pub unsafe fn from_parts(meter: &Meter, id: u64, counter_type: CounterType) -> Counter<'_> {
        Counter {
            meter,
            id,
            counter_type,
        }
    }
}

impl<'a> Drop for Counter<'a> {
    fn drop(&mut self) {
        self.counter_type.drop(self.id)
    }
}

#[must_use]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CounterBuilder<'a, 'm> {
    meter: &'m Meter,
    name: &'a str,
    counter_type: CounterType,
    description: Option<&'a str>,
    unit: Option<&'a str>,
}

impl<'a, 'm> CounterBuilder<'a, 'm> {
    pub fn new(meter: &'m Meter, name: &'a str, counter_type: CounterType) -> Self {
        CounterBuilder {
            meter,
            name,
            counter_type,
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
        let id = self
            .counter_type
            .create(self.meter.id(), self.name, description, unit);
        unsafe { Counter::from_parts(self.meter, id, self.counter_type) }
    }
}

#[must_use]
#[derive(Debug, PartialEq)]
pub struct CounterAddition<'c, 'm, 's> {
    counter: &'c Counter<'m>,
    amount: f64,
    parent: Option<&'s Span>,
    attributes: Option<Vec<u8>>,
}

impl<'c, 'm, 's> CounterAddition<'c, 'm, 's> {
    pub fn new(
        counter: &'c Counter<'m>,
        amount: impl Into<f64>,
    ) -> CounterAddition<'c, 'm, 'static> {
        CounterAddition {
            counter,
            amount: amount.into(),
            parent: None,
            attributes: None,
        }
    }

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
        self.counter
            .counter_type
            .add(parent_id, self.counter.id, self.amount, &attributes_bytes);
    }
}

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub enum CounterType {
    Accumulative,
    UpDown,
}

impl CounterType {
    fn create(&self, meter: u64, name: &str, description: &str, unit: &str) -> u64 {
        unsafe {
            match self {
                CounterType::Accumulative => host::api::metrics::counter(
                    meter,
                    name.as_ptr(),
                    name.len(),
                    description.as_ptr(),
                    description.len(),
                    unit.as_ptr(),
                    unit.len(),
                ),
                CounterType::UpDown => host::api::metrics::up_down_counter(
                    meter,
                    name.as_ptr(),
                    name.len(),
                    description.as_ptr(),
                    description.len(),
                    unit.as_ptr(),
                    unit.len(),
                ),
            }
        }
    }

    fn add(&self, span: u64, counter: u64, amount: f64, attributes: &[u8]) {
        unsafe {
            match self {
                CounterType::Accumulative => host::api::metrics::counter_add(
                    span,
                    counter,
                    amount,
                    attributes.as_ptr(),
                    attributes.len(),
                ),
                CounterType::UpDown => host::api::metrics::up_down_counter_add(
                    span,
                    counter,
                    amount,
                    attributes.as_ptr(),
                    attributes.len(),
                ),
            }
        }
    }

    fn drop(&self, id: u64) {
        unsafe {
            match self {
                CounterType::Accumulative => host::api::metrics::counter_drop(id),
                CounterType::UpDown => host::api::metrics::up_down_counter_drop(id),
            }
        }
    }
}
