//! Expose [metrics](https://crates.io/metrics) functions from lunatic's runtime
//! Lunatic's runtime comes with metrics integration and can expose those metrics
//! to prometheus if prometheus feature is enable at build time and using --prometheus
//! flag to start the exporter
//!
//! All this functions are similar to the macros defined in [metrics docs](https://docs.rs/metrics/latest/metrics/index.html#emission)

use serde::Serialize;

use crate::host::{self, api::metrics};

#[derive(Clone, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Span {
    id: u64,
}

impl Span {
    pub fn new<T>(name: &str, attributes: &T) -> Result<Self, serde_json::Error>
    where
        T: Serialize + 'static,
    {
        Self::new_(None, name, attributes)
    }

    pub fn new_with_parent<T>(
        parent: &Span,
        name: &str,
        attributes: &T,
    ) -> Result<Self, serde_json::Error>
    where
        T: Serialize + 'static,
    {
        Self::new_(Some(parent), name, attributes)
    }

    fn new_<T>(parent: Option<&Span>, name: &str, attributes: &T) -> Result<Self, serde_json::Error>
    where
        T: Serialize + 'static,
    {
        let parent_id = parent.map(|span| span.id).unwrap_or(u64::MAX);
        let id = if std::any::TypeId::of::<T>() == std::any::TypeId::of::<()>() {
            unsafe {
                host::api::metrics::start_span(
                    parent_id,
                    name.as_ptr(),
                    name.len(),
                    0 as *const u8,
                    0,
                )
            }
        } else {
            let attributes_bytes = serde_json::to_vec(attributes)?;
            unsafe {
                host::api::metrics::start_span(
                    parent_id,
                    name.as_ptr(),
                    name.len(),
                    attributes_bytes.as_ptr(),
                    attributes_bytes.len(),
                )
            }
        };

        let span = Span { id };

        Ok(span)
    }

    pub fn add_event<T>(&self, name: &str, attributes: &T) -> Result<(), serde_json::Error>
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

pub fn add_event<T>(span: Option<u64>, name: &str, attributes: &T) -> Result<(), serde_json::Error>
where
    T: Serialize + 'static,
{
    if std::any::TypeId::of::<T>() == std::any::TypeId::of::<()>() {
        unsafe {
            host::api::metrics::add_event(
                span.unwrap_or(u64::MAX),
                name.as_ptr(),
                name.len(),
                0 as *const u8,
                0,
            )
        }
    } else {
        let attributes_bytes = serde_json::to_vec(attributes)?;
        unsafe {
            host::api::metrics::add_event(
                span.unwrap_or(u64::MAX),
                name.as_ptr(),
                name.len(),
                attributes_bytes.as_ptr(),
                attributes_bytes.len(),
            )
        }
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
