use pjrt_sys::{
    PJRT_Error_Code_PJRT_Error_Code_ABORTED, PJRT_Error_Code_PJRT_Error_Code_ALREADY_EXISTS,
    PJRT_Error_Code_PJRT_Error_Code_CANCELLED, PJRT_Error_Code_PJRT_Error_Code_DATA_LOSS,
    PJRT_Error_Code_PJRT_Error_Code_DEADLINE_EXCEEDED,
    PJRT_Error_Code_PJRT_Error_Code_FAILED_PRECONDITION, PJRT_Error_Code_PJRT_Error_Code_INTERNAL,
    PJRT_Error_Code_PJRT_Error_Code_INVALID_ARGUMENT, PJRT_Error_Code_PJRT_Error_Code_NOT_FOUND,
    PJRT_Error_Code_PJRT_Error_Code_OUT_OF_RANGE,
    PJRT_Error_Code_PJRT_Error_Code_PERMISSION_DENIED,
    PJRT_Error_Code_PJRT_Error_Code_RESOURCE_EXHAUSTED,
    PJRT_Error_Code_PJRT_Error_Code_UNAUTHENTICATED, PJRT_Error_Code_PJRT_Error_Code_UNAVAILABLE,
    PJRT_Error_Code_PJRT_Error_Code_UNIMPLEMENTED, PJRT_Error_Code_PJRT_Error_Code_UNKNOWN,
};

use crate::PrimitiveType;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("pjrt error {msg}\n{backtrace}")]
    PjrtError {
        msg: String,
        code: ErrorCode,
        backtrace: String,
    },

    #[error("null function pointer: {0}")]
    NullFunctionPointer(&'static str),

    #[error("no addressable device")]
    NoAddressableDevice,

    #[error("invalid primitive type: {0}")]
    InvalidPrimitiveType(u32),

    #[error("invalid errro code: {0}")]
    InvalidErrorCode(u32),

    #[error("invalid memory layout type: {0}")]
    InvalidMemoryLayoutType(u32),

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
}

pub type Result<T> = std::result::Result<T, Error>;

#[repr(u32)]
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum ErrorCode {
    Cancel = PJRT_Error_Code_PJRT_Error_Code_CANCELLED,
    Unknown = PJRT_Error_Code_PJRT_Error_Code_UNKNOWN,
    InvalidArgument = PJRT_Error_Code_PJRT_Error_Code_INVALID_ARGUMENT,
    DeadlineExceeded = PJRT_Error_Code_PJRT_Error_Code_DEADLINE_EXCEEDED,
    NotFound = PJRT_Error_Code_PJRT_Error_Code_NOT_FOUND,
    AlreadyExists = PJRT_Error_Code_PJRT_Error_Code_ALREADY_EXISTS,
    PermissionDenied = PJRT_Error_Code_PJRT_Error_Code_PERMISSION_DENIED,
    ResourceExhaused = PJRT_Error_Code_PJRT_Error_Code_RESOURCE_EXHAUSTED,
    FailedPrecondition = PJRT_Error_Code_PJRT_Error_Code_FAILED_PRECONDITION,
    Aborted = PJRT_Error_Code_PJRT_Error_Code_ABORTED,
    OutOfRange = PJRT_Error_Code_PJRT_Error_Code_OUT_OF_RANGE,
    Unimplemeted = PJRT_Error_Code_PJRT_Error_Code_UNIMPLEMENTED,
    Internal = PJRT_Error_Code_PJRT_Error_Code_INTERNAL,
    Unavaliable = PJRT_Error_Code_PJRT_Error_Code_UNAVAILABLE,
    DataLoss = PJRT_Error_Code_PJRT_Error_Code_DATA_LOSS,
    Unauthenticated = PJRT_Error_Code_PJRT_Error_Code_UNAUTHENTICATED,
}

impl TryFrom<u32> for ErrorCode {
    type Error = Error;
    #[allow(non_upper_case_globals)]
    fn try_from(code: u32) -> Result<Self> {
        match code {
            PJRT_Error_Code_PJRT_Error_Code_CANCELLED => Ok(Self::Cancel),
            PJRT_Error_Code_PJRT_Error_Code_UNKNOWN => Ok(Self::Unknown),
            PJRT_Error_Code_PJRT_Error_Code_INVALID_ARGUMENT => Ok(Self::InvalidArgument),
            PJRT_Error_Code_PJRT_Error_Code_DEADLINE_EXCEEDED => Ok(Self::DeadlineExceeded),
            PJRT_Error_Code_PJRT_Error_Code_NOT_FOUND => Ok(Self::NotFound),
            PJRT_Error_Code_PJRT_Error_Code_ALREADY_EXISTS => Ok(Self::AlreadyExists),
            PJRT_Error_Code_PJRT_Error_Code_PERMISSION_DENIED => Ok(Self::PermissionDenied),
            PJRT_Error_Code_PJRT_Error_Code_RESOURCE_EXHAUSTED => Ok(Self::ResourceExhaused),
            PJRT_Error_Code_PJRT_Error_Code_FAILED_PRECONDITION => Ok(Self::FailedPrecondition),
            PJRT_Error_Code_PJRT_Error_Code_ABORTED => Ok(Self::Aborted),
            PJRT_Error_Code_PJRT_Error_Code_OUT_OF_RANGE => Ok(Self::OutOfRange),
            PJRT_Error_Code_PJRT_Error_Code_UNIMPLEMENTED => Ok(Self::Unimplemeted),
            PJRT_Error_Code_PJRT_Error_Code_INTERNAL => Ok(Self::Internal),
            PJRT_Error_Code_PJRT_Error_Code_UNAVAILABLE => Ok(Self::Unavaliable),
            PJRT_Error_Code_PJRT_Error_Code_DATA_LOSS => Ok(Self::DataLoss),
            PJRT_Error_Code_PJRT_Error_Code_UNAUTHENTICATED => Ok(Self::Unauthenticated),
            _ => Err(Error::InvalidErrorCode(code)),
        }
    }
}
