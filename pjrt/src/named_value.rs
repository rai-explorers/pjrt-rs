//! PJRT Named Values
//!
//! This module provides types for working with named key-value pairs used
//! throughout the PJRT API for configuration and metadata.
//!
//! Named values are used for:
//! - Plugin attributes and configuration options
//! - Device and topology descriptions
//! - Cost analysis properties
//! - Compile options
//!
//! The module provides:
//!
//! - `NamedValue`: A single named value with a strongly-typed value
//! - `NamedValueMap`: A collection of named values backed by a HashMap
//! - `Value`: An enum representing different value types (i64, f32, bool, string, i64 list)

use std::collections::HashMap;
use std::slice;

use pjrt_sys::{
    PJRT_NamedValue, PJRT_NamedValue_Type_PJRT_NamedValue_kBool,
    PJRT_NamedValue_Type_PJRT_NamedValue_kFloat, PJRT_NamedValue_Type_PJRT_NamedValue_kInt64,
    PJRT_NamedValue_Type_PJRT_NamedValue_kInt64List, PJRT_NamedValue_Type_PJRT_NamedValue_kString,
};

use crate::{utils, Error};

/// A named value with a strongly-typed value.
///
/// `NamedValue` represents a key-value pair where the value is one of several
/// supported types (integer, float, boolean, string, or integer list).
///
/// # Example
///
/// ```rust,ignore
/// let option1 = NamedValue::i64("device_count", 8);
/// let option2 = NamedValue::string("platform", "cuda");
/// let option3 = NamedValue::bool("enable_xla", true);
/// ```
#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct NamedValue {
    pub name: String,
    pub value: Value,
}

impl NamedValue {
    pub fn new(name: &str, value: Value) -> Self {
        Self {
            name: name.to_string(),
            value,
        }
    }

    pub fn i64(name: &str, value: i64) -> Self {
        Self {
            name: name.to_string(),
            value: Value::I64(value),
        }
    }

    pub fn f32(name: &str, value: f32) -> Self {
        Self {
            name: name.to_string(),
            value: Value::F32(value),
        }
    }

    pub fn bool(name: &str, value: bool) -> Self {
        Self {
            name: name.to_string(),
            value: Value::Bool(value),
        }
    }

    pub fn string(name: &str, value: &str) -> Self {
        Self {
            name: name.to_string(),
            value: Value::String(value.to_string()),
        }
    }

    pub fn i64_list(name: &str, value: Vec<i64>) -> Self {
        Self {
            name: name.to_string(),
            value: Value::I64List(value),
        }
    }
}

/// An enum representing different value types for named values.
///
/// This enum supports the types commonly used in PJRT configuration:
/// integers, floats, booleans, strings, and lists of integers.
#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum Value {
    I64(i64),
    F32(f32),
    Bool(bool),
    String(String),
    I64List(Vec<i64>),
}

impl<'a> From<&'a NamedValue> for PJRT_NamedValue {
    fn from(v: &'a NamedValue) -> Self {
        let mut out = PJRT_NamedValue::new();
        out.name = v.name.as_ptr() as *const i8;
        out.name_size = v.name.len();
        match &v.value {
            Value::I64(i) => {
                out.type_ = PJRT_NamedValue_Type_PJRT_NamedValue_kInt64;
                out.__bindgen_anon_1.int64_value = *i;
            }
            Value::F32(f) => {
                out.type_ = PJRT_NamedValue_Type_PJRT_NamedValue_kFloat;
                out.__bindgen_anon_1.float_value = *f
            }
            Value::Bool(b) => {
                out.type_ = PJRT_NamedValue_Type_PJRT_NamedValue_kBool;
                out.__bindgen_anon_1.bool_value = *b
            }
            Value::String(s) => {
                out.type_ = PJRT_NamedValue_Type_PJRT_NamedValue_kString;
                out.__bindgen_anon_1.string_value = s.as_ptr() as *const i8;
                out.value_size = s.len();
            }
            Value::I64List(l) => {
                out.type_ = PJRT_NamedValue_Type_PJRT_NamedValue_kInt64List;
                out.__bindgen_anon_1.int64_array_value = l.as_ptr();
                out.value_size = l.len();
            }
        }
        out
    }
}

impl<'a> TryFrom<&'a PJRT_NamedValue> for NamedValue {
    type Error = Error;

    #[allow(non_upper_case_globals)]
    fn try_from(value: &'a PJRT_NamedValue) -> std::result::Result<Self, Self::Error> {
        let name = utils::str_from_raw(value.name, value.name_size).into_owned();
        let value = match value.type_ {
            PJRT_NamedValue_Type_PJRT_NamedValue_kInt64 => {
                Value::I64(unsafe { value.__bindgen_anon_1.int64_value })
            }
            PJRT_NamedValue_Type_PJRT_NamedValue_kFloat => {
                Value::F32(unsafe { value.__bindgen_anon_1.float_value })
            }
            PJRT_NamedValue_Type_PJRT_NamedValue_kBool => {
                Value::Bool(unsafe { value.__bindgen_anon_1.bool_value })
            }
            PJRT_NamedValue_Type_PJRT_NamedValue_kString => {
                let value = unsafe {
                    slice::from_raw_parts(
                        value.__bindgen_anon_1.string_value as *const u8,
                        value.value_size,
                    )
                };
                Value::String(String::from_utf8_lossy(value).into_owned())
            }
            PJRT_NamedValue_Type_PJRT_NamedValue_kInt64List => {
                let value = unsafe {
                    slice::from_raw_parts(
                        value.__bindgen_anon_1.int64_array_value,
                        value.value_size,
                    )
                };
                Value::I64List(value.to_vec())
            }
            unknown => return Err(Error::InvalidNamedValueType(unknown as i32)),
        };
        Ok(Self { name, value })
    }
}

/// A map of named values backed by a HashMap.
///
/// `NamedValueMap` provides a convenient way to work with collections of
/// named values, with methods for lookup, conversion, and iteration.
///
/// # Example
///
/// ```rust,ignore
/// let map = NamedValueMap::from(vec![
///     NamedValue::i64("device_count", 8),
///     NamedValue::string("platform", "cuda"),
/// ]);
///
/// if let Some(Value::I64(count)) = map.get("device_count") {
///     println!("Devices: {}", count);
/// }
/// ```
#[derive(Debug, Clone)]
pub struct NamedValueMap {
    inner: HashMap<String, Value>,
}

impl NamedValueMap {
    pub fn new() -> Self {
        Self {
            inner: HashMap::new(),
        }
    }

    pub fn into_inner(self) -> HashMap<String, Value> {
        self.inner
    }

    pub fn into_vec(self) -> Vec<NamedValue> {
        self.inner
            .into_iter()
            .map(|(name, value)| NamedValue { name, value })
            .collect()
    }

    pub fn get(&self, name: &str) -> Option<&Value> {
        self.inner.get(name)
    }
}

impl Default for NamedValueMap {
    fn default() -> Self {
        Self::new()
    }
}

impl From<HashMap<String, Value>> for NamedValueMap {
    fn from(map: HashMap<String, Value>) -> Self {
        Self { inner: map }
    }
}

impl From<Vec<NamedValue>> for NamedValueMap {
    fn from(vec: Vec<NamedValue>) -> Self {
        let map = vec.into_iter().map(|v| (v.name, v.value)).collect();
        Self { inner: map }
    }
}

impl<const N: usize> From<[NamedValue; N]> for NamedValueMap {
    fn from(vec: [NamedValue; N]) -> Self {
        let map = vec.into_iter().map(|v| (v.name, v.value)).collect();
        Self { inner: map }
    }
}

impl<'a> TryFrom<&'a [PJRT_NamedValue]> for NamedValueMap {
    type Error = Error;

    fn try_from(values: &'a [PJRT_NamedValue]) -> std::result::Result<Self, Self::Error> {
        let map = values
            .iter()
            .map(|v| {
                let v = NamedValue::try_from(v)?;
                Ok((v.name, v.value))
            })
            .collect::<std::result::Result<HashMap<String, Value>, Error>>()?;
        Ok(Self { inner: map })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_named_value_new() {
        let nv = NamedValue::new("test", Value::I64(42));
        assert_eq!(nv.name, "test");
        assert_eq!(nv.value, Value::I64(42));
    }

    #[test]
    fn test_named_value_i64() {
        let nv = NamedValue::i64("count", 100);
        assert_eq!(nv.name, "count");
        assert_eq!(nv.value, Value::I64(100));
    }

    #[test]
    fn test_named_value_f32() {
        let nv = NamedValue::f32("threshold", 0.5);
        assert_eq!(nv.name, "threshold");
        assert_eq!(nv.value, Value::F32(0.5));
    }

    #[test]
    fn test_named_value_bool() {
        let nv = NamedValue::bool("enabled", true);
        assert_eq!(nv.name, "enabled");
        assert_eq!(nv.value, Value::Bool(true));

        let nv = NamedValue::bool("disabled", false);
        assert_eq!(nv.value, Value::Bool(false));
    }

    #[test]
    fn test_named_value_string() {
        let nv = NamedValue::string("name", "test_value");
        assert_eq!(nv.name, "name");
        assert_eq!(nv.value, Value::String("test_value".to_string()));
    }

    #[test]
    fn test_named_value_i64_list() {
        let list = vec![1i64, 2, 3, 4, 5];
        let nv = NamedValue::i64_list("dims", list.clone());
        assert_eq!(nv.name, "dims");
        assert_eq!(nv.value, Value::I64List(list));
    }

    #[test]
    fn test_named_value_equality() {
        let nv1 = NamedValue::i64("x", 10);
        let nv2 = NamedValue::i64("x", 10);
        let nv3 = NamedValue::i64("x", 20);
        let nv4 = NamedValue::f32("x", 10.0);

        assert_eq!(nv1, nv2);
        assert_ne!(nv1, nv3);
        assert_ne!(nv1, nv4);
    }

    #[test]
    fn test_value_equality() {
        assert_eq!(Value::I64(42), Value::I64(42));
        assert_ne!(Value::I64(42), Value::I64(43));
        assert_eq!(Value::F32(1.5), Value::F32(1.5));
        assert_eq!(Value::Bool(true), Value::Bool(true));
        assert_eq!(
            Value::String("a".to_string()),
            Value::String("a".to_string())
        );
        assert_eq!(Value::I64List(vec![1, 2]), Value::I64List(vec![1, 2]));

        // Different types should not be equal
        assert_ne!(Value::I64(42), Value::F32(42.0));
        assert_ne!(Value::Bool(true), Value::I64(1));
    }

    #[test]
    fn test_named_value_map_new() {
        let map = NamedValueMap::new();
        assert!(map.inner.is_empty());
    }

    #[test]
    fn test_named_value_map_default() {
        let map: NamedValueMap = Default::default();
        assert!(map.inner.is_empty());
    }

    #[test]
    fn test_named_value_map_get() {
        let nv1 = NamedValue::i64("x", 10);
        let nv2 = NamedValue::f32("y", 0.5);
        let map = NamedValueMap::from(vec![nv1, nv2]);

        assert_eq!(map.get("x"), Some(&Value::I64(10)));
        assert_eq!(map.get("y"), Some(&Value::F32(0.5)));
        assert_eq!(map.get("z"), None);
    }

    #[test]
    fn test_named_value_map_into_inner() {
        let nv = NamedValue::i64("x", 10);
        let map = NamedValueMap::from(vec![nv]);
        let inner = map.into_inner();
        assert_eq!(inner.len(), 1);
        assert_eq!(inner.get("x"), Some(&Value::I64(10)));
    }

    #[test]
    fn test_named_value_map_into_vec() {
        let nv1 = NamedValue::i64("x", 10);
        let nv2 = NamedValue::f32("y", 0.5);
        let map = NamedValueMap::from(vec![nv1, nv2]);
        let vec = map.into_vec();

        assert_eq!(vec.len(), 2);
        // Check that both values are present (order may vary due to HashMap)
        let names: Vec<_> = vec.iter().map(|v| &v.name).collect();
        assert!(names.contains(&&"x".to_string()));
        assert!(names.contains(&&"y".to_string()));
    }

    #[test]
    fn test_from_hashmap() {
        let mut hashmap = HashMap::new();
        hashmap.insert("a".to_string(), Value::I64(1));
        hashmap.insert("b".to_string(), Value::Bool(true));

        let map = NamedValueMap::from(hashmap);
        assert_eq!(map.get("a"), Some(&Value::I64(1)));
        assert_eq!(map.get("b"), Some(&Value::Bool(true)));
    }

    #[test]
    fn test_from_array() {
        let arr = [NamedValue::i64("x", 1), NamedValue::i64("y", 2)];
        let map = NamedValueMap::from(arr);

        assert_eq!(map.get("x"), Some(&Value::I64(1)));
        assert_eq!(map.get("y"), Some(&Value::I64(2)));
    }

    #[test]
    fn test_from_vec() {
        let vec = vec![NamedValue::i64("x", 1), NamedValue::f32("y", 2.0)];
        let map = NamedValueMap::from(vec);

        assert_eq!(map.get("x"), Some(&Value::I64(1)));
        assert_eq!(map.get("y"), Some(&Value::F32(2.0)));
    }

    #[test]
    fn test_clone() {
        let nv = NamedValue::i64("x", 10);
        let cloned = nv.clone();
        assert_eq!(nv, cloned);

        let map = NamedValueMap::from(vec![nv]);
        let cloned_map = map.clone();
        assert_eq!(map.get("x"), cloned_map.get("x"));
    }

    #[test]
    fn test_debug_output() {
        let nv = NamedValue::i64("count", 42);
        let debug = format!("{:?}", nv);
        assert!(debug.contains("NamedValue"));
        assert!(debug.contains("count"));
        assert!(debug.contains("I64"));
        assert!(debug.contains("42"));

        let val = Value::String("test".to_string());
        let debug = format!("{:?}", val);
        assert!(debug.contains("String"));
        assert!(debug.contains("test"));
    }
}
