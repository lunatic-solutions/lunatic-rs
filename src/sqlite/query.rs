use lunatic_sqlite_api::guest_api::sqlite_guest_bindings as bindings;
use lunatic_sqlite_api::wire_format::{BindKey, BindList, BindPair, SqliteRow};

use super::client::SqliteClient;
use super::error::{SqliteCode, SqliteError, SqliteErrorExt};
use super::value::Value;
use crate::host::call_host_alloc;

/// Trait for querying data and executing queries.
pub trait Query {
    /// Executes a query with no bindings.
    fn query(&self, query: &str) -> Vec<Vec<Value>>;
    /// Prepares a query with bindings.
    fn prepare_query(&self, query: &str) -> Statement;
    /// Executes a query, ignoring any results.
    fn execute(&self, query: &str) -> Result<(), SqliteError>;
}

impl Query for SqliteClient {
    fn query(&self, query: &str) -> Vec<Vec<Value>> {
        self.prepare_query(query).execute()
    }

    fn prepare_query(&self, query: &str) -> Statement {
        let id = unsafe { bindings::query_prepare(self.id(), query.as_ptr(), query.len() as u32) };
        Statement {
            id,
            bindings: BindList(vec![]),
        }
    }

    fn execute(&self, query: &str) -> Result<(), SqliteError> {
        unsafe {
            lunatic_sqlite_api::guest_api::sqlite_guest_bindings::execute(
                self.id(),
                query.as_ptr(),
                query.len() as u32,
            )
        }
        .into_sqlite_error()
    }
}

/// Prepared SQL statement.
pub struct Statement {
    id: u64,
    bindings: BindList,
}

impl Statement {
    /// Bind based on an incrementing index.
    pub fn bind(mut self, value: impl Into<Value>) -> Self {
        let next_idx = self
            .bindings
            .iter()
            .rev()
            .find_map(|binding| match binding {
                BindPair(BindKey::Numeric(idx), _) => Some(idx + 1),
                _ => None,
            })
            .unwrap_or(1);
        self.bindings.0.push(BindPair(
            BindKey::Numeric(next_idx),
            Into::<Value>::into(value).into(),
        ));
        self
    }

    /// Bind based on a name.
    pub fn bind_named(mut self, name: impl Into<String>, value: impl Into<Value>) -> Self {
        self.bindings.0.push(BindPair(
            BindKey::String(name.into()),
            Into::<Value>::into(value).into(),
        ));
        self
    }

    /// Executes the query returning all rows collected as a `Vec`.
    pub fn execute(self) -> Vec<Vec<Value>> {
        self.execute_iter().collect()
    }

    /// Executes the query returning an iterator over rows.
    ///
    /// The query will not be executed until the iter is iterated upon.
    pub fn execute_iter(self) -> QueryIter {
        let encoded = bincode::serialize(&self.bindings).unwrap();
        unsafe { bindings::bind_value(self.id, encoded.as_ptr() as u32, encoded.len() as u32) };

        QueryIter { statement: self }
    }
}

impl Drop for Statement {
    fn drop(&mut self) {
        unsafe {
            bindings::sqlite3_finalize(self.id);
        }
    }
}

/// Iterator for iterating query result rows.
pub struct QueryIter {
    statement: Statement,
}

impl Iterator for QueryIter {
    type Item = Vec<Value>;

    fn next(&mut self) -> Option<Self::Item> {
        match SqliteCode::from_code(unsafe { bindings::sqlite3_step(self.statement.id) }) {
            Some(SqliteCode::Done) => return None,
            Some(SqliteCode::Row) => {}
            Some(code) => panic!("unexpected code {code:?} from lunatic::sqlite::sqlite3_step. Expected SQLITE_DONE or SQLITE_ROW"),
            None => panic!("unexpected code from lunatic::sqlite::sqlite3_step. Expected SQLITE_DONE or SQLITE_ROW"),
        }

        Some(
            call_host_alloc::<SqliteRow>(|len_ptr| unsafe {
                bindings::read_row(self.statement.id, len_ptr)
            })
            .unwrap()
            .0
            .into_iter()
            .map(|value| value.into())
            .collect(),
        )
    }
}
