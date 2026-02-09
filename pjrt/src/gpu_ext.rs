//! PJRT GPU Extension
//!
//! This module provides safe Rust bindings for the PJRT GPU extension.
//! The GPU extension provides capabilities for registering custom calls
//! for GPU-specific operations.
//!
//! ## Usage
//!
//! ```rust,ignore
//! use pjrt::gpu::{GpuExtension, CustomCallApiVersion};
//!
//! // Get the GPU extension
//! let gpu_ext = api.get_extension::<GpuExtension>()?;
//!
//! // Register a typed custom call handler
//! gpu_ext.register_custom_call(
//!     "my_custom_op",
//!     CustomCallApiVersion::Typed,
//!     None, // instantiate
//!     None, // prepare
//!     None, // initialize
//!     Some(handler_execute),
//! )?;
//! ```

use std::ffi::CString;
use std::rc::Rc;

use pjrt_sys::{PJRT_Gpu_Custom_Call, PJRT_Gpu_Register_Custom_Call_Args};

use crate::extension::{Extension, ExtensionType};
use crate::{Api, Error, Result};

/// Safe wrapper for PJRT GPU extension
///
/// The GPU extension provides capabilities for registering custom calls
/// for GPU-specific operations.
pub struct GpuExtension {
    raw: Rc<PJRT_Gpu_Custom_Call>,
    api: Api,
}

impl std::fmt::Debug for GpuExtension {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GpuExtension").finish()
    }
}

unsafe impl Extension for GpuExtension {
    fn extension_type() -> ExtensionType {
        ExtensionType::GpuCustomCall
    }

    unsafe fn from_raw(ptr: *mut pjrt_sys::PJRT_Extension_Base, api: &Api) -> Option<Self>
    where
        Self: Sized,
    {
        if ptr.is_null() {
            return None;
        }

        let gpu_ext = ptr as *mut PJRT_Gpu_Custom_Call;
        if (*gpu_ext).base.type_ != ExtensionType::GpuCustomCall.to_raw() {
            return None;
        }

        Some(Self {
            raw: Rc::new(*gpu_ext),
            api: api.clone(),
        })
    }
}

/// API version for custom calls
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CustomCallApiVersion {
    /// Untyped custom call (version 0)
    Untyped = 0,
    /// Typed custom call (version 1)
    Typed = 1,
}

/// Opaque custom call handler type
pub type CustomCallHandler = *mut std::ffi::c_void;

impl GpuExtension {
    /// Register a custom call handler
    ///
    /// Registers a custom call handler with the specified function name and
    /// API version. The handler functions are called at different stages of
    /// the custom call lifecycle.
    ///
    /// # Arguments
    ///
    /// * `function_name` - The name of the custom call function
    /// * `api_version` - The API version (untyped or typed)
    /// * `handler_instantiate` - Optional handler for instantiation stage
    /// * `handler_prepare` - Optional handler for preparation stage
    /// * `handler_initialize` - Optional handler for initialization stage
    /// * `handler_execute` - Optional handler for execution stage
    ///
    /// # Safety
    ///
    /// All handler pointers must remain valid for the lifetime of the program.
    pub unsafe fn register_custom_call(
        &self,
        function_name: &str,
        api_version: CustomCallApiVersion,
        handler_instantiate: Option<CustomCallHandler>,
        handler_prepare: Option<CustomCallHandler>,
        handler_initialize: Option<CustomCallHandler>,
        handler_execute: Option<CustomCallHandler>,
    ) -> Result<()> {
        let name = CString::new(function_name)
            .map_err(|_| Error::InvalidArgument("function_name contains null byte".into()))?;

        let mut args = unsafe { std::mem::zeroed::<PJRT_Gpu_Register_Custom_Call_Args>() };
        args.struct_size = std::mem::size_of::<PJRT_Gpu_Register_Custom_Call_Args>();
        args.function_name = name.as_ptr();
        args.function_name_size = name.count_bytes();
        args.api_version = api_version as i32;
        args.handler_instantiate = handler_instantiate.unwrap_or(std::ptr::null_mut());
        args.handler_prepare = handler_prepare.unwrap_or(std::ptr::null_mut());
        args.handler_initialize = handler_initialize.unwrap_or(std::ptr::null_mut());
        args.handler_execute = handler_execute.unwrap_or(std::ptr::null_mut());

        let ext_fn = self
            .raw
            .custom_call
            .ok_or(Error::NullFunctionPointer("PJRT_Gpu_Register_Custom_Call"))?;

        let err = unsafe { ext_fn(&mut args) };
        self.api.err_or(err, ())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extension_type() {
        assert_eq!(GpuExtension::extension_type(), ExtensionType::GpuCustomCall);
    }

    #[test]
    fn test_from_raw_null_returns_none() {
        let api = unsafe { Api::empty_for_testing() };
        let result = unsafe { GpuExtension::from_raw(std::ptr::null_mut(), &api) };
        assert!(result.is_none());
    }

    #[test]
    fn test_from_raw_wrong_type_returns_none() {
        let api = unsafe { Api::empty_for_testing() };
        let mut ext = unsafe { std::mem::zeroed::<PJRT_Gpu_Custom_Call>() };
        ext.base.type_ = ExtensionType::Example.to_raw();
        let result = unsafe {
            GpuExtension::from_raw(
                &mut ext as *mut PJRT_Gpu_Custom_Call as *mut pjrt_sys::PJRT_Extension_Base,
                &api,
            )
        };
        assert!(result.is_none());
    }

    #[test]
    fn test_from_raw_correct_type() {
        let api = unsafe { Api::empty_for_testing() };
        let mut ext = unsafe { std::mem::zeroed::<PJRT_Gpu_Custom_Call>() };
        ext.base.type_ = ExtensionType::GpuCustomCall.to_raw();
        let result = unsafe {
            GpuExtension::from_raw(
                &mut ext as *mut PJRT_Gpu_Custom_Call as *mut pjrt_sys::PJRT_Extension_Base,
                &api,
            )
        };
        assert!(result.is_some());
    }

    #[test]
    fn test_debug_format() {
        let api = unsafe { Api::empty_for_testing() };
        let mut ext = unsafe { std::mem::zeroed::<PJRT_Gpu_Custom_Call>() };
        ext.base.type_ = ExtensionType::GpuCustomCall.to_raw();
        let gpu = unsafe {
            GpuExtension::from_raw(
                &mut ext as *mut PJRT_Gpu_Custom_Call as *mut pjrt_sys::PJRT_Extension_Base,
                &api,
            )
        }
        .unwrap();
        let debug = format!("{:?}", gpu);
        assert!(debug.contains("GpuExtension"));
    }

    #[test]
    fn test_register_custom_call_null_function_pointer() {
        let api = unsafe { Api::empty_for_testing() };
        let mut ext = unsafe { std::mem::zeroed::<PJRT_Gpu_Custom_Call>() };
        ext.base.type_ = ExtensionType::GpuCustomCall.to_raw();
        // custom_call is None (zeroed)
        let gpu = unsafe {
            GpuExtension::from_raw(
                &mut ext as *mut PJRT_Gpu_Custom_Call as *mut pjrt_sys::PJRT_Extension_Base,
                &api,
            )
        }
        .unwrap();
        let result = unsafe {
            gpu.register_custom_call(
                "test_fn",
                CustomCallApiVersion::Typed,
                None,
                None,
                None,
                None,
            )
        };
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(format!("{}", err).contains("PJRT_Gpu_Register_Custom_Call"));
    }

    #[test]
    fn test_custom_call_api_version_values() {
        assert_eq!(CustomCallApiVersion::Untyped as i32, 0);
        assert_eq!(CustomCallApiVersion::Typed as i32, 1);
    }
}
