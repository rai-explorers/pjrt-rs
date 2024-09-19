use std::collections::HashMap;
use std::slice;

use pjrt_sys::{
    PJRT_NamedValue, PJRT_NamedValue_Type_PJRT_NamedValue_kBool,
    PJRT_NamedValue_Type_PJRT_NamedValue_kFloat, PJRT_NamedValue_Type_PJRT_NamedValue_kInt64,
    PJRT_NamedValue_Type_PJRT_NamedValue_kInt64List, PJRT_NamedValue_Type_PJRT_NamedValue_kString,
};

use crate::utils;

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
        out.name_size = v.name.as_bytes().len();
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
                out.value_size = s.as_bytes().len();
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

impl<'a> From<&'a PJRT_NamedValue> for NamedValue {
    #[allow(non_upper_case_globals)]
    fn from(value: &'a PJRT_NamedValue) -> Self {
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
            // using try_from instead?
            _ => panic!("Unknown PJRT_NamedValue_Type"),
        };
        Self { name, value }
    }
}

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

impl<'a> From<&'a [PJRT_NamedValue]> for NamedValueMap {
    fn from(values: &'a [PJRT_NamedValue]) -> Self {
        let map = values
            .iter()
            .map(|v| {
                let v = NamedValue::from(v);
                (v.name, v.value)
            })
            .collect::<HashMap<String, Value>>();
        Self { inner: map }
    }
}
