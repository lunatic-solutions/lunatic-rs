//! Expose [metrics](https://crates.io/metrics) functions from lunatic's runtime
//! Lunatic's runtime comes with metrics integration and can expose those metrics
//! to prometheus if prometheus feature is enable at build time and using --prometheus
//! flag to start the exporter
//!
//! All this functions are similar to the macros defined in [metrics docs](https://docs.rs/metrics/latest/metrics/index.html#emission)

pub use log::Level;

pub use self::counter::*;
pub use self::meter::*;
pub use self::span::*;
use crate::host::api::metrics;

mod counter;
mod macros;
mod meter;
mod span;

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
