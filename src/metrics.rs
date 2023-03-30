//! Expose [metrics](https://crates.io/metrics) functions from lunatic's runtime
//! Lunatic's runtime comes with metrics integration and can expose those metrics
//! to prometheus if prometheus feature is enable at build time and using --prometheus
//! flag to start the exporter
//!
//! All this functions are similar to the macros defined in [metrics docs](https://docs.rs/metrics/latest/metrics/index.html#emission)

use std::{
    collections::BTreeMap,
    fmt,
    time::{SystemTime, UNIX_EPOCH},
};

pub use log::Level;
use serde::{ser::SerializeMap, Serialize};

use crate::host::{self, api::metrics};

mod macros;

#[derive(Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Span {
    id: u64,
}

impl Span {
    pub fn new<T>(name: &str, attributes: Option<&T>) -> Result<Self, serde_json::Error>
    where
        T: Serialize + 'static,
    {
        Self::new_(None, name, attributes)
    }

    pub fn new_with_parent<T>(
        parent: &Span,
        name: &str,
        attributes: Option<&T>,
    ) -> Result<Self, serde_json::Error>
    where
        T: Serialize + 'static,
    {
        Self::new_(Some(parent), name, attributes)
    }

    fn new_<T>(
        parent: Option<&Span>,
        name: &str,
        attributes: Option<&T>,
    ) -> Result<Self, serde_json::Error>
    where
        T: Serialize + 'static,
    {
        let parent_id = parent.map(|span| span.id).unwrap_or(u64::MAX);
        let attributes_bytes = match attributes {
            Some(attributes) => serde_json::to_vec(attributes)?,
            None => vec![],
        };
        let id = unsafe {
            host::api::metrics::start_span(
                parent_id,
                name.as_ptr(),
                name.len(),
                attributes_bytes.as_ptr(),
                attributes_bytes.len(),
            )
        };

        let span = Span { id };

        Ok(span)
    }

    pub fn add_event<T>(&self, name: &str, attributes: Option<&T>) -> Result<(), serde_json::Error>
    where
        T: Serialize + 'static,
    {
        add_event(Some(self.id), name, attributes)
    }
}

impl Drop for Span {
    fn drop(&mut self) {
        unsafe {
            host::api::metrics::drop_span(self.id);
        }
    }
}

pub fn add_event<T>(
    span: Option<u64>,
    name: &str,
    attributes: Option<&T>,
) -> Result<(), serde_json::Error>
where
    T: Serialize,
{
    let attributes_bytes = match attributes {
        Some(attributes) => serde_json::to_vec(attributes)?,
        None => vec![],
    };
    unsafe {
        host::api::metrics::add_event(
            span.unwrap_or(u64::MAX),
            name.as_ptr(),
            name.len(),
            attributes_bytes.as_ptr(),
            attributes_bytes.len(),
        )
    }

    Ok(())
}

/// Sets a counter
pub fn counter(name: &str, value: u64) {
    unsafe { metrics::counter(name.as_ptr(), name.len(), value) }
}

/// Increments a counter
pub fn increment_counter(name: &str) {
    unsafe { metrics::increment_counter(name.as_ptr(), name.len()) }
}

/// Sets a gauge
pub fn gauge(name: &str, value: f64) {
    unsafe { metrics::gauge(name.as_ptr(), name.len(), value) }
}

/// Increments a gauge
pub fn increment_gauge(name: &str, value: f64) {
    unsafe { metrics::increment_gauge(name.as_ptr(), name.len(), value) }
}

/// Decrements a gauge
pub fn decrement_gauge(name: &str, value: f64) {
    unsafe { metrics::decrement_gauge(name.as_ptr(), name.len(), value) }
}

/// Sets a histogram
pub fn histogram(name: &str, value: f64) {
    unsafe { metrics::histogram(name.as_ptr(), name.len(), value) }
}

#[derive(Clone, Debug)]
pub struct Attributes<'a> {
    target: &'static str,
    level: Level,
    message: fmt::Arguments<'a>,
    file: &'static str,
    line: u32,
    column: u32,
    module_path: &'static str,
    attributes: BTreeMap<&'static str, serde_json::Value>,
    timestamp: SystemTime,
}

impl<'a> Attributes<'a> {
    #[inline]
    pub fn new(
        target: &'static str,
        level: Level,
        message: fmt::Arguments<'a>,
        file: &'static str,
        line: u32,
        column: u32,
        module_path: &'static str,
        attributes: BTreeMap<&'static str, serde_json::Value>,
    ) -> Self {
        Attributes {
            target,
            level,
            message,
            file,
            line,
            column,
            module_path,
            attributes,
            timestamp: SystemTime::now(),
        }
    }
}

impl<'a> Serialize for Attributes<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let (severity_number, severity_text) = level_severity(&self.level);
        let timestamp = self
            .timestamp
            .duration_since(UNIX_EPOCH)
            .map_err(|_| serde::ser::Error::custom("Time went backwards"))?;

        let message_is_empty = self.message.as_str().map(|s| s.is_empty()).unwrap_or(false);
        let mut map = serializer.serialize_map(Some(9 + message_is_empty as usize))?;
        if !message_is_empty {
            map.serialize_entry("body", &self.message)?;
        }
        map.serialize_entry("target", &self.target)?;
        map.serialize_entry("severityNumber", &severity_number)?;
        map.serialize_entry("severityText", &severity_text)?;
        map.serialize_entry("code.filepath", &self.file)?;
        map.serialize_entry("code.lineno", &self.line)?;
        map.serialize_entry("code.column", &self.column)?;
        map.serialize_entry("code.namespace", &self.module_path)?;
        map.serialize_entry("attributes", &self.attributes)?;
        map.serialize_entry("timestamp", &timestamp)?;
        map.end()
    }
}

fn level_severity(level: &Level) -> (u8, &'static str) {
    match level {
        Level::Error => (17, "ERROR"),
        Level::Warn => (13, "WARN"),
        Level::Info => (9, "INFO"),
        Level::Debug => (5, "DEBUG"),
        Level::Trace => (1, "TRACE"),
    }
}
