//! PJRT TPU Executable Extension
//!
//! This module provides safe Rust bindings for the PJRT TPU Executable extension.
//! The TPU Executable extension provides access to TPU-specific executable features.
//!
//! ## Overview
//!
//! This extension is primarily used with TPU devices and provides capabilities for:
//!
//! - Extracting target arguments from serialized TPU executables
//! - Retrieving HLO module with configuration from serialized executables
//! - Querying the core program ABI version of serialized executables
//!
//! ## Usage
//!
//! ```rust,ignore
//! use pjrt::TpuExecutableExtension;
//!
//! // Get the TPU executable extension if available
//! if let Some(ext) = api.get_extension::<TpuExecutableExtension>() {
//!     let data = ext.get_target_arguments(&serialized_executable)?;
//!     println!("Target arguments: {} bytes", data.as_bytes().len());
//! }
//! ```
//!
//! ## Note
//!
//! This extension is only available in TPU PJRT plugins.

use std::rc::Rc;

use pjrt_sys::{
    PJRT_TpuExecutable_CoreProgramAbiVersion, PJRT_TpuExecutable_Extension,
    PJRT_TpuExecutable_GetCoreProgramAbiVersion_Args,
    PJRT_TpuExecutable_GetHloModuleWithConfig_Args, PJRT_TpuExecutable_GetTargetArguments_Args,
    PJRT_TpuExecutable_HloModuleWithConfig, PJRT_TpuExecutable_TargetArguments,
};

use crate::extension::{Extension, ExtensionType};
use crate::{Api, Error, Result};

/// Owned data returned by TPU executable extension methods.
///
/// Each method returns serialized data along with an opaque handle and deleter
/// function. When this struct is dropped, the deleter is called to free the
/// plugin-allocated memory.
pub struct OwnedTargetArguments {
    data: Vec<u8>,
    handle: *mut PJRT_TpuExecutable_TargetArguments,
    deleter: Option<unsafe extern "C" fn(*mut PJRT_TpuExecutable_TargetArguments)>,
}

impl OwnedTargetArguments {
    /// Returns the raw bytes of the target arguments.
    pub fn as_bytes(&self) -> &[u8] {
        &self.data
    }
}

impl Drop for OwnedTargetArguments {
    fn drop(&mut self) {
        if let Some(deleter) = self.deleter {
            if !self.handle.is_null() {
                unsafe { deleter(self.handle) };
            }
        }
    }
}

/// Owned data for core program ABI version.
pub struct OwnedCoreProgramAbiVersion {
    data: Vec<u8>,
    handle: *mut PJRT_TpuExecutable_CoreProgramAbiVersion,
    deleter: Option<unsafe extern "C" fn(*mut PJRT_TpuExecutable_CoreProgramAbiVersion)>,
}

impl OwnedCoreProgramAbiVersion {
    /// Returns the raw bytes of the ABI version.
    pub fn as_bytes(&self) -> &[u8] {
        &self.data
    }
}

impl Drop for OwnedCoreProgramAbiVersion {
    fn drop(&mut self) {
        if let Some(deleter) = self.deleter {
            if !self.handle.is_null() {
                unsafe { deleter(self.handle) };
            }
        }
    }
}

/// Owned data for HLO module with configuration.
pub struct OwnedHloModuleWithConfig {
    data: Vec<u8>,
    handle: *mut PJRT_TpuExecutable_HloModuleWithConfig,
    deleter: Option<unsafe extern "C" fn(*mut PJRT_TpuExecutable_HloModuleWithConfig)>,
}

impl OwnedHloModuleWithConfig {
    /// Returns the raw bytes of the HLO module with config.
    pub fn as_bytes(&self) -> &[u8] {
        &self.data
    }
}

impl Drop for OwnedHloModuleWithConfig {
    fn drop(&mut self) {
        if let Some(deleter) = self.deleter {
            if !self.handle.is_null() {
                unsafe { deleter(self.handle) };
            }
        }
    }
}

/// Safe wrapper for PJRT TPU Executable extension.
///
/// This extension provides access to TPU-specific executable features,
/// including TPU-optimized execution and compilation metadata.
///
/// ## Availability
///
/// This extension is only available in TPU PJRT plugins.
pub struct TpuExecutableExtension {
    raw: Rc<PJRT_TpuExecutable_Extension>,
    api: Api,
}

impl std::fmt::Debug for TpuExecutableExtension {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TpuExecutableExtension")
            .field("type", &"TpuExecutable")
            .finish()
    }
}

unsafe impl Extension for TpuExecutableExtension {
    fn extension_type() -> ExtensionType {
        ExtensionType::TpuExecutable
    }

    unsafe fn from_raw(ptr: *mut pjrt_sys::PJRT_Extension_Base, api: &Api) -> Option<Self>
    where
        Self: Sized,
    {
        if ptr.is_null() {
            return None;
        }

        if (*ptr).type_ != ExtensionType::TpuExecutable.to_raw() {
            return None;
        }

        let ext = ptr as *mut PJRT_TpuExecutable_Extension;
        Some(Self {
            raw: Rc::new(*ext),
            api: api.clone(),
        })
    }
}

impl TpuExecutableExtension {
    /// Returns the raw extension pointer.
    pub fn raw_ptr(&self) -> *mut pjrt_sys::PJRT_Extension_Base {
        &self.raw.base as *const pjrt_sys::PJRT_Extension_Base as *mut pjrt_sys::PJRT_Extension_Base
    }

    /// Extract target arguments from a serialized TPU executable.
    ///
    /// # Arguments
    ///
    /// * `serialized_executable` - The serialized TPU executable bytes
    ///
    /// # Returns
    ///
    /// Owned data containing the serialized target arguments. The data is
    /// freed automatically when dropped.
    pub fn get_target_arguments(
        &self,
        serialized_executable: &[u8],
    ) -> Result<OwnedTargetArguments> {
        let mut args: PJRT_TpuExecutable_GetTargetArguments_Args = unsafe { std::mem::zeroed() };
        args.struct_size = std::mem::size_of::<PJRT_TpuExecutable_GetTargetArguments_Args>();
        args.serialized_executable = serialized_executable.as_ptr() as *const i8;
        args.serialized_executable_size = serialized_executable.len();

        let ext_fn = self
            .raw
            .get_target_arguments
            .ok_or(Error::NullFunctionPointer(
                "PJRT_TpuExecutable_GetTargetArguments",
            ))?;

        let err = unsafe { ext_fn(&mut args) };
        self.api.err_or(err, ())?;

        let data = if args.target_arguments.is_null() || args.target_arguments_size == 0 {
            Vec::new()
        } else {
            unsafe {
                std::slice::from_raw_parts(
                    args.target_arguments as *const u8,
                    args.target_arguments_size,
                )
            }
            .to_vec()
        };

        Ok(OwnedTargetArguments {
            data,
            handle: args.target_arguments_ptr,
            deleter: args.target_arguments_deleter,
        })
    }

    /// Get the core program ABI version from a serialized TPU executable.
    ///
    /// # Arguments
    ///
    /// * `serialized_executable` - The serialized TPU executable bytes
    ///
    /// # Returns
    ///
    /// Owned data containing the ABI version string. The data is freed
    /// automatically when dropped.
    pub fn get_core_program_abi_version(
        &self,
        serialized_executable: &[u8],
    ) -> Result<OwnedCoreProgramAbiVersion> {
        let mut args: PJRT_TpuExecutable_GetCoreProgramAbiVersion_Args =
            unsafe { std::mem::zeroed() };
        args.struct_size = std::mem::size_of::<PJRT_TpuExecutable_GetCoreProgramAbiVersion_Args>();
        args.serialized_executable = serialized_executable.as_ptr() as *const i8;
        args.serialized_executable_size = serialized_executable.len();

        let ext_fn = self
            .raw
            .get_core_program_abi_version
            .ok_or(Error::NullFunctionPointer(
                "PJRT_TpuExecutable_GetCoreProgramAbiVersion",
            ))?;

        let err = unsafe { ext_fn(&mut args) };
        self.api.err_or(err, ())?;

        let data = if args.abi_version.is_null() || args.abi_version_size == 0 {
            Vec::new()
        } else {
            unsafe {
                std::slice::from_raw_parts(args.abi_version as *const u8, args.abi_version_size)
            }
            .to_vec()
        };

        Ok(OwnedCoreProgramAbiVersion {
            data,
            handle: args.abi_version_ptr,
            deleter: args.abi_version_deleter,
        })
    }

    /// Get the HLO module with configuration from a serialized TPU executable.
    ///
    /// # Arguments
    ///
    /// * `serialized_executable` - The serialized TPU executable bytes
    ///
    /// # Returns
    ///
    /// Owned data containing the serialized HLO module with config. The data
    /// is freed automatically when dropped.
    pub fn get_hlo_module_with_config(
        &self,
        serialized_executable: &[u8],
    ) -> Result<OwnedHloModuleWithConfig> {
        let mut args: PJRT_TpuExecutable_GetHloModuleWithConfig_Args =
            unsafe { std::mem::zeroed() };
        args.struct_size = std::mem::size_of::<PJRT_TpuExecutable_GetHloModuleWithConfig_Args>();
        args.serialized_executable = serialized_executable.as_ptr() as *const i8;
        args.serialized_executable_size = serialized_executable.len();

        let ext_fn = self
            .raw
            .get_hlo_module_with_config
            .ok_or(Error::NullFunctionPointer(
                "PJRT_TpuExecutable_GetHloModuleWithConfig",
            ))?;

        let err = unsafe { ext_fn(&mut args) };
        self.api.err_or(err, ())?;

        let data = if args.hlo_module_with_config.is_null() || args.hlo_module_with_config_size == 0
        {
            Vec::new()
        } else {
            unsafe {
                std::slice::from_raw_parts(
                    args.hlo_module_with_config as *const u8,
                    args.hlo_module_with_config_size,
                )
            }
            .to_vec()
        };

        Ok(OwnedHloModuleWithConfig {
            data,
            handle: args.hlo_module_with_config_ptr,
            deleter: args.hlo_module_with_config_deleter,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extension_type() {
        assert_eq!(
            TpuExecutableExtension::extension_type(),
            ExtensionType::TpuExecutable
        );
    }

    #[test]
    fn test_from_raw_null_returns_none() {
        let api = unsafe { Api::empty_for_testing() };
        let result = unsafe { TpuExecutableExtension::from_raw(std::ptr::null_mut(), &api) };
        assert!(result.is_none());
    }

    #[test]
    fn test_from_raw_wrong_type_returns_none() {
        let api = unsafe { Api::empty_for_testing() };
        let mut ext: PJRT_TpuExecutable_Extension = unsafe { std::mem::zeroed() };
        ext.base.struct_size = std::mem::size_of::<PJRT_TpuExecutable_Extension>();
        ext.base.type_ = ExtensionType::Example.to_raw();
        let result = unsafe {
            TpuExecutableExtension::from_raw(
                &mut ext.base as *mut pjrt_sys::PJRT_Extension_Base,
                &api,
            )
        };
        assert!(result.is_none());
    }

    #[test]
    fn test_from_raw_correct_type() {
        let api = unsafe { Api::empty_for_testing() };
        let mut ext: PJRT_TpuExecutable_Extension = unsafe { std::mem::zeroed() };
        ext.base.struct_size = std::mem::size_of::<PJRT_TpuExecutable_Extension>();
        ext.base.type_ = ExtensionType::TpuExecutable.to_raw();
        let result = unsafe {
            TpuExecutableExtension::from_raw(
                &mut ext.base as *mut pjrt_sys::PJRT_Extension_Base,
                &api,
            )
        };
        assert!(result.is_some());
    }

    #[test]
    fn test_null_fn_pointer_get_target_arguments() {
        let api = unsafe { Api::empty_for_testing() };
        let mut ext: PJRT_TpuExecutable_Extension = unsafe { std::mem::zeroed() };
        ext.base.struct_size = std::mem::size_of::<PJRT_TpuExecutable_Extension>();
        ext.base.type_ = ExtensionType::TpuExecutable.to_raw();
        let wrapper = unsafe {
            TpuExecutableExtension::from_raw(
                &mut ext.base as *mut pjrt_sys::PJRT_Extension_Base,
                &api,
            )
        }
        .unwrap();
        let result = wrapper.get_target_arguments(&[]);
        assert!(result.is_err());
    }

    #[test]
    fn test_null_fn_pointer_get_core_program_abi_version() {
        let api = unsafe { Api::empty_for_testing() };
        let mut ext: PJRT_TpuExecutable_Extension = unsafe { std::mem::zeroed() };
        ext.base.struct_size = std::mem::size_of::<PJRT_TpuExecutable_Extension>();
        ext.base.type_ = ExtensionType::TpuExecutable.to_raw();
        let wrapper = unsafe {
            TpuExecutableExtension::from_raw(
                &mut ext.base as *mut pjrt_sys::PJRT_Extension_Base,
                &api,
            )
        }
        .unwrap();
        let result = wrapper.get_core_program_abi_version(&[]);
        assert!(result.is_err());
    }

    #[test]
    fn test_null_fn_pointer_get_hlo_module_with_config() {
        let api = unsafe { Api::empty_for_testing() };
        let mut ext: PJRT_TpuExecutable_Extension = unsafe { std::mem::zeroed() };
        ext.base.struct_size = std::mem::size_of::<PJRT_TpuExecutable_Extension>();
        ext.base.type_ = ExtensionType::TpuExecutable.to_raw();
        let wrapper = unsafe {
            TpuExecutableExtension::from_raw(
                &mut ext.base as *mut pjrt_sys::PJRT_Extension_Base,
                &api,
            )
        }
        .unwrap();
        let result = wrapper.get_hlo_module_with_config(&[]);
        assert!(result.is_err());
    }

    #[test]
    fn test_debug_format() {
        let api = unsafe { Api::empty_for_testing() };
        let mut ext: PJRT_TpuExecutable_Extension = unsafe { std::mem::zeroed() };
        ext.base.struct_size = std::mem::size_of::<PJRT_TpuExecutable_Extension>();
        ext.base.type_ = ExtensionType::TpuExecutable.to_raw();
        let wrapper = unsafe {
            TpuExecutableExtension::from_raw(
                &mut ext.base as *mut pjrt_sys::PJRT_Extension_Base,
                &api,
            )
        }
        .unwrap();
        let debug = format!("{:?}", wrapper);
        assert!(debug.contains("TpuExecutableExtension"));
        assert!(debug.contains("TpuExecutable"));
    }
}
