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
use crate::{Api, ExecuteContext, Result};

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
    bits: u32,
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
        let name = CString::new(type_name).expect("type_name contains null bytes");

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
            .expect("PJRT_FFI_Type_Register not implemented");

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
        let target = CString::new(target_name).expect("target_name contains null bytes");
        let platform = CString::new(platform_name).expect("platform_name contains null bytes");

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
            .expect("PJRT_FFI_Register_Handler not implemented");

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
            .expect("PJRT_FFI_UserData_Add not implemented");

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
        // Access the API's extension chain
        // This is a placeholder - in a real implementation we'd traverse the extension chain
        None
    }
}
