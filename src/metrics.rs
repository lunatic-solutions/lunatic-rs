//! Expose [metrics](https://crates.io/metrics) functions from lunatic's runtime
//! Lunatic's runtime comes with metrics integration and can expose those metrics
//! to prometheus if prometheus feature is enable at build time and using --prometheus
//! flag to start the exporter
//!
//! All this functions are similar to the macros defined in [metrics docs](https://docs.rs/metrics/latest/metrics/index.html#emission)
use crate::host::api::metrics;

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
