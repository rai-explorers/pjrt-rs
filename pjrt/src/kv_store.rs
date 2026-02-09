use std::os::raw::c_char;

use pjrt_sys::{
    PJRT_Error, PJRT_Error_Code, PJRT_Error_Code_PJRT_Error_Code_INVALID_ARGUMENT,
    PJRT_Error_Code_PJRT_Error_Code_NOT_FOUND, PJRT_KeyValueGetCallback_Args,
    PJRT_KeyValuePutCallback_Args, PJRT_KeyValueTryGetCallback_Args,
};

use crate::Result;

// ---------------------------------------------------------------------------
// Value allocation helpers for returning binary data through the C API
// ---------------------------------------------------------------------------
//
// The PJRT value_deleter_callback receives only a `*mut c_char` pointer and no
// size information.  To support arbitrary binary values (which may contain null
// bytes) we use a length-prefixed allocation:
//
//   [ usize length ][ u8 data ... ]
//                    ^— pointer returned to PJRT
//
// The deleter reads the length from before the pointer to reconstruct the
// `Layout` and deallocate.

const HEADER: usize = std::mem::size_of::<usize>();

/// Allocate a length-prefixed buffer and copy `data` into it.
///
/// Returns a pointer to the *data* region (past the length header) and the byte
/// count.  The pointer is suitable for `value_deleter_callback`.
fn alloc_callback_value(data: &[u8]) -> (*mut c_char, usize) {
    if data.is_empty() {
        return (std::ptr::null_mut(), 0);
    }
    let total = HEADER + data.len();
    // SAFETY: alignment of usize is always valid and total > 0.
    let layout = std::alloc::Layout::from_size_align(total, std::mem::align_of::<usize>()).unwrap();
    let ptr = unsafe { std::alloc::alloc(layout) };
    if ptr.is_null() {
        std::alloc::handle_alloc_error(layout);
    }
    unsafe {
        (ptr as *mut usize).write(data.len());
        std::ptr::copy_nonoverlapping(data.as_ptr(), ptr.add(HEADER), data.len());
    }
    (unsafe { ptr.add(HEADER) as *mut c_char }, data.len())
}

/// # Safety
///
/// `value` must have been allocated by `alloc_callback_value`, or be null.
unsafe extern "C" fn value_deleter_callback(value: *mut c_char) {
    if !value.is_null() {
        let base = (value as *mut u8).sub(HEADER);
        let len = (base as *const usize).read();
        let total = HEADER + len;
        let layout =
            std::alloc::Layout::from_size_align_unchecked(total, std::mem::align_of::<usize>());
        std::alloc::dealloc(base, layout);
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

    let key = unsafe {
        let bytes = std::slice::from_raw_parts(args.key as *const u8, args.key_size);
        String::from_utf8_lossy(bytes)
    };
    args.value_deleter_callback = Some(value_deleter_callback);

    match store.get(&key, args.timeout_in_ms) {
        Ok(value) => {
            let (ptr, len) = alloc_callback_value(&value);
            args.value = ptr;
            args.value_size = len;
            std::ptr::null_mut()
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

    let key = unsafe {
        let bytes = std::slice::from_raw_parts(args.key as *const u8, args.key_size);
        String::from_utf8_lossy(bytes)
    };
    let value = unsafe { std::slice::from_raw_parts(args.value as *const u8, args.value_size) };

    match store.put(&key, value) {
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

    let key = unsafe {
        let bytes = std::slice::from_raw_parts(args.key as *const u8, args.key_size);
        String::from_utf8_lossy(bytes)
    };
    args.value_deleter_callback = Some(value_deleter_callback);

    match store.try_get(&key) {
        Ok(Some(value)) => {
            let (ptr, len) = alloc_callback_value(&value);
            args.value = ptr;
            args.value_size = len;
            std::ptr::null_mut()
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

/// A key-value store trait for distributed PJRT coordination.
///
/// Values are opaque byte slices (`Vec<u8>` / `&[u8]`) rather than strings
/// because the PJRT runtime may store arbitrary binary data (e.g. serialised
/// executables or metadata).
pub trait KeyValueStore {
    /// Retrieve the value for `key`, blocking up to `timeout_in_ms` milliseconds.
    fn get(&self, key: &str, timeout_in_ms: i32) -> Result<Vec<u8>>;
    /// Store a value under `key`.
    fn put(&self, key: &str, value: &[u8]) -> Result<()>;
    /// Try to get a value from the key-value store (non-blocking).
    /// Returns `Ok(Some(value))` if the key exists.
    /// Returns `Ok(None)` if the key does not exist.
    /// Returns `Err` for other errors.
    fn try_get(&self, key: &str) -> Result<Option<Vec<u8>>>;
}
