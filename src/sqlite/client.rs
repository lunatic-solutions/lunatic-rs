use super::error::SqliteError;

/// Sqlite client witn an existing connection.
pub struct SqliteClient {
    conn: u64,
}

impl SqliteClient {
    /// Connects to the Sqlite database at `path` if present, otherwise creates
    /// a new database.
    pub fn connect(path: &str) -> Result<Self, SqliteError> {
        let connection_id = 0;
        let res = unsafe {
            lunatic_sqlite_api::guest_api::sqlite_guest_bindings::open(
                path.as_ptr(),
                path.len(),
                connection_id as *mut u32,
            )
        };
        if res != 0 {
            return Err(SqliteError::default());
        }
        Ok(SqliteClient {
            conn: connection_id,
        })
    }

    pub(crate) fn id(&self) -> u64 {
        self.conn
    }
}
