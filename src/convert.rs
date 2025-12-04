//! Value conversion traits and helpers.
//!
//! This module provides traits and implementations for converting between
//! Fusabi [`Value`] and Rust types, including serde support when enabled.

use std::collections::HashMap;
use thiserror::Error;

use crate::value::{Value, ValueType};

/// Error that occurs during value conversion.
#[derive(Error, Debug, Clone, PartialEq)]
pub enum ValueConversionError {
    /// Type mismatch during conversion.
    #[error("type mismatch: expected {expected}, got {actual}")]
    TypeMismatch {
        /// The expected type.
        expected: ValueType,
        /// The actual type.
        actual: ValueType,
    },

    /// Missing required field in map.
    #[error("missing field: {0}")]
    MissingField(String),

    /// Invalid value for the target type.
    #[error("invalid value: {0}")]
    InvalidValue(String),

    /// Out of range for numeric conversion.
    #[error("value out of range: {0}")]
    OutOfRange(String),

    /// Custom conversion error.
    #[error("{0}")]
    Custom(String),
}

impl ValueConversionError {
    /// Create a type mismatch error.
    pub fn type_mismatch(expected: ValueType, actual: ValueType) -> Self {
        Self::TypeMismatch { expected, actual }
    }

    /// Create a missing field error.
    pub fn missing_field(field: impl Into<String>) -> Self {
        Self::MissingField(field.into())
    }

    /// Create an invalid value error.
    pub fn invalid_value(msg: impl Into<String>) -> Self {
        Self::InvalidValue(msg.into())
    }

    /// Create an out of range error.
    pub fn out_of_range(msg: impl Into<String>) -> Self {
        Self::OutOfRange(msg.into())
    }

    /// Create a custom error.
    pub fn custom(msg: impl Into<String>) -> Self {
        Self::Custom(msg.into())
    }
}

/// Trait for types that can be created from a [`Value`].
pub trait FromValue: Sized {
    /// Convert from a Value, returning an error if conversion fails.
    fn from_value(value: Value) -> Result<Self, ValueConversionError>;

    /// Convert from a Value reference, cloning as needed.
    fn from_value_ref(value: &Value) -> Result<Self, ValueConversionError> {
        Self::from_value(value.clone())
    }
}

/// Trait for types that can be converted into a [`Value`].
pub trait IntoValue {
    /// Convert into a Value.
    fn into_value(self) -> Value;
}

// Blanket implementation for types that implement Into<Value>
impl<T: Into<Value>> IntoValue for T {
    fn into_value(self) -> Value {
        self.into()
    }
}

// FromValue implementations for primitive types

impl FromValue for Value {
    fn from_value(value: Value) -> Result<Self, ValueConversionError> {
        Ok(value)
    }
}

impl FromValue for () {
    fn from_value(value: Value) -> Result<Self, ValueConversionError> {
        match value {
            Value::Null => Ok(()),
            _ => Err(ValueConversionError::type_mismatch(
                ValueType::Null,
                value.value_type(),
            )),
        }
    }
}

impl FromValue for bool {
    fn from_value(value: Value) -> Result<Self, ValueConversionError> {
        match value {
            Value::Bool(b) => Ok(b),
            _ => Err(ValueConversionError::type_mismatch(
                ValueType::Bool,
                value.value_type(),
            )),
        }
    }
}

impl FromValue for i64 {
    fn from_value(value: Value) -> Result<Self, ValueConversionError> {
        match value {
            Value::Int(i) => Ok(i),
            _ => Err(ValueConversionError::type_mismatch(
                ValueType::Int,
                value.value_type(),
            )),
        }
    }
}

impl FromValue for i32 {
    fn from_value(value: Value) -> Result<Self, ValueConversionError> {
        match value {
            Value::Int(i) => i.try_into().map_err(|_| {
                ValueConversionError::out_of_range(format!(
                    "{} is out of range for i32",
                    i
                ))
            }),
            _ => Err(ValueConversionError::type_mismatch(
                ValueType::Int,
                value.value_type(),
            )),
        }
    }
}

impl FromValue for u64 {
    fn from_value(value: Value) -> Result<Self, ValueConversionError> {
        match value {
            Value::Int(i) => i.try_into().map_err(|_| {
                ValueConversionError::out_of_range(format!(
                    "{} is out of range for u64",
                    i
                ))
            }),
            _ => Err(ValueConversionError::type_mismatch(
                ValueType::Int,
                value.value_type(),
            )),
        }
    }
}

impl FromValue for u32 {
    fn from_value(value: Value) -> Result<Self, ValueConversionError> {
        match value {
            Value::Int(i) => i.try_into().map_err(|_| {
                ValueConversionError::out_of_range(format!(
                    "{} is out of range for u32",
                    i
                ))
            }),
            _ => Err(ValueConversionError::type_mismatch(
                ValueType::Int,
                value.value_type(),
            )),
        }
    }
}

impl FromValue for usize {
    fn from_value(value: Value) -> Result<Self, ValueConversionError> {
        match value {
            Value::Int(i) => i.try_into().map_err(|_| {
                ValueConversionError::out_of_range(format!(
                    "{} is out of range for usize",
                    i
                ))
            }),
            _ => Err(ValueConversionError::type_mismatch(
                ValueType::Int,
                value.value_type(),
            )),
        }
    }
}

impl FromValue for f64 {
    fn from_value(value: Value) -> Result<Self, ValueConversionError> {
        match value {
            Value::Float(f) => Ok(f),
            Value::Int(i) => Ok(i as f64),
            _ => Err(ValueConversionError::type_mismatch(
                ValueType::Float,
                value.value_type(),
            )),
        }
    }
}

impl FromValue for f32 {
    fn from_value(value: Value) -> Result<Self, ValueConversionError> {
        match value {
            Value::Float(f) => Ok(f as f32),
            Value::Int(i) => Ok(i as f32),
            _ => Err(ValueConversionError::type_mismatch(
                ValueType::Float,
                value.value_type(),
            )),
        }
    }
}

impl FromValue for String {
    fn from_value(value: Value) -> Result<Self, ValueConversionError> {
        match value {
            Value::String(s) => Ok(s),
            _ => Err(ValueConversionError::type_mismatch(
                ValueType::String,
                value.value_type(),
            )),
        }
    }
}

impl<T: FromValue> FromValue for Vec<T> {
    fn from_value(value: Value) -> Result<Self, ValueConversionError> {
        match value {
            Value::List(list) => list.into_iter().map(T::from_value).collect(),
            _ => Err(ValueConversionError::type_mismatch(
                ValueType::List,
                value.value_type(),
            )),
        }
    }
}

impl<T: FromValue> FromValue for HashMap<String, T> {
    fn from_value(value: Value) -> Result<Self, ValueConversionError> {
        match value {
            Value::Map(map) => map
                .into_iter()
                .map(|(k, v)| T::from_value(v).map(|v| (k, v)))
                .collect(),
            _ => Err(ValueConversionError::type_mismatch(
                ValueType::Map,
                value.value_type(),
            )),
        }
    }
}

impl<T: FromValue> FromValue for Option<T> {
    fn from_value(value: Value) -> Result<Self, ValueConversionError> {
        match value {
            Value::Null => Ok(None),
            other => T::from_value(other).map(Some),
        }
    }
}

impl FromValue for Vec<u8> {
    fn from_value(value: Value) -> Result<Self, ValueConversionError> {
        match value {
            Value::Bytes(b) => Ok(b),
            _ => Err(ValueConversionError::type_mismatch(
                ValueType::Bytes,
                value.value_type(),
            )),
        }
    }
}

// Additional Into<Value> implementations for common types

impl From<()> for Value {
    fn from(_: ()) -> Self {
        Value::Null
    }
}

impl From<usize> for Value {
    fn from(u: usize) -> Self {
        Value::Int(u as i64)
    }
}

impl From<u64> for Value {
    fn from(u: u64) -> Self {
        Value::Int(u as i64)
    }
}

impl From<u32> for Value {
    fn from(u: u32) -> Self {
        Value::Int(u as i64)
    }
}

impl<T: IntoValue> From<Vec<T>> for Value {
    fn from(v: Vec<T>) -> Self {
        Value::List(v.into_iter().map(|x| x.into_value()).collect())
    }
}

// Serde integration when feature is enabled
#[cfg(feature = "serde-support")]
mod serde_support {
    use super::*;
    use serde::{de::DeserializeOwned, Serialize};
    use serde_json;

    /// Convert a Value to a JSON value.
    pub fn value_to_json(value: &Value) -> serde_json::Value {
        match value {
            Value::Null => serde_json::Value::Null,
            Value::Bool(b) => serde_json::Value::Bool(*b),
            Value::Int(i) => serde_json::Value::Number((*i).into()),
            Value::Float(f) => {
                serde_json::Number::from_f64(*f)
                    .map(serde_json::Value::Number)
                    .unwrap_or(serde_json::Value::Null)
            }
            Value::String(s) => serde_json::Value::String(s.clone()),
            Value::List(l) => {
                serde_json::Value::Array(l.iter().map(value_to_json).collect())
            }
            Value::Map(m) => {
                let obj: serde_json::Map<String, serde_json::Value> = m
                    .iter()
                    .map(|(k, v)| (k.clone(), value_to_json(v)))
                    .collect();
                serde_json::Value::Object(obj)
            }
            Value::Function(_) => serde_json::Value::Null,
            Value::Bytes(b) => {
                use base64::Engine as _;
                let encoded = base64::engine::general_purpose::STANDARD.encode(b);
                serde_json::Value::String(encoded)
            }
            Value::Error(e) => {
                let mut obj = serde_json::Map::new();
                obj.insert("error".into(), serde_json::Value::String(e.clone()));
                serde_json::Value::Object(obj)
            }
        }
    }

    /// Convert a JSON value to a Value.
    pub fn json_to_value(json: serde_json::Value) -> Value {
        match json {
            serde_json::Value::Null => Value::Null,
            serde_json::Value::Bool(b) => Value::Bool(b),
            serde_json::Value::Number(n) => {
                if let Some(i) = n.as_i64() {
                    Value::Int(i)
                } else if let Some(f) = n.as_f64() {
                    Value::Float(f)
                } else {
                    Value::Null
                }
            }
            serde_json::Value::String(s) => Value::String(s),
            serde_json::Value::Array(arr) => {
                Value::List(arr.into_iter().map(json_to_value).collect())
            }
            serde_json::Value::Object(obj) => {
                let map: HashMap<String, Value> = obj
                    .into_iter()
                    .map(|(k, v)| (k, json_to_value(v)))
                    .collect();
                Value::Map(map)
            }
        }
    }

    /// Deserialize a Value into a type implementing DeserializeOwned.
    pub fn from_value_serde<T: DeserializeOwned>(
        value: Value,
    ) -> Result<T, ValueConversionError> {
        let json = value_to_json(&value);
        serde_json::from_value(json)
            .map_err(|e| ValueConversionError::custom(e.to_string()))
    }

    /// Serialize a type implementing Serialize into a Value.
    pub fn to_value_serde<T: Serialize>(value: &T) -> Result<Value, ValueConversionError> {
        let json = serde_json::to_value(value)
            .map_err(|e| ValueConversionError::custom(e.to_string()))?;
        Ok(json_to_value(json))
    }

    impl Value {
        /// Deserialize this Value into a serde-compatible type.
        pub fn deserialize<T: DeserializeOwned>(self) -> Result<T, ValueConversionError> {
            from_value_serde(self)
        }

        /// Convert to JSON string.
        pub fn to_json_string(&self) -> String {
            let json = value_to_json(self);
            serde_json::to_string(&json).unwrap_or_else(|_| "null".to_string())
        }

        /// Convert to pretty JSON string.
        pub fn to_json_string_pretty(&self) -> String {
            let json = value_to_json(self);
            serde_json::to_string_pretty(&json).unwrap_or_else(|_| "null".to_string())
        }

        /// Parse from JSON string.
        pub fn from_json_str(s: &str) -> Result<Self, ValueConversionError> {
            let json: serde_json::Value = serde_json::from_str(s)
                .map_err(|e| ValueConversionError::custom(e.to_string()))?;
            Ok(json_to_value(json))
        }
    }
}

#[cfg(feature = "serde-support")]
pub use serde_support::*;

/// Helper macro to extract a required field from a map Value.
#[macro_export]
macro_rules! extract_field {
    ($map:expr, $field:expr, $type:ty) => {{
        let map = match $map {
            $crate::Value::Map(m) => m,
            other => {
                return Err($crate::ValueConversionError::type_mismatch(
                    $crate::ValueType::Map,
                    other.value_type(),
                ))
            }
        };
        let value = map
            .get($field)
            .ok_or_else(|| $crate::ValueConversionError::missing_field($field))?
            .clone();
        <$type as $crate::FromValue>::from_value(value)?
    }};
}

/// Helper macro to extract an optional field from a map Value.
#[macro_export]
macro_rules! extract_field_opt {
    ($map:expr, $field:expr, $type:ty) => {{
        let map = match $map {
            $crate::Value::Map(ref m) => m,
            other => {
                return Err($crate::ValueConversionError::type_mismatch(
                    $crate::ValueType::Map,
                    other.value_type(),
                ))
            }
        };
        match map.get($field) {
            Some(v) => Some(<$type as $crate::FromValue>::from_value(v.clone())?),
            None => None,
        }
    }};
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_value_primitives() {
        assert_eq!(bool::from_value(Value::Bool(true)).unwrap(), true);
        assert_eq!(i64::from_value(Value::Int(42)).unwrap(), 42);
        assert_eq!(f64::from_value(Value::Float(3.14)).unwrap(), 3.14);
        assert_eq!(
            String::from_value(Value::String("hello".into())).unwrap(),
            "hello"
        );
    }

    #[test]
    fn test_from_value_type_mismatch() {
        let err = bool::from_value(Value::Int(42)).unwrap_err();
        assert!(matches!(err, ValueConversionError::TypeMismatch { .. }));
    }

    #[test]
    fn test_from_value_collections() {
        let list = Value::List(vec![Value::Int(1), Value::Int(2), Value::Int(3)]);
        let vec: Vec<i64> = Vec::from_value(list).unwrap();
        assert_eq!(vec, vec![1, 2, 3]);

        let mut map = HashMap::new();
        map.insert("a".into(), Value::Int(1));
        map.insert("b".into(), Value::Int(2));
        let value = Value::Map(map);
        let result: HashMap<String, i64> = HashMap::from_value(value).unwrap();
        assert_eq!(result.get("a"), Some(&1));
        assert_eq!(result.get("b"), Some(&2));
    }

    #[test]
    fn test_from_value_option() {
        let opt: Option<i64> = Option::from_value(Value::Null).unwrap();
        assert_eq!(opt, None);

        let opt: Option<i64> = Option::from_value(Value::Int(42)).unwrap();
        assert_eq!(opt, Some(42));
    }

    #[test]
    fn test_numeric_range() {
        let err = i32::from_value(Value::Int(i64::MAX)).unwrap_err();
        assert!(matches!(err, ValueConversionError::OutOfRange(_)));
    }

    #[cfg(feature = "serde-support")]
    mod serde_tests {
        use super::*;
        use serde::{Deserialize, Serialize};

        #[derive(Debug, PartialEq, Serialize, Deserialize)]
        struct TestStruct {
            name: String,
            value: i32,
            optional: Option<String>,
        }

        #[test]
        fn test_serde_roundtrip() {
            let original = TestStruct {
                name: "test".into(),
                value: 42,
                optional: Some("opt".into()),
            };

            let value = to_value_serde(&original).unwrap();
            let restored: TestStruct = from_value_serde(value).unwrap();
            assert_eq!(original, restored);
        }

        #[test]
        fn test_json_conversion() {
            let value = Value::Map({
                let mut m = HashMap::new();
                m.insert("key".into(), Value::String("value".into()));
                m.insert("number".into(), Value::Int(42));
                m
            });

            let json_str = value.to_json_string();
            let parsed = Value::from_json_str(&json_str).unwrap();

            // Map ordering may differ, so check individual fields
            let parsed_map = parsed.as_map().unwrap();
            assert_eq!(
                parsed_map.get("key"),
                Some(&Value::String("value".into()))
            );
            assert_eq!(parsed_map.get("number"), Some(&Value::Int(42)));
        }
    }
}
