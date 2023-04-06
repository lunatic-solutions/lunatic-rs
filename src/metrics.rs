//! Lunatic runtime comes with metrics integration with [OpenTelemetry](https://opentelemetry.io/).
//!
//! Logging can be printed to the terminal, or collected via jaeger.
//!
//! Metrics are available for prometheus.
//!
//! The logging macros are modified from tracing, and the syntax is the same.

pub use self::counter::*;
pub use self::histogram::*;
pub use self::level::*;
pub use self::meter::*;
pub use self::span::*;

mod counter;
mod histogram;
mod level;
mod macros;
mod meter;
mod span;
