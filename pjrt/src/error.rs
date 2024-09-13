use crate::PrimitiveType;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("pjrt error {msg}\n{backtrace}")]
    PjrtError { msg: String, backtrace: String },

    #[error("null function pointer: {0}")]
    NullFunctionPointer(&'static str),

    #[error("no addressable device")]
    NoAddressableDevice,

    #[error("invalid primitive type: {0}")]
    InvalidPrimitiveType(u32),

    #[error("invalid memory layout type: {0}")]
    InvalidMemoryLayoutType(u32),

    #[error("invalid program format: {0}")]
    InvalidProgramFormat(String),

    #[error("not supported type: {0:?}")]
    NotSupportedType(PrimitiveType),

    #[error("not implemented")]
    NotImplemented,

    #[error("null pointer")]
    NullPointer,

    #[error("plugin not found: {0}")]
    PluginNotFound(String),

    #[error("io error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("lib loading error: {0}")]
    LibLoadingError(#[from] libloading::Error),
}

pub type Result<T> = std::result::Result<T, Error>;
