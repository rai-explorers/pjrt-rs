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
    InvalidPrimitiveType(i32),

    #[error("invalid errro code: {0}")]
    InvalidErrorCode(i32),

    #[error("invalid memory layout type: {0}")]
    InvalidMemoryLayoutType(i32),

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
    Unimplemeted,
}

impl Error {
    pub fn code(&self) -> ErrorCode {
        match self {
            Error::PjrtError { code, .. } => *code,
            _ => ErrorCode::Internal,
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
    ResourceExhaused = PJRT_Error_Code_PJRT_Error_Code_RESOURCE_EXHAUSTED as i32,
    FailedPrecondition = PJRT_Error_Code_PJRT_Error_Code_FAILED_PRECONDITION as i32,
    Aborted = PJRT_Error_Code_PJRT_Error_Code_ABORTED as i32,
    OutOfRange = PJRT_Error_Code_PJRT_Error_Code_OUT_OF_RANGE as i32,
    Unimplemeted = PJRT_Error_Code_PJRT_Error_Code_UNIMPLEMENTED as i32,
    Internal = PJRT_Error_Code_PJRT_Error_Code_INTERNAL as i32,
    Unavaliable = PJRT_Error_Code_PJRT_Error_Code_UNAVAILABLE as i32,
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
            PJRT_Error_Code_PJRT_Error_Code_RESOURCE_EXHAUSTED => Ok(Self::ResourceExhaused),
            PJRT_Error_Code_PJRT_Error_Code_FAILED_PRECONDITION => Ok(Self::FailedPrecondition),
            PJRT_Error_Code_PJRT_Error_Code_ABORTED => Ok(Self::Aborted),
            PJRT_Error_Code_PJRT_Error_Code_OUT_OF_RANGE => Ok(Self::OutOfRange),
            PJRT_Error_Code_PJRT_Error_Code_UNIMPLEMENTED => Ok(Self::Unimplemeted),
            PJRT_Error_Code_PJRT_Error_Code_INTERNAL => Ok(Self::Internal),
            PJRT_Error_Code_PJRT_Error_Code_UNAVAILABLE => Ok(Self::Unavaliable),
            PJRT_Error_Code_PJRT_Error_Code_DATA_LOSS => Ok(Self::DataLoss),
            PJRT_Error_Code_PJRT_Error_Code_UNAUTHENTICATED => Ok(Self::Unauthenticated),
            _ => Err(Error::InvalidErrorCode(code as i32)),
        }
    }
}
