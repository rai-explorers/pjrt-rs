//! PJRT Error Handling
//!
//! This module provides comprehensive error handling for PJRT operations.
//! It defines error types that map directly to PJRT's error codes and provides
//! detailed error information including:
//!
//! - Error codes matching PJRT's canonical error space
//! - Detailed error messages
//! - The function name where the error occurred
//! - Stack traces for debugging
//! - Rust-style Result types for idiomatic error handling
//!
//! All PJRT operations return `Result<T>` which is either `Ok(T)` on success
//! or `Err(Error)` on failure.

#![allow(unused_assignments)]

use pjrt_sys::{
    PJRT_Error_Code, PJRT_Error_Code_PJRT_Error_Code_ABORTED,
    PJRT_Error_Code_PJRT_Error_Code_ALREADY_EXISTS, PJRT_Error_Code_PJRT_Error_Code_CANCELLED,
    PJRT_Error_Code_PJRT_Error_Code_DATA_LOSS, PJRT_Error_Code_PJRT_Error_Code_DEADLINE_EXCEEDED,
    PJRT_Error_Code_PJRT_Error_Code_FAILED_PRECONDITION, PJRT_Error_Code_PJRT_Error_Code_INTERNAL,
    PJRT_Error_Code_PJRT_Error_Code_INVALID_ARGUMENT, PJRT_Error_Code_PJRT_Error_Code_NOT_FOUND,
    PJRT_Error_Code_PJRT_Error_Code_OUT_OF_RANGE,
    PJRT_Error_Code_PJRT_Error_Code_PERMISSION_DENIED,
    PJRT_Error_Code_PJRT_Error_Code_RESOURCE_EXHAUSTED,
    PJRT_Error_Code_PJRT_Error_Code_UNAUTHENTICATED, PJRT_Error_Code_PJRT_Error_Code_UNAVAILABLE,
    PJRT_Error_Code_PJRT_Error_Code_UNIMPLEMENTED, PJRT_Error_Code_PJRT_Error_Code_UNKNOWN,
};

use crate::{GlobalDeviceId, PrimitiveType};

/// Error type for PJRT operations.
///
/// This enum represents all possible errors that can occur when using the PJRT API.
/// Errors include both PJRT-specific errors (returned by the C API) and Rust-side
/// errors (e.g., invalid arguments, null pointers).
#[derive(thiserror::Error, Debug)]
pub enum Error {
    /// An error returned by the PJRT C API.
    ///
    /// This variant includes:
    /// - `function`: The PJRT function that returned the error
    /// - `msg`: The error message from PJRT
    /// - `code`: The PJRT error code
    /// - `backtrace`: A captured Rust backtrace for debugging
    #[error("{function}: {msg} (code: {code:?})\n{backtrace}")]
    #[allow(unused_assignments)]
    PjrtError {
        /// The PJRT function that returned this error
        function: &'static str,
        /// The error message from PJRT
        msg: String,
        /// The error code
        code: ErrorCode,
        /// A captured backtrace
        backtrace: String,
    },

    #[error("null function pointer: {0}")]
    NullFunctionPointer(&'static str),

    #[error("invalid argument: {0}")]
    InvalidArgument(String),

    #[error("no addressable device")]
    NoAddressableDevice,

    #[error("invalid primitive type: {0}")]
    InvalidPrimitiveType(i32),

    #[error("invalid error code: {0}")]
    InvalidErrorCode(i32),

    #[error("invalid memory layout type: {0}")]
    InvalidMemoryLayoutType(i32),

    #[error("invalid named value type: {0}")]
    InvalidNamedValueType(i32),

    #[error("device not in device assignment: {0}")]
    DeviceNotInDeviceAssignment(GlobalDeviceId),

    #[error("invalid program format: {0}")]
    InvalidProgramFormat(String),

    #[error("not supported type: {0:?}")]
    NotSupportedType(PrimitiveType),

    #[error("null pointer")]
    NullPointer,

    #[error("plugin not found: {0}")]
    PluginNotFound(String),

    #[error("io error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("lib loading error: {0}")]
    LibLoadingError(#[from] libloading::Error),

    #[error("lock poison error: {0}")]
    PoisonError(String),

    #[error("unimplemented")]
    Unimplemented,
}

impl Error {
    /// Returns the PJRT error code associated with this error.
    ///
    /// For `PjrtError` variants, returns the actual PJRT error code.
    /// For other variants, returns `ErrorCode::Internal`.
    pub fn code(&self) -> ErrorCode {
        match self {
            Error::PjrtError { code, .. } => *code,
            _ => ErrorCode::Internal,
        }
    }

    /// Returns the PJRT function name that caused this error, if available.
    pub fn function(&self) -> Option<&'static str> {
        match self {
            Error::PjrtError { function, .. } => Some(function),
            Error::NullFunctionPointer(name) => Some(name),
            _ => None,
        }
    }
}

pub type Result<T> = std::result::Result<T, Error>;

#[repr(i32)]
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum ErrorCode {
    Cancel = PJRT_Error_Code_PJRT_Error_Code_CANCELLED as i32,
    Unknown = PJRT_Error_Code_PJRT_Error_Code_UNKNOWN as i32,
    InvalidArgument = PJRT_Error_Code_PJRT_Error_Code_INVALID_ARGUMENT as i32,
    DeadlineExceeded = PJRT_Error_Code_PJRT_Error_Code_DEADLINE_EXCEEDED as i32,
    NotFound = PJRT_Error_Code_PJRT_Error_Code_NOT_FOUND as i32,
    AlreadyExists = PJRT_Error_Code_PJRT_Error_Code_ALREADY_EXISTS as i32,
    PermissionDenied = PJRT_Error_Code_PJRT_Error_Code_PERMISSION_DENIED as i32,
    ResourceExhausted = PJRT_Error_Code_PJRT_Error_Code_RESOURCE_EXHAUSTED as i32,
    FailedPrecondition = PJRT_Error_Code_PJRT_Error_Code_FAILED_PRECONDITION as i32,
    Aborted = PJRT_Error_Code_PJRT_Error_Code_ABORTED as i32,
    OutOfRange = PJRT_Error_Code_PJRT_Error_Code_OUT_OF_RANGE as i32,
    Unimplemented = PJRT_Error_Code_PJRT_Error_Code_UNIMPLEMENTED as i32,
    Internal = PJRT_Error_Code_PJRT_Error_Code_INTERNAL as i32,
    Unavailable = PJRT_Error_Code_PJRT_Error_Code_UNAVAILABLE as i32,
    DataLoss = PJRT_Error_Code_PJRT_Error_Code_DATA_LOSS as i32,
    Unauthenticated = PJRT_Error_Code_PJRT_Error_Code_UNAUTHENTICATED as i32,
}

impl TryFrom<PJRT_Error_Code> for ErrorCode {
    type Error = Error;
    #[allow(non_upper_case_globals)]
    #[allow(non_snake_case)]
    fn try_from(code: PJRT_Error_Code) -> Result<Self> {
        match code {
            PJRT_Error_Code_PJRT_Error_Code_CANCELLED => Ok(Self::Cancel),
            PJRT_Error_Code_PJRT_Error_Code_UNKNOWN => Ok(Self::Unknown),
            PJRT_Error_Code_PJRT_Error_Code_INVALID_ARGUMENT => Ok(Self::InvalidArgument),
            PJRT_Error_Code_PJRT_Error_Code_DEADLINE_EXCEEDED => Ok(Self::DeadlineExceeded),
            PJRT_Error_Code_PJRT_Error_Code_NOT_FOUND => Ok(Self::NotFound),
            PJRT_Error_Code_PJRT_Error_Code_ALREADY_EXISTS => Ok(Self::AlreadyExists),
            PJRT_Error_Code_PJRT_Error_Code_PERMISSION_DENIED => Ok(Self::PermissionDenied),
            PJRT_Error_Code_PJRT_Error_Code_RESOURCE_EXHAUSTED => Ok(Self::ResourceExhausted),
            PJRT_Error_Code_PJRT_Error_Code_FAILED_PRECONDITION => Ok(Self::FailedPrecondition),
            PJRT_Error_Code_PJRT_Error_Code_ABORTED => Ok(Self::Aborted),
            PJRT_Error_Code_PJRT_Error_Code_OUT_OF_RANGE => Ok(Self::OutOfRange),
            PJRT_Error_Code_PJRT_Error_Code_UNIMPLEMENTED => Ok(Self::Unimplemented),
            PJRT_Error_Code_PJRT_Error_Code_INTERNAL => Ok(Self::Internal),
            PJRT_Error_Code_PJRT_Error_Code_UNAVAILABLE => Ok(Self::Unavailable),
            PJRT_Error_Code_PJRT_Error_Code_DATA_LOSS => Ok(Self::DataLoss),
            PJRT_Error_Code_PJRT_Error_Code_UNAUTHENTICATED => Ok(Self::Unauthenticated),
            _ => Err(Error::InvalidErrorCode(code as i32)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_code_from_pjrt_error_code() {
        use pjrt_sys::{
            PJRT_Error_Code_PJRT_Error_Code_CANCELLED, PJRT_Error_Code_PJRT_Error_Code_INTERNAL,
            PJRT_Error_Code_PJRT_Error_Code_INVALID_ARGUMENT,
            PJRT_Error_Code_PJRT_Error_Code_UNKNOWN,
        };

        let code: ErrorCode = PJRT_Error_Code_PJRT_Error_Code_CANCELLED
            .try_into()
            .unwrap();
        assert_eq!(code, ErrorCode::Cancel);

        let code: ErrorCode = PJRT_Error_Code_PJRT_Error_Code_UNKNOWN.try_into().unwrap();
        assert_eq!(code, ErrorCode::Unknown);

        let code: ErrorCode = PJRT_Error_Code_PJRT_Error_Code_INVALID_ARGUMENT
            .try_into()
            .unwrap();
        assert_eq!(code, ErrorCode::InvalidArgument);

        let code: ErrorCode = PJRT_Error_Code_PJRT_Error_Code_INTERNAL.try_into().unwrap();
        assert_eq!(code, ErrorCode::Internal);
    }

    #[test]
    fn test_error_code_from_invalid_pjrt_error_code() {
        // Test with an invalid error code using the raw type
        use pjrt_sys::PJRT_Error_Code;
        let invalid_code: PJRT_Error_Code = 9999;
        let result: Result<ErrorCode> = invalid_code.try_into();
        assert!(result.is_err());
        match result {
            Err(Error::InvalidErrorCode(code)) => assert_eq!(code, 9999),
            _ => panic!("Expected InvalidErrorCode error"),
        }
    }

    #[test]
    fn test_error_code_values() {
        assert_eq!(
            ErrorCode::Cancel as i32,
            PJRT_Error_Code_PJRT_Error_Code_CANCELLED as i32
        );
        assert_eq!(
            ErrorCode::Unknown as i32,
            PJRT_Error_Code_PJRT_Error_Code_UNKNOWN as i32
        );
        assert_eq!(
            ErrorCode::InvalidArgument as i32,
            PJRT_Error_Code_PJRT_Error_Code_INVALID_ARGUMENT as i32
        );
        assert_eq!(
            ErrorCode::DeadlineExceeded as i32,
            PJRT_Error_Code_PJRT_Error_Code_DEADLINE_EXCEEDED as i32
        );
        assert_eq!(
            ErrorCode::NotFound as i32,
            PJRT_Error_Code_PJRT_Error_Code_NOT_FOUND as i32
        );
        assert_eq!(
            ErrorCode::AlreadyExists as i32,
            PJRT_Error_Code_PJRT_Error_Code_ALREADY_EXISTS as i32
        );
        assert_eq!(
            ErrorCode::PermissionDenied as i32,
            PJRT_Error_Code_PJRT_Error_Code_PERMISSION_DENIED as i32
        );
        assert_eq!(
            ErrorCode::ResourceExhausted as i32,
            PJRT_Error_Code_PJRT_Error_Code_RESOURCE_EXHAUSTED as i32
        );
        assert_eq!(
            ErrorCode::FailedPrecondition as i32,
            PJRT_Error_Code_PJRT_Error_Code_FAILED_PRECONDITION as i32
        );
        assert_eq!(
            ErrorCode::Aborted as i32,
            PJRT_Error_Code_PJRT_Error_Code_ABORTED as i32
        );
        assert_eq!(
            ErrorCode::OutOfRange as i32,
            PJRT_Error_Code_PJRT_Error_Code_OUT_OF_RANGE as i32
        );
        assert_eq!(
            ErrorCode::Unimplemented as i32,
            PJRT_Error_Code_PJRT_Error_Code_UNIMPLEMENTED as i32
        );
        assert_eq!(
            ErrorCode::Internal as i32,
            PJRT_Error_Code_PJRT_Error_Code_INTERNAL as i32
        );
        assert_eq!(
            ErrorCode::Unavailable as i32,
            PJRT_Error_Code_PJRT_Error_Code_UNAVAILABLE as i32
        );
        assert_eq!(
            ErrorCode::DataLoss as i32,
            PJRT_Error_Code_PJRT_Error_Code_DATA_LOSS as i32
        );
        assert_eq!(
            ErrorCode::Unauthenticated as i32,
            PJRT_Error_Code_PJRT_Error_Code_UNAUTHENTICATED as i32
        );
    }

    #[test]
    fn test_error_code_method() {
        let pjrt_err = Error::PjrtError {
            function: "test_function",
            msg: "test error".to_string(),
            code: ErrorCode::InvalidArgument,
            backtrace: String::new(),
        };
        assert_eq!(pjrt_err.code(), ErrorCode::InvalidArgument);

        let null_fp_err = Error::NullFunctionPointer("test_fn");
        assert_eq!(null_fp_err.code(), ErrorCode::Internal);

        let io_err = Error::IoError(std::io::Error::other("io error"));
        assert_eq!(io_err.code(), ErrorCode::Internal);
    }

    #[test]
    fn test_error_display() {
        let err = Error::InvalidArgument("bad arg".to_string());
        let display = format!("{}", err);
        assert!(display.contains("invalid argument: bad arg"));

        let err = Error::NullFunctionPointer("PJRT_Test");
        let display = format!("{}", err);
        assert!(display.contains("null function pointer: PJRT_Test"));

        let err = Error::NoAddressableDevice;
        let display = format!("{}", err);
        assert!(display.contains("no addressable device"));

        let err = Error::InvalidPrimitiveType(999);
        let display = format!("{}", err);
        assert!(display.contains("invalid primitive type: 999"));

        let err = Error::InvalidErrorCode(888);
        let display = format!("{}", err);
        assert!(display.contains("invalid error code: 888"));

        let err = Error::InvalidMemoryLayoutType(777);
        let display = format!("{}", err);
        assert!(display.contains("invalid memory layout type: 777"));

        let err = Error::DeviceNotInDeviceAssignment(42i32);
        let display = format!("{}", err);
        assert!(display.contains("device not in device assignment: 42"));

        let err = Error::InvalidProgramFormat("unknown".to_string());
        let display = format!("{}", err);
        assert!(display.contains("invalid program format: unknown"));

        let err = Error::NotSupportedType(PrimitiveType::F8E5M2);
        let display = format!("{}", err);
        assert!(display.contains("not supported type"));

        let err = Error::NullPointer;
        let display = format!("{}", err);
        assert!(display.contains("null pointer"));

        let err = Error::PluginNotFound("test_plugin".to_string());
        let display = format!("{}", err);
        assert!(display.contains("plugin not found: test_plugin"));

        let err = Error::Unimplemented;
        let display = format!("{}", err);
        assert!(display.contains("unimplemented"));

        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let err = Error::IoError(io_err);
        let display = format!("{}", err);
        assert!(display.contains("io error"));
    }

    #[test]
    fn test_error_from_io_error() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "test");
        let err: Error = io_err.into();
        match err {
            Error::IoError(_) => (),
            _ => panic!("Expected IoError variant"),
        }
    }

    #[test]
    fn test_result_type() {
        fn returns_ok() -> Result<i32> {
            Ok(42)
        }

        fn returns_err() -> Result<i32> {
            Err(Error::NullPointer)
        }

        assert_eq!(returns_ok().unwrap(), 42);
        assert!(returns_err().is_err());
    }
}
