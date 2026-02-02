use std::ffi::CString;
use std::mem;
use std::os::raw::c_char;

use pjrt_sys::{
    PJRT_Error, PJRT_Error_Code, PJRT_Error_Code_PJRT_Error_Code_INTERNAL,
    PJRT_Error_Code_PJRT_Error_Code_INVALID_ARGUMENT, PJRT_Error_Code_PJRT_Error_Code_NOT_FOUND,
    PJRT_KeyValueGetCallback_Args, PJRT_KeyValuePutCallback_Args, PJRT_KeyValueTryGetCallback_Args,
};

use crate::{utils, Result};

/// # Safety
///
/// This function is called by the PJRT runtime to free values returned by the KV store callbacks.
/// The `value` pointer must have been allocated by `CString::into_raw()`.
unsafe extern "C" fn value_deleter_callback(value: *mut c_char) {
    if !value.is_null() {
        // SAFETY: The value was created by CString::into_raw() in the callback functions
        let _ = unsafe { CString::from_raw(value) };
    }
}

/// Helper function to create a PJRT error via the callback error function.
///
/// # Safety
///
/// The `callback_error` must be a valid pointer to a PJRT_CallbackError function.
unsafe fn create_callback_error(
    callback_error: *mut pjrt_sys::PJRT_CallbackError,
    code: PJRT_Error_Code,
    message: &str,
) -> *mut PJRT_Error {
    if callback_error.is_null() {
        // Cannot create an error - return null and hope for the best
        // This shouldn't happen in practice as PJRT always provides callback_error
        return std::ptr::null_mut();
    }
    let err_callback = unsafe { *callback_error };
    match err_callback {
        Some(cb) => {
            let msg_bytes = message.as_bytes();
            cb(code, msg_bytes.as_ptr() as *const _, msg_bytes.len())
        }
        None => std::ptr::null_mut(),
    }
}

/// Callback function for key-value get operations.
///
/// # Safety
///
/// This function is called by the PJRT runtime.
/// - `args` must be a valid, non-null pointer to `PJRT_KeyValueGetCallback_Args`
/// - `args.user_arg` must point to a valid `&dyn KeyValueStore` reference
/// - `args.callback_error` must be a valid pointer to the error callback function
pub(crate) unsafe extern "C" fn kv_get_callback(
    args: *mut PJRT_KeyValueGetCallback_Args,
) -> *mut PJRT_Error {
    // SAFETY-001 fix: Check for null args pointer
    let Some(args) = (unsafe { args.as_mut() }) else {
        // Cannot return an error because we don't have access to callback_error
        // This would be a severe bug in the PJRT runtime
        return std::ptr::null_mut();
    };

    // SAFETY-001 fix: Check for null user_arg pointer
    let Some(store) = (unsafe { (args.user_arg as *mut &dyn KeyValueStore).as_mut() }) else {
        return unsafe {
            create_callback_error(
                args.callback_error,
                PJRT_Error_Code_PJRT_Error_Code_INVALID_ARGUMENT,
                "kv_get_callback: null user_arg pointer",
            )
        };
    };

    let key = utils::str_from_raw(args.key, args.key_size);
    args.value_deleter_callback = Some(value_deleter_callback);

    match store.get(&key, args.timeout_in_ms) {
        Ok(value) => {
            // CString::new can fail if the string contains null bytes
            match CString::new(value) {
                Ok(cvalue) => {
                    args.value = cvalue.as_ptr() as *mut c_char;
                    args.value_size = cvalue.count_bytes();
                    // Prevent the CString from being dropped - it will be freed by value_deleter_callback
                    mem::forget(cvalue);
                    std::ptr::null_mut()
                }
                Err(e) => unsafe {
                    create_callback_error(
                        args.callback_error,
                        PJRT_Error_Code_PJRT_Error_Code_INTERNAL,
                        &format!("kv_get_callback: invalid value (contains null byte): {}", e),
                    )
                },
            }
        }
        Err(err) => unsafe {
            create_callback_error(
                args.callback_error,
                err.code() as PJRT_Error_Code,
                &format!("{:?}", err),
            )
        },
    }
}

/// Callback function for key-value put operations.
///
/// # Safety
///
/// This function is called by the PJRT runtime.
/// - `args` must be a valid, non-null pointer to `PJRT_KeyValuePutCallback_Args`
/// - `args.user_arg` must point to a valid `&dyn KeyValueStore` reference
/// - `args.callback_error` must be a valid pointer to the error callback function
pub(crate) unsafe extern "C" fn kv_put_callback(
    args: *mut PJRT_KeyValuePutCallback_Args,
) -> *mut PJRT_Error {
    // SAFETY-001 fix: Check for null args pointer
    let Some(args) = (unsafe { args.as_mut() }) else {
        return std::ptr::null_mut();
    };

    // SAFETY-001 fix: Check for null user_arg pointer
    let Some(store) = (unsafe { (args.user_arg as *mut &dyn KeyValueStore).as_mut() }) else {
        return unsafe {
            create_callback_error(
                args.callback_error,
                PJRT_Error_Code_PJRT_Error_Code_INVALID_ARGUMENT,
                "kv_put_callback: null user_arg pointer",
            )
        };
    };

    let key = utils::str_from_raw(args.key, args.key_size);
    let value = utils::str_from_raw(args.value, args.value_size);

    match store.put(&key, &value) {
        Ok(_) => std::ptr::null_mut(),
        Err(err) => unsafe {
            create_callback_error(
                args.callback_error,
                err.code() as PJRT_Error_Code,
                &format!("{:?}", err),
            )
        },
    }
}

/// Callback function for key-value try-get operations.
///
/// # Safety
///
/// This function is called by the PJRT runtime.
/// - `args` must be a valid, non-null pointer to `PJRT_KeyValueTryGetCallback_Args`
/// - `args.user_arg` must point to a valid `&dyn KeyValueStore` reference
/// - `args.callback_error` must be a valid pointer to the error callback function
pub(crate) unsafe extern "C" fn kv_try_get_callback(
    args: *mut PJRT_KeyValueTryGetCallback_Args,
) -> *mut PJRT_Error {
    // SAFETY-001 fix: Check for null args pointer
    let Some(args) = (unsafe { args.as_mut() }) else {
        return std::ptr::null_mut();
    };

    // SAFETY-001 fix: Check for null user_arg pointer
    let Some(store) = (unsafe { (args.user_arg as *mut &dyn KeyValueStore).as_mut() }) else {
        return unsafe {
            create_callback_error(
                args.callback_error,
                PJRT_Error_Code_PJRT_Error_Code_INVALID_ARGUMENT,
                "kv_try_get_callback: null user_arg pointer",
            )
        };
    };

    let key = utils::str_from_raw(args.key, args.key_size);
    args.value_deleter_callback = Some(value_deleter_callback);

    match store.try_get(&key) {
        Ok(Some(value)) => {
            // CString::new can fail if the string contains null bytes
            match CString::new(value) {
                Ok(cvalue) => {
                    args.value = cvalue.as_ptr() as *mut c_char;
                    args.value_size = cvalue.count_bytes();
                    // Prevent the CString from being dropped - it will be freed by value_deleter_callback
                    mem::forget(cvalue);
                    std::ptr::null_mut()
                }
                Err(e) => unsafe {
                    create_callback_error(
                        args.callback_error,
                        PJRT_Error_Code_PJRT_Error_Code_INTERNAL,
                        &format!(
                            "kv_try_get_callback: invalid value (contains null byte): {}",
                            e
                        ),
                    )
                },
            }
        }
        Ok(None) => {
            // Key not found - return NotFound error
            unsafe {
                create_callback_error(
                    args.callback_error,
                    PJRT_Error_Code_PJRT_Error_Code_NOT_FOUND,
                    &format!("Key not found: {}", key),
                )
            }
        }
        Err(err) => unsafe {
            create_callback_error(
                args.callback_error,
                err.code() as PJRT_Error_Code,
                &format!("{:?}", err),
            )
        },
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
