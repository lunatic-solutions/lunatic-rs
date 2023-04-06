//! Expose [metrics](https://crates.io/metrics) functions from lunatic's runtime
//! Lunatic's runtime comes with metrics integration and can expose those metrics
//! to prometheus if prometheus feature is enable at build time and using --prometheus
//! flag to start the exporter
//!
//! All this functions are similar to the macros defined in [metrics docs](https://docs.rs/metrics/latest/metrics/index.html#emission)

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
