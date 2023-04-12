use serde::{Deserialize, Serialize};

use crate::host;

use super::{Meter, Span};

/// Represents a Counter in the OpenTelemetry system.
#[derive(Debug, PartialEq, Eq)]
pub struct Counter<'a> {
    meter: &'a Meter,
    id: u64,
    counter_type: CounterType,
}

impl<'a> Counter<'a> {
    /// Add a value to the counter.
    ///
    /// If the counter type is accumulative, then the amount should not be negative.
    pub fn add(&self, amount: impl Into<f64>) -> CounterAddition<'_, 'a, 'static> {
        CounterAddition::new(self, amount)
    }

    /// Get the meter associated with this counter.
    pub fn meter(&self) -> &Meter {
        self.meter
    }

    /// Get the identifier for this counter.
    pub fn id(&self) -> u64 {
        self.id
    }

    /// Get the counter type for this counter.
    pub fn counter_type(&self) -> CounterType {
        self.counter_type
    }

    /// Create a counter from its parts.
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

/// A builder for creating a counter.
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
    /// Create a new instance of `CounterBuilder` with the given meter and name.
    pub fn new(meter: &'m Meter, name: &'a str, counter_type: CounterType) -> Self {
        CounterBuilder {
            meter,
            name,
            counter_type,
            description: None,
            unit: None,
        }
    }

    /// Set the description for the counter.
    pub fn description(mut self, description: &'a str) -> Self {
        self.description = Some(description);
        self
    }

    /// Set the unit for the counter.
    pub fn unit(mut self, unit: &'a str) -> Self {
        self.unit = Some(unit);
        self
    }

    /// Build the counter instance.
    pub fn build(self) -> Counter<'m> {
        let description = self.description.unwrap_or("");
        let unit = self.unit.unwrap_or("");
        let id = self
            .counter_type
            .create(self.meter.id(), self.name, description, unit);
        unsafe { Counter::from_parts(self.meter, id, self.counter_type) }
    }
}

/// An addition to a counter.
#[must_use]
#[derive(Debug, PartialEq)]
pub struct CounterAddition<'c, 'm, 's> {
    counter: &'c Counter<'m>,
    amount: f64,
    parent: Option<&'s Span>,
    attributes: Option<Vec<u8>>,
}

impl<'c, 'm, 's> CounterAddition<'c, 'm, 's> {
    /// Create a new instance of `CounterAddition` with the given counter and amount.
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

    /// Set the parent span for the addition.
    pub fn parent<'a>(self, parent: &'a Span) -> CounterAddition<'c, 'm, 'a> {
        CounterAddition {
            counter: self.counter,
            amount: self.amount,
            parent: Some(parent),
            attributes: self.attributes,
        }
    }

    /// Set the attributes for this addition.
    pub fn attributes<T>(mut self, attributes: &T) -> Result<Self, serde_json::Error>
    where
        T: Serialize,
    {
        let attributes = serde_json::to_vec(attributes)?;
        self.attributes = Some(attributes);
        Ok(self)
    }

    /// Perform the addition.
    pub fn done(self) {
        let parent_id = self.parent.map(|span| span.id()).unwrap_or(u64::MAX);
        let attributes_bytes = self.attributes.unwrap_or(vec![]);
        self.counter
            .counter_type
            .add(parent_id, self.counter.id, self.amount, &attributes_bytes);
    }
}

/// Represents the type of a counter.
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub enum CounterType {
    /// An accumulative counter.
    Accumulative,
    /// An up-down counter.
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