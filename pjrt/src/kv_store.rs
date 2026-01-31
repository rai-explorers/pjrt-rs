use std::ffi::CString;
use std::mem;

use pjrt_sys::{
    PJRT_Error, PJRT_Error_Code, PJRT_Error_Code_PJRT_Error_Code_NOT_FOUND,
    PJRT_KeyValueGetCallback_Args, PJRT_KeyValuePutCallback_Args, PJRT_KeyValueTryGetCallback_Args,
};
use std::os::raw::c_char;

use crate::{utils, Result};

unsafe extern "C" fn value_deleter_callback(value: *mut c_char) {
    if !value.is_null() {
        let _ = CString::from_raw(value);
    }
}

pub(crate) unsafe extern "C" fn kv_get_callback(
    args: *mut PJRT_KeyValueGetCallback_Args,
) -> *mut PJRT_Error {
    let args = unsafe { args.as_mut().unwrap() };
    let store = unsafe { (args.user_arg as *mut &dyn KeyValueStore).as_mut().unwrap() };
    let key = utils::str_from_raw(args.key, args.key_size);
    args.value_deleter_callback = Some(value_deleter_callback);
    match store.get(&key, args.timeout_in_ms) {
        Ok(value) => {
            // as value_deleter_callback only accepts *mut c_char, we need to convert to CString
            let value = CString::new(value).unwrap();
            args.value = value.as_ptr() as *mut c_char;
            args.value_size = value.count_bytes();
            mem::forget(value);
            std::ptr::null_mut()
        }
        Err(err) => {
            let err_callback = (*args.callback_error).expect("callback_error");
            let code = err.code() as PJRT_Error_Code;
            let message = format!("{:?}", err);
            let msg_bytes = message.as_bytes();
            (err_callback)(code, msg_bytes.as_ptr() as *const _, msg_bytes.len())
        }
    }
}

pub(crate) unsafe extern "C" fn kv_put_callback(
    args: *mut PJRT_KeyValuePutCallback_Args,
) -> *mut PJRT_Error {
    let args = unsafe { args.as_mut().unwrap() };
    let store = unsafe { (args.user_arg as *mut &dyn KeyValueStore).as_mut().unwrap() };
    let key = utils::str_from_raw(args.key, args.key_size);
    let value = utils::str_from_raw(args.value, args.value_size);
    match store.put(&key, &value) {
        Ok(_) => std::ptr::null_mut(),
        Err(err) => {
            let err_callback = (*args.callback_error).expect("callback_error");
            let code = err.code() as PJRT_Error_Code;
            let message = format!("{:?}", err);
            let msg_bytes = message.as_bytes();
            (err_callback)(code, msg_bytes.as_ptr() as *const _, msg_bytes.len())
        }
    }
}

pub(crate) unsafe extern "C" fn kv_try_get_callback(
    args: *mut PJRT_KeyValueTryGetCallback_Args,
) -> *mut PJRT_Error {
    let args = unsafe { args.as_mut().unwrap() };
    let store = unsafe { (args.user_arg as *mut &dyn KeyValueStore).as_mut().unwrap() };
    let key = utils::str_from_raw(args.key, args.key_size);
    args.value_deleter_callback = Some(value_deleter_callback);
    match store.try_get(&key) {
        Ok(Some(value)) => {
            // as value_deleter_callback only accepts *mut c_char, we need to convert to CString
            let value = CString::new(value).unwrap();
            args.value = value.as_ptr() as *mut c_char;
            args.value_size = value.count_bytes();
            mem::forget(value);
            std::ptr::null_mut()
        }
        Ok(None) => {
            // Key not found - return NotFound error
            let err_callback = (*args.callback_error).expect("callback_error");
            let code = PJRT_Error_Code_PJRT_Error_Code_NOT_FOUND;
            let message = format!("Key not found: {}", key);
            let msg_bytes = message.as_bytes();
            (err_callback)(code, msg_bytes.as_ptr() as *const _, msg_bytes.len())
        }
        Err(err) => {
            let err_callback = (*args.callback_error).expect("callback_error");
            let code = err.code() as PJRT_Error_Code;
            let message = format!("{:?}", err);
            let msg_bytes = message.as_bytes();
            (err_callback)(code, msg_bytes.as_ptr() as *const _, msg_bytes.len())
        }
    }
}

pub trait KeyValueStore {
    fn get(&self, key: &str, timeout_in_ms: i32) -> Result<String>;
    fn put(&self, key: &str, value: &str) -> Result<()>;
    /// Try to get a value from the key-value store.
    /// Returns `Ok(Some(value))` if the key exists.
    /// Returns `Ok(None)` if the key does not exist.
    /// Returns `Err` for other errors.
    fn try_get(&self, key: &str) -> Result<Option<String>>;
}
