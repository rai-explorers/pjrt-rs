//! PJRT FFI Extension
//!
//! This module provides safe Rust bindings for the PJRT FFI (Foreign Function Interface)
//! extension. The FFI extension provides capabilities for integrating with backend-specific
//! FFI libraries, allowing custom operations to be registered and called.
//!
//! ## Usage
//!
//! ```rust,ignore
//! use pjrt::ffi::{FfiExtension, FfiHandler, FfiHandlerTraits};
//!
//! // Get the FFI extension
//! let ffi_ext = api.get_extension::<FfiExtension>()?;
//!
//! // Register a type
//! let type_id = ffi_ext.register_type("MyType", type_info)?;
//!
//! // Register an FFI handler
//! ffi_ext.register_handler(
//!     "my_custom_op",
//!     "CUDA",
//!     handler_ptr,
//!     FfiHandlerTraits::empty()
//! )?;
//!
//! // Add user data to execution context
//! ffi_ext.add_user_data(&execute_context, type_id, data_ptr)?;
//! ```

use std::ffi::CString;
use std::marker::PhantomData;
use std::rc::Rc;

use pjrt_sys::{
    PJRT_FFI_Extension, PJRT_FFI_Handler_TraitsBits,
    PJRT_FFI_Handler_TraitsBits_PJRT_FFI_HANDLER_TRAITS_COMMAND_BUFFER_COMPATIBLE,
    PJRT_FFI_Register_Handler_Args, PJRT_FFI_Type_Info, PJRT_FFI_Type_Register_Args,
    PJRT_FFI_UserData, PJRT_FFI_UserData_Add_Args,
};

use crate::extension::{Extension, ExtensionType};
use crate::{Api, Error, ExecuteContext, Result};

/// Safe wrapper for PJRT FFI extension
///
/// The FFI extension provides capabilities for integrating with backend-specific
/// FFI libraries. This allows registration of custom types and handlers that can
/// be called from XLA computations.
pub struct FfiExtension {
    raw: Rc<PJRT_FFI_Extension>,
    api: Api,
}

impl std::fmt::Debug for FfiExtension {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FfiExtension")
            .field("api_version", &3i32) // Version 3
            .finish()
    }
}

unsafe impl Extension for FfiExtension {
    fn extension_type() -> ExtensionType {
        ExtensionType::Ffi
    }

    unsafe fn from_raw(ptr: *mut pjrt_sys::PJRT_Extension_Base, api: &Api) -> Option<Self>
    where
        Self: Sized,
    {
        if ptr.is_null() {
            return None;
        }

        let ffi_ext = ptr as *mut PJRT_FFI_Extension;
        if (*ffi_ext).base.type_ != ExtensionType::Ffi.to_raw() {
            return None;
        }

        Some(Self {
            raw: Rc::new(*ffi_ext),
            api: api.clone(),
        })
    }
}

/// Traits for FFI handlers
#[derive(Debug, Clone, Copy, Default)]
pub struct FfiHandlerTraits {
    bits: PJRT_FFI_Handler_TraitsBits,
}

impl FfiHandlerTraits {
    /// Create empty traits (no special flags)
    pub const fn empty() -> Self {
        Self { bits: 0 }
    }

    /// Check if the command buffer compatible flag is set
    pub fn is_command_buffer_compatible(&self) -> bool {
        self.bits & PJRT_FFI_Handler_TraitsBits_PJRT_FFI_HANDLER_TRAITS_COMMAND_BUFFER_COMPATIBLE
            != 0
    }

    /// Set the command buffer compatible flag
    pub fn set_command_buffer_compatible(mut self, value: bool) -> Self {
        if value {
            self.bits |=
                PJRT_FFI_Handler_TraitsBits_PJRT_FFI_HANDLER_TRAITS_COMMAND_BUFFER_COMPATIBLE;
        } else {
            self.bits &=
                !PJRT_FFI_Handler_TraitsBits_PJRT_FFI_HANDLER_TRAITS_COMMAND_BUFFER_COMPATIBLE;
        }
        self
    }

    fn to_raw(self) -> PJRT_FFI_Handler_TraitsBits {
        self.bits
    }
}

/// Opaque FFI handler type
pub type FfiHandler = *mut std::ffi::c_void;

/// Type information for FFI registered types
pub struct FfiTypeInfo {
    /// Function to delete objects of this type
    pub deleter: Option<unsafe extern "C" fn(*mut std::ffi::c_void)>,
    /// Placeholder for future serialization support
    pub _serialize: PhantomData<()>,
    /// Placeholder for future deserialization support  
    pub _deserialize: PhantomData<()>,
}

impl FfiExtension {
    /// Register an external type in the static type registry
    ///
    /// If `type_id` is 0, XLA will assign a unique type id and return it.
    /// Otherwise, it will verify that the provided type id matches previously
    /// registered type id for the given type name.
    ///
    /// # Arguments
    ///
    /// * `type_name` - The name of the type to register
    /// * `type_info` - Information about the type including deleter function
    /// * `type_id` - Input/output type ID (0 to get auto-assigned ID)
    ///
    /// # Returns
    ///
    /// The registered type ID (auto-assigned if input was 0)
    pub fn register_type(
        &self,
        type_name: &str,
        type_info: &FfiTypeInfo,
        type_id: i64,
    ) -> Result<i64> {
        let name = CString::new(type_name)
            .map_err(|_| Error::InvalidArgument("type_name contains null byte".into()))?;

        let raw_type_info = PJRT_FFI_Type_Info {
            deleter: type_info.deleter,
            serialize: None,
            deserialize: None,
        };

        let mut args = unsafe { std::mem::zeroed::<PJRT_FFI_Type_Register_Args>() };
        args.struct_size = std::mem::size_of::<PJRT_FFI_Type_Register_Args>();
        args.type_name = name.as_ptr();
        args.type_name_size = name.count_bytes();
        args.type_id = type_id;
        args.type_info = &raw_type_info as *const _ as *mut _;

        let ext_fn = self
            .raw
            .type_register
            .ok_or(Error::NullFunctionPointer("PJRT_FFI_Type_Register"))?;

        let err = unsafe { ext_fn(&mut args) };
        self.api.err_or(err, ())?;

        let new_type_id = args.type_id;
        Ok(new_type_id)
    }

    /// Register an FFI call handler for a specific platform
    ///
    /// # Arguments
    ///
    /// * `target_name` - The name of the custom call target
    /// * `platform_name` - The platform name (e.g., "CUDA", "Host", "ROCM")
    /// * `handler` - Pointer to the handler function (XLA_FFI_Handler*)
    /// * `traits` - Handler traits specifying compatibility flags
    ///
    /// # Safety
    ///
    /// The handler pointer must be a valid XLA_FFI_Handler function pointer
    /// that remains valid for the lifetime of the program.
    pub unsafe fn register_handler(
        &self,
        target_name: &str,
        platform_name: &str,
        handler: FfiHandler,
        traits: FfiHandlerTraits,
    ) -> Result<()> {
        let target = CString::new(target_name)
            .map_err(|_| Error::InvalidArgument("target_name contains null byte".into()))?;
        let platform = CString::new(platform_name)
            .map_err(|_| Error::InvalidArgument("platform_name contains null byte".into()))?;

        let mut args = unsafe { std::mem::zeroed::<PJRT_FFI_Register_Handler_Args>() };
        args.struct_size = std::mem::size_of::<PJRT_FFI_Register_Handler_Args>();
        args.target_name = target.as_ptr();
        args.target_name_size = target.count_bytes();
        args.handler = handler;
        args.platform_name = platform.as_ptr();
        args.platform_name_size = platform.count_bytes();
        args.traits = traits.to_raw();

        let ext_fn = self
            .raw
            .register_handler
            .ok_or(Error::NullFunctionPointer("PJRT_FFI_Register_Handler"))?;

        let err = unsafe { ext_fn(&mut args) };
        self.api.err_or(err, ())
    }

    /// Add user data to an execution context
    ///
    /// User data is forwarded to FFI handlers during execution.
    ///
    /// # Arguments
    ///
    /// * `context` - The execution context to add data to
    /// * `type_id` - The registered type ID of the data
    /// * `data` - Pointer to the user data
    ///
    /// # Safety
    ///
    /// The data pointer must remain valid until the execution context is destroyed.
    pub unsafe fn add_user_data(
        &self,
        context: &ExecuteContext,
        type_id: i64,
        data: *mut std::ffi::c_void,
    ) -> Result<()> {
        let mut args = unsafe { std::mem::zeroed::<PJRT_FFI_UserData_Add_Args>() };
        args.struct_size = std::mem::size_of::<PJRT_FFI_UserData_Add_Args>();
        args.context = context.ptr;
        args.user_data = PJRT_FFI_UserData { type_id, data };

        let ext_fn = self
            .raw
            .user_data_add
            .ok_or(Error::NullFunctionPointer("PJRT_FFI_UserData_Add"))?;

        let err = unsafe { ext_fn(&mut args) };
        self.api.err_or(err, ())
    }
}

/// Extension trait for accessing FFI extension from Api
#[allow(dead_code)]
pub trait FfiExt {
    /// Get the FFI extension if available
    fn ffi_extension(&self) -> Option<FfiExtension>;
}

impl FfiExt for Api {
    fn ffi_extension(&self) -> Option<FfiExtension> {
        let ext_start = self.extension_start();
        unsafe { FfiExtension::from_raw(ext_start, self) }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extension_type() {
        assert_eq!(FfiExtension::extension_type(), ExtensionType::Ffi);
    }

    #[test]
    fn test_from_raw_null_returns_none() {
        let api = unsafe { Api::empty_for_testing() };
        let result = unsafe { FfiExtension::from_raw(std::ptr::null_mut(), &api) };
        assert!(result.is_none());
    }

    #[test]
    fn test_from_raw_wrong_type_returns_none() {
        let api = unsafe { Api::empty_for_testing() };
        let mut ext = unsafe { std::mem::zeroed::<PJRT_FFI_Extension>() };
        ext.base.type_ = ExtensionType::Example.to_raw();
        let result = unsafe {
            FfiExtension::from_raw(
                &mut ext as *mut PJRT_FFI_Extension as *mut pjrt_sys::PJRT_Extension_Base,
                &api,
            )
        };
        assert!(result.is_none());
    }

    #[test]
    fn test_from_raw_correct_type() {
        let api = unsafe { Api::empty_for_testing() };
        let mut ext = unsafe { std::mem::zeroed::<PJRT_FFI_Extension>() };
        ext.base.type_ = ExtensionType::Ffi.to_raw();
        let result = unsafe {
            FfiExtension::from_raw(
                &mut ext as *mut PJRT_FFI_Extension as *mut pjrt_sys::PJRT_Extension_Base,
                &api,
            )
        };
        assert!(result.is_some());
    }

    #[test]
    fn test_debug_format() {
        let api = unsafe { Api::empty_for_testing() };
        let mut ext = unsafe { std::mem::zeroed::<PJRT_FFI_Extension>() };
        ext.base.type_ = ExtensionType::Ffi.to_raw();
        let ffi = unsafe {
            FfiExtension::from_raw(
                &mut ext as *mut PJRT_FFI_Extension as *mut pjrt_sys::PJRT_Extension_Base,
                &api,
            )
        }
        .unwrap();
        let debug = format!("{:?}", ffi);
        assert!(debug.contains("FfiExtension"));
        assert!(debug.contains("api_version"));
    }

    #[test]
    fn test_register_type_null_function_pointer() {
        let api = unsafe { Api::empty_for_testing() };
        let mut ext = unsafe { std::mem::zeroed::<PJRT_FFI_Extension>() };
        ext.base.type_ = ExtensionType::Ffi.to_raw();
        let ffi = unsafe {
            FfiExtension::from_raw(
                &mut ext as *mut PJRT_FFI_Extension as *mut pjrt_sys::PJRT_Extension_Base,
                &api,
            )
        }
        .unwrap();
        let type_info = FfiTypeInfo {
            deleter: None,
            _serialize: PhantomData,
            _deserialize: PhantomData,
        };
        let result = ffi.register_type("test_type", &type_info, 0);
        assert!(result.is_err());
        assert!(format!("{}", result.unwrap_err()).contains("PJRT_FFI_Type_Register"));
    }

    #[test]
    fn test_register_handler_null_function_pointer() {
        let api = unsafe { Api::empty_for_testing() };
        let mut ext = unsafe { std::mem::zeroed::<PJRT_FFI_Extension>() };
        ext.base.type_ = ExtensionType::Ffi.to_raw();
        let ffi = unsafe {
            FfiExtension::from_raw(
                &mut ext as *mut PJRT_FFI_Extension as *mut pjrt_sys::PJRT_Extension_Base,
                &api,
            )
        }
        .unwrap();
        let result = unsafe {
            ffi.register_handler(
                "target",
                "CUDA",
                std::ptr::null_mut(),
                FfiHandlerTraits::empty(),
            )
        };
        assert!(result.is_err());
        assert!(format!("{}", result.unwrap_err()).contains("PJRT_FFI_Register_Handler"));
    }

    #[test]
    fn test_register_type_null_byte_in_name() {
        let api = unsafe { Api::empty_for_testing() };
        let mut ext = unsafe { std::mem::zeroed::<PJRT_FFI_Extension>() };
        ext.base.type_ = ExtensionType::Ffi.to_raw();
        let ffi = unsafe {
            FfiExtension::from_raw(
                &mut ext as *mut PJRT_FFI_Extension as *mut pjrt_sys::PJRT_Extension_Base,
                &api,
            )
        }
        .unwrap();
        let type_info = FfiTypeInfo {
            deleter: None,
            _serialize: PhantomData,
            _deserialize: PhantomData,
        };
        let result = ffi.register_type("test\0type", &type_info, 0);
        assert!(result.is_err());
        assert!(format!("{}", result.unwrap_err()).contains("null byte"));
    }

    #[test]
    fn test_ffi_handler_traits_empty() {
        let traits = FfiHandlerTraits::empty();
        assert!(!traits.is_command_buffer_compatible());
    }

    #[test]
    fn test_ffi_handler_traits_command_buffer_compatible() {
        let traits = FfiHandlerTraits::empty().set_command_buffer_compatible(true);
        assert!(traits.is_command_buffer_compatible());

        let traits = traits.set_command_buffer_compatible(false);
        assert!(!traits.is_command_buffer_compatible());
    }

    #[test]
    fn test_ffi_ext_trait_returns_none_for_empty_api() {
        let api = unsafe { Api::empty_for_testing() };
        let result = api.ffi_extension();
        assert!(result.is_none());
    }
}
