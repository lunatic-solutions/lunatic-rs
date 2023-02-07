use lunatic_sqlite_api::wire_format::{BindValue, SqliteValue};
use serde::{Deserialize, Serialize};

/// An Sqlite value for binding in queries.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Value {
    Null,
    Blob(Vec<u8>),
    Text(String),
    Double(f64),
    Int(i32),
    Int64(i64),
}

impl From<()> for Value {
    fn from(_value: ()) -> Self {
        Value::Null
    }
}

impl From<&[u8]> for Value {
    fn from(value: &[u8]) -> Self {
        value.to_vec().into()
    }
}

impl From<Vec<u8>> for Value {
    fn from(value: Vec<u8>) -> Self {
        Value::Blob(value)
    }
}

impl From<&str> for Value {
    fn from(value: &str) -> Self {
        value.to_string().into()
    }
}

impl From<String> for Value {
    fn from(value: String) -> Self {
        Value::Text(value)
    }
}

impl From<f32> for Value {
    fn from(value: f32) -> Self {
        Value::Double(value as f64)
    }
}

impl From<f64> for Value {
    fn from(value: f64) -> Self {
        Value::Double(value)
    }
}

impl From<i32> for Value {
    fn from(value: i32) -> Self {
        Value::Int(value)
    }
}

impl From<i64> for Value {
    fn from(value: i64) -> Self {
        Value::Int64(value)
    }
}

impl From<Value> for BindValue {
    fn from(value: Value) -> Self {
        match value {
            Value::Null => BindValue::Null,
            Value::Blob(v) => BindValue::Blob(v),
            Value::Text(v) => BindValue::Text(v),
            Value::Double(v) => BindValue::Double(v),
            Value::Int(v) => BindValue::Int(v),
            Value::Int64(v) => BindValue::Int64(v),
        }
    }
}

impl From<SqliteValue> for Value {
    fn from(value: SqliteValue) -> Self {
        match value {
            SqliteValue::Null => Value::Null,
            SqliteValue::Blob(v) => Value::Blob(v),
            SqliteValue::Text(v) => Value::Text(v),
            SqliteValue::Double(v) => Value::Double(v),
            SqliteValue::Integer(v) => Value::Int64(v),
            SqliteValue::I64(v) => Value::Int64(v),
        }
    }
}
