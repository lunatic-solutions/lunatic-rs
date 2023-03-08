//! Sqlite database client.
//!
//! # Example
//!
//! ```
//! use lunatic::sqlite::{SqliteClient, Query};
//!
//! let client = SqliteClient::connect("app.db")?;
//!
//! // Query users
//! let mut rows = client.query("select * from users");
//! for row in rows {
//!     // ...
//! }
//! ```

mod client;
mod error;
mod query;
mod value;

pub use client::*;
pub use error::*;
pub use query::*;
pub use value::*;
