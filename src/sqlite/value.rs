use lunatic_sqlite_api::wire_format::{BindValue, SqliteValue};
use serde::{Deserialize, Serialize};

/// Sqlite value for binding in queries.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Value {
    Null,
    Blob(Vec<u8>),
    Text(String),
    Double(f64),
    Int(i32),
    Int64(i64),
}

macro_rules! impl_into_value {
    ($into_fn: ident, $as_fn: ident, Null, $t: ty) => {
        impl Value {
            pub fn $into_fn(self) -> Option<$t> {
                match self {
                    Value::Null => Some(()),
                    _ => None,
                }
            }

            pub fn $as_fn(&self) -> Option<&$t> {
                match self {
                    Value::Null => Some(&()),
                    _ => None,
                }
            }
        }
    };
    ($into_fn: ident, $as_fn: ident, $v: ident, $t: ty) => {
        impl Value {
            pub fn $into_fn(self) -> Option<$t> {
                match self {
                    Value::$v(v) => Some(v),
                    _ => None,
                }
            }

            pub fn $as_fn(&self) -> Option<&$t> {
                match self {
                    Value::$v(v) => Some(v),
                    _ => None,
                }
            }
        }
    };
}

impl_into_value!(into_null, as_null, Null, ());
impl_into_value!(into_blob, as_blob, Blob, Vec<u8>);
impl_into_value!(into_text, as_text, Text, String);
impl_into_value!(into_double, as_double, Double, f64);
impl_into_value!(into_int, as_int, Int, i32);
impl_into_value!(into_int64, as_int64, Int64, i64);

impl Value {
    pub fn as_int_any(&self) -> Option<i64> {
        match self {
            Value::Int(v) => Some(*v as i64),
            Value::Int64(v) => Some(*v),
            _ => None,
        }
    }
}

macro_rules! impl_from_type {
    ($t: ty, $v: ident) => {
        impl From<$t> for Value {
            fn from(value: $t) -> Self {
                Value::$v(value)
            }
        }
    };
}

impl_from_type!(Vec<u8>, Blob);
impl_from_type!(String, Text);
impl_from_type!(f64, Double);
impl_from_type!(i32, Int);
impl_from_type!(i64, Int64);

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

impl From<&str> for Value {
    fn from(value: &str) -> Self {
        value.to_string().into()
    }
}

impl From<&String> for Value {
    fn from(value: &String) -> Self {
        value.as_str().into()
    }
}

impl From<f32> for Value {
    fn from(value: f32) -> Self {
        Value::Double(value as f64)
    }
}

impl<T> From<Option<T>> for Value
where
    T: Into<Value>,
{
    fn from(value: Option<T>) -> Self {
        value.map(|v| v.into()).unwrap_or(Value::Null)
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
