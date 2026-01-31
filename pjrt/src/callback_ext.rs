//! PJRT Callback Extension
//!
//! This module provides safe Rust bindings for the PJRT Callback extension.
//! The Callback extension provides functionality for registering custom callbacks
//! that can be invoked by the PJRT runtime for various events.
//!
//! ## Usage
//!
//! ```rust,ignore
//! use pjrt::callback::{CallbackExtension, CallbackType, TpuSliceFailureType};
//!
//! // Get the callback extension
//! let callback_ext = client.extension::<CallbackExtension>()?;
//!
//! // Register a callback
//! callback_ext.register_callback(
//!     CallbackType::TpuSliceBuilder,
//!     |args, user_arg| {
//!         // Handle the callback
//!     },
//!     user_data
//! )?;
//! ```

use std::ffi::c_void;
use std::rc::Rc;

use pjrt_sys::{
    PJRT_Callback_Extension, PJRT_Callback_InvokeCallback_Args,
    PJRT_Callback_RegisterCallback_Args, PJRT_Callback_Tpu_SliceBuilderArgs,
    PJRT_Callback_Tpu_SliceFailureType, PJRT_Callback_Type, PJRT_Client,
};

use crate::extension::{Extension, ExtensionType};
use crate::{Api, Client, Result};

/// Safe wrapper for PJRT Callback extension
///
/// The Callback extension provides functionality for registering custom callbacks
/// that can be invoked by the PJRT runtime for various events like TPU slice failures
/// or pre-fatal errors.
pub struct CallbackExtension {
    raw: Rc<PJRT_Callback_Extension>,
    api: Api,
}

impl std::fmt::Debug for CallbackExtension {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CallbackExtension")
            .field("api_version", &1i32)
            .finish()
    }
}

unsafe impl Extension for CallbackExtension {
    fn extension_type() -> ExtensionType {
        ExtensionType::Callback
    }

    unsafe fn from_raw(ptr: *mut pjrt_sys::PJRT_Extension_Base, api: &Api) -> Option<Self>
    where
        Self: Sized,
    {
        if ptr.is_null() {
            return None;
        }

        let ext = ptr as *mut PJRT_Callback_Extension;
        if (*ext).base.type_ != ExtensionType::Callback.to_raw() {
            return None;
        }

        Some(Self {
            raw: Rc::new((*ext).clone()),
            api: api.clone(),
        })
    }
}

/// Types of callbacks that can be registered
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CallbackType {
    /// Unknown callback type
    Unknown,
    /// TPU SliceBuilder callback for handling slice failures
    TpuSliceBuilder,
    /// Pre-fatal callback invoked before fatal errors
    Prefatal,
}

impl CallbackType {
    fn to_raw(&self) -> PJRT_Callback_Type {
        match self {
            CallbackType::Unknown => pjrt_sys::PJRT_Callback_Type_PJRT_Callback_Type_Unknown,
            CallbackType::TpuSliceBuilder => {
                pjrt_sys::PJRT_Callback_Type_PJRT_Callback_Type_Tpu_SliceBuilder
            }
            CallbackType::Prefatal => pjrt_sys::PJRT_Callback_Type_PJRT_Callback_Type_Prefatal,
        }
    }
}

/// Types of TPU slice failures
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TpuSliceFailureType {
    /// Unknown failure type
    Unknown,
    /// Failure during slice initialization (ICI network config) phase
    InitError,
    /// Worker disconnected after heartbeat check threshold
    WorkerUnavailable,
    /// Flapping task error from fast task restart
    FlappingTaskError,
    /// Software injected error for testing
    SoftwareInjectedError,
    /// Chip driver error (bad TPU or ICI link down)
    ChipDriverError,
}

impl TpuSliceFailureType {
    fn from_raw(raw: PJRT_Callback_Tpu_SliceFailureType) -> Self {
        match raw {
            pjrt_sys::PJRT_Callback_Tpu_SliceFailureType_SLICE_FAILURE_INIT_ERROR => {
                Self::InitError
            }
            pjrt_sys::PJRT_Callback_Tpu_SliceFailureType_SLICE_FAILURE_WORKER_UNAVAILABLE => {
                Self::WorkerUnavailable
            }
            pjrt_sys::PJRT_Callback_Tpu_SliceFailureType_SLICE_FAILURE_FLAPPING_TASK_ERROR => {
                Self::FlappingTaskError
            }
            pjrt_sys::PJRT_Callback_Tpu_SliceFailureType_SLICE_FAILURE_SW_INJECT_ERROR => {
                Self::SoftwareInjectedError
            }
            pjrt_sys::PJRT_Callback_Tpu_SliceFailureType_SLICE_FAILURE_CHIP_DRIVER_ERROR => {
                Self::ChipDriverError
            }
            _ => Self::Unknown,
        }
    }
}

/// Arguments passed to a TPU SliceBuilder callback
pub struct TpuSliceBuilderCallbackArgs {
    /// The type of failure that occurred
    pub failure_type: TpuSliceFailureType,
}

impl TpuSliceBuilderCallbackArgs {
    fn from_raw(raw: &PJRT_Callback_Tpu_SliceBuilderArgs) -> Self {
        Self {
            failure_type: TpuSliceFailureType::from_raw(raw.failure_type),
        }
    }
}

/// Callback function type
pub type CallbackFn = Box<dyn Fn(*mut c_void, *mut c_void)>;

impl CallbackExtension {
    /// Register a callback for a specific callback type
    ///
    /// # Arguments
    ///
    /// * `callback_type` - The type of callback to register
    /// * `callback` - The callback function pointer (extern "C" fn)
    /// * `user_arg` - User-provided argument passed to the callback
    ///
    /// # Safety
    ///
    /// The callback function must remain valid for the lifetime of the client.
    /// The user_arg must also remain valid if it's passed to the callback.
    pub unsafe fn register_callback(
        &self,
        client: &Client,
        callback_type: CallbackType,
        callback: unsafe extern "C" fn(*mut c_void, *mut c_void),
        user_arg: *mut c_void,
    ) -> Result<()> {
        let mut args = PJRT_Callback_RegisterCallback_Args {
            struct_size: std::mem::size_of::<PJRT_Callback_RegisterCallback_Args>(),
            client: client.ptr() as *mut PJRT_Client,
            type_: callback_type.to_raw(),
            callback: Some(callback),
            user_arg,
        };

        let ext_fn = self
            .raw
            .register_callback
            .expect("PJRT_Register_Callback not implemented");

        let err = ext_fn(&mut args);
        self.api.err_or(err, ())
    }

    /// Invoke a callback of a specific type
    ///
    /// # Arguments
    ///
    /// * `client` - The PJRT client
    /// * `callback_type` - The type of callback to invoke
    /// * `callback_args` - Type-specific arguments for the callback
    ///
    /// # Safety
    ///
    /// The callback_args pointer must be valid for the duration of the call.
    pub unsafe fn invoke_callback(
        &self,
        client: &Client,
        callback_type: CallbackType,
        callback_args: *mut c_void,
    ) -> Result<()> {
        let mut args = PJRT_Callback_InvokeCallback_Args {
            struct_size: std::mem::size_of::<PJRT_Callback_InvokeCallback_Args>(),
            client: client.ptr() as *mut PJRT_Client,
            type_: callback_type.to_raw(),
            args: callback_args,
        };

        let ext_fn = self
            .raw
            .invoke_callback
            .expect("PJRT_Callback_InvokeCallback not implemented");

        let err = ext_fn(&mut args);
        self.api.err_or(err, ())
    }
}

/// Extension trait for accessing Callback extension from Client
pub trait CallbackExt {
    /// Get the Callback extension if available
    fn callback_extension(&self) -> Option<CallbackExtension>;
}
