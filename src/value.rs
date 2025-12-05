//! Fusabi Value type and basic operations.

use std::collections::HashMap;
use std::fmt;

/// The type of a Fusabi value.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ValueType {
    /// Null/nil value.
    Null,
    /// Boolean value.
    Bool,
    /// Integer value.
    Int,
    /// Floating point value.
    Float,
    /// String value.
    String,
    /// List/array value.
    List,
    /// Map/object value.
    Map,
    /// Function value.
    Function,
    /// Bytes/binary data.
    Bytes,
    /// Error value.
    Error,
}

impl fmt::Display for ValueType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ValueType::Null => write!(f, "null"),
            ValueType::Bool => write!(f, "bool"),
            ValueType::Int => write!(f, "int"),
            ValueType::Float => write!(f, "float"),
            ValueType::String => write!(f, "string"),
            ValueType::List => write!(f, "list"),
            ValueType::Map => write!(f, "map"),
            ValueType::Function => write!(f, "function"),
            ValueType::Bytes => write!(f, "bytes"),
            ValueType::Error => write!(f, "error"),
        }
    }
}

/// A Fusabi runtime value.
///
/// This is a representation of values that can be passed between
/// the host and Fusabi scripts.
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    /// Null/nil value.
    Null,
    /// Boolean value.
    Bool(bool),
    /// Integer value (64-bit signed).
    Int(i64),
    /// Floating point value (64-bit).
    Float(f64),
    /// UTF-8 string value.
    String(String),
    /// Ordered list of values.
    List(Vec<Value>),
    /// Key-value map (string keys).
    Map(HashMap<String, Value>),
    /// Opaque function reference (not directly usable by host).
    Function(FunctionRef),
    /// Binary data.
    Bytes(Vec<u8>),
    /// Error value with message.
    Error(String),
}

/// An opaque reference to a Fusabi function.
#[derive(Debug, Clone, PartialEq)]
pub struct FunctionRef {
    /// Internal identifier.
    pub(crate) id: u64,
    /// Function name if known.
    pub(crate) name: Option<String>,
}

impl FunctionRef {
    /// Get the function name if available.
    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }
}

impl Value {
    /// Get the type of this value.
    pub fn value_type(&self) -> ValueType {
        match self {
            Value::Null => ValueType::Null,
            Value::Bool(_) => ValueType::Bool,
            Value::Int(_) => ValueType::Int,
            Value::Float(_) => ValueType::Float,
            Value::String(_) => ValueType::String,
            Value::List(_) => ValueType::List,
            Value::Map(_) => ValueType::Map,
            Value::Function(_) => ValueType::Function,
            Value::Bytes(_) => ValueType::Bytes,
            Value::Error(_) => ValueType::Error,
        }
    }

    /// Returns true if this is a null value.
    pub fn is_null(&self) -> bool {
        matches!(self, Value::Null)
    }

    /// Returns true if this is an error value.
    pub fn is_error(&self) -> bool {
        matches!(self, Value::Error(_))
    }

    /// Try to get as a bool.
    pub fn as_bool(&self) -> Option<bool> {
        match self {
            Value::Bool(b) => Some(*b),
            _ => None,
        }
    }

    /// Try to get as an integer.
    pub fn as_int(&self) -> Option<i64> {
        match self {
            Value::Int(i) => Some(*i),
            _ => None,
        }
    }

    /// Try to get as a float.
    pub fn as_float(&self) -> Option<f64> {
        match self {
            Value::Float(f) => Some(*f),
            Value::Int(i) => Some(*i as f64),
            _ => None,
        }
    }

    /// Try to get as a string.
    pub fn as_str(&self) -> Option<&str> {
        match self {
            Value::String(s) => Some(s.as_str()),
            _ => None,
        }
    }

    /// Try to get as a list.
    pub fn as_list(&self) -> Option<&[Value]> {
        match self {
            Value::List(l) => Some(l.as_slice()),
            _ => None,
        }
    }

    /// Try to get as a map.
    pub fn as_map(&self) -> Option<&HashMap<String, Value>> {
        match self {
            Value::Map(m) => Some(m),
            _ => None,
        }
    }

    /// Try to get as bytes.
    pub fn as_bytes(&self) -> Option<&[u8]> {
        match self {
            Value::Bytes(b) => Some(b.as_slice()),
            _ => None,
        }
    }

    /// Get the error message if this is an error value.
    pub fn as_error(&self) -> Option<&str> {
        match self {
            Value::Error(e) => Some(e.as_str()),
            _ => None,
        }
    }

    /// Create a null value.
    pub fn null() -> Self {
        Value::Null
    }

    /// Create an error value.
    pub fn error(msg: impl Into<String>) -> Self {
        Value::Error(msg.into())
    }
}

impl Default for Value {
    fn default() -> Self {
        Value::Null
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Null => write!(f, "null"),
            Value::Bool(b) => write!(f, "{}", b),
            Value::Int(i) => write!(f, "{}", i),
            Value::Float(fl) => write!(f, "{}", fl),
            Value::String(s) => write!(f, "\"{}\"", s),
            Value::List(l) => {
                write!(f, "[")?;
                for (i, v) in l.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", v)?;
                }
                write!(f, "]")
            }
            Value::Map(m) => {
                write!(f, "{{")?;
                for (i, (k, v)) in m.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "\"{}\": {}", k, v)?;
                }
                write!(f, "}}")
            }
            Value::Function(fr) => {
                if let Some(name) = &fr.name {
                    write!(f, "<function {}>", name)
                } else {
                    write!(f, "<function>")
                }
            }
            Value::Bytes(b) => write!(f, "<bytes len={}>", b.len()),
            Value::Error(e) => write!(f, "<error: {}>", e),
        }
    }
}

// Conversion implementations
impl From<bool> for Value {
    fn from(b: bool) -> Self {
        Value::Bool(b)
    }
}

impl From<i64> for Value {
    fn from(i: i64) -> Self {
        Value::Int(i)
    }
}

impl From<i32> for Value {
    fn from(i: i32) -> Self {
        Value::Int(i as i64)
    }
}

impl From<f64> for Value {
    fn from(f: f64) -> Self {
        Value::Float(f)
    }
}

impl From<f32> for Value {
    fn from(f: f32) -> Self {
        Value::Float(f as f64)
    }
}

impl From<String> for Value {
    fn from(s: String) -> Self {
        Value::String(s)
    }
}

impl From<&str> for Value {
    fn from(s: &str) -> Self {
        Value::String(s.to_string())
    }
}



impl From<HashMap<String, Value>> for Value {
    fn from(m: HashMap<String, Value>) -> Self {
        Value::Map(m)
    }
}

impl From<Vec<u8>> for Value {
    fn from(b: Vec<u8>) -> Self {
        Value::Bytes(b)
    }
}

impl<T: Into<Value>> From<Option<T>> for Value {
    fn from(opt: Option<T>) -> Self {
        match opt {
            Some(v) => v.into(),
            None => Value::Null,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_value_types() {
        assert_eq!(Value::Null.value_type(), ValueType::Null);
        assert_eq!(Value::Bool(true).value_type(), ValueType::Bool);
        assert_eq!(Value::Int(42).value_type(), ValueType::Int);
        assert_eq!(Value::Float(3.14).value_type(), ValueType::Float);
        assert_eq!(Value::String("hello".into()).value_type(), ValueType::String);
        assert_eq!(Value::List(vec![]).value_type(), ValueType::List);
        assert_eq!(Value::Map(HashMap::new()).value_type(), ValueType::Map);
    }

    #[test]
    fn test_value_accessors() {
        assert!(Value::Null.is_null());
        assert!(!Value::Bool(true).is_null());

        assert_eq!(Value::Bool(true).as_bool(), Some(true));
        assert_eq!(Value::Int(42).as_bool(), None);

        assert_eq!(Value::Int(42).as_int(), Some(42));
        assert_eq!(Value::Float(3.14).as_float(), Some(3.14));
        assert_eq!(Value::Int(42).as_float(), Some(42.0));

        assert_eq!(Value::String("test".into()).as_str(), Some("test"));
    }

    #[test]
    fn test_value_from_impls() {
        let v: Value = true.into();
        assert_eq!(v, Value::Bool(true));

        let v: Value = 42i64.into();
        assert_eq!(v, Value::Int(42));

        let v: Value = "hello".into();
        assert_eq!(v, Value::String("hello".into()));

        let v: Value = None::<i64>.into();
        assert!(v.is_null());

        let v: Value = Some(42i64).into();
        assert_eq!(v, Value::Int(42));
    }

    #[test]
    fn test_value_display() {
        assert_eq!(format!("{}", Value::Null), "null");
        assert_eq!(format!("{}", Value::Bool(true)), "true");
        assert_eq!(format!("{}", Value::Int(42)), "42");
        assert_eq!(format!("{}", Value::String("test".into())), "\"test\"");
    }
}
