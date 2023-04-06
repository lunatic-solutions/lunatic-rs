use std::{
    collections::BTreeMap,
    fmt,
    time::{SystemTime, UNIX_EPOCH},
};

use serde::{ser::SerializeMap, Serialize};

use super::Level;
use crate::host;

#[derive(Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Span {
    id: u64,
}

impl Span {
    pub fn builder(name: &str) -> SpanBuilder<'_> {
        SpanBuilder::new(name)
    }

    pub fn id(&self) -> u64 {
        self.id
    }

    pub unsafe fn from_id(id: u64) -> Self {
        Span { id }
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
            host::api::metrics::span_drop(self.id);
        }
    }
}

#[must_use]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SpanBuilder<'a> {
    name: &'a str,
    parent: Option<&'a Span>,
    attributes: Option<Vec<u8>>,
}

impl<'a> SpanBuilder<'a> {
    pub fn new(name: &'a str) -> Self {
        SpanBuilder {
            name,
            parent: None,
            attributes: None,
        }
    }

    pub fn parent(mut self, parent: &'a Span) -> Self {
        self.parent = Some(parent);
        self
    }

    pub fn attributes<T>(mut self, attributes: &T) -> Result<Self, serde_json::Error>
    where
        T: Serialize,
    {
        let attributes = serde_json::to_vec(attributes)?;
        self.attributes = Some(attributes);
        Ok(self)
    }

    pub fn build(self) -> Span {
        let parent_id = self.parent.map(|span| span.id).unwrap_or(u64::MAX);
        let attributes_bytes = self.attributes.unwrap_or(vec![]);
        let id = unsafe {
            host::api::metrics::span_start(
                parent_id,
                self.name.as_ptr(),
                self.name.len(),
                attributes_bytes.as_ptr(),
                attributes_bytes.len(),
            )
        };
        Span { id }
    }
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
        host::api::metrics::event(
            span.unwrap_or(u64::MAX),
            name.as_ptr(),
            name.len(),
            attributes_bytes.as_ptr(),
            attributes_bytes.len(),
        )
    }

    Ok(())
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
