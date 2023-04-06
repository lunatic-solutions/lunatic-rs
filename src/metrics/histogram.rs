use serde::Serialize;

use crate::host;

use super::{Meter, Span};

#[derive(Debug, PartialEq, Eq)]
pub struct Histogram<'a> {
    meter: &'a Meter,
    id: u64,
}

impl<'a> Histogram<'a> {
    pub fn record(&self, value: impl Into<f64>) -> HistogramRecord<'_, 'a, 'static> {
        HistogramRecord::new(self, value)
    }

    pub fn meter(&self) -> &Meter {
        self.meter
    }

    pub fn id(&self) -> u64 {
        self.id
    }

    pub unsafe fn from_parts(meter: &Meter, id: u64) -> Histogram<'_> {
        Histogram { meter, id }
    }
}

impl<'a> Drop for Histogram<'a> {
    fn drop(&mut self) {
        unsafe {
            host::api::metrics::histogram_drop(self.id);
        }
    }
}

#[must_use]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct HistogramBuilder<'a, 'm> {
    meter: &'m Meter,
    name: &'a str,
    description: Option<&'a str>,
    unit: Option<&'a str>,
}

impl<'a, 'm> HistogramBuilder<'a, 'm> {
    pub fn new(meter: &'m Meter, name: &'a str) -> Self {
        HistogramBuilder {
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

    pub fn build(self) -> Histogram<'m> {
        let description = self.description.unwrap_or("");
        let unit = self.unit.unwrap_or("");
        unsafe {
            let id = host::api::metrics::histogram(
                self.meter.id(),
                self.name.as_ptr(),
                self.name.len(),
                description.as_ptr(),
                description.len(),
                unit.as_ptr(),
                unit.len(),
            );
            Histogram::from_parts(self.meter, id)
        }
    }
}

#[must_use]
#[derive(Debug, PartialEq)]
pub struct HistogramRecord<'c, 'm, 's> {
    histogram: &'c Histogram<'m>,
    value: f64,
    parent: Option<&'s Span>,
    attributes: Option<Vec<u8>>,
}

impl<'c, 'm, 's> HistogramRecord<'c, 'm, 's> {
    pub fn new(
        histogram: &'c Histogram<'m>,
        value: impl Into<f64>,
    ) -> HistogramRecord<'c, 'm, 'static> {
        HistogramRecord {
            histogram,
            value: value.into(),
            parent: None,
            attributes: None,
        }
    }

    pub fn parent<'a>(self, parent: &'a Span) -> HistogramRecord<'c, 'm, 'a> {
        HistogramRecord {
            histogram: self.histogram,
            value: self.value,
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
            host::api::metrics::histogram_record(
                parent_id,
                self.histogram.id,
                self.value,
                attributes_bytes.as_ptr(),
                attributes_bytes.len(),
            );
        }
    }
}
