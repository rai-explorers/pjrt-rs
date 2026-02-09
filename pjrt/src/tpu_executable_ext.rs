//! PJRT TPU Executable Extension
//!
//! This module provides safe Rust bindings for the PJRT TPU Executable extension.
//! The TPU Executable extension provides access to TPU-specific executable features.
//!
//! ## Overview
//!
//! This extension is primarily used with TPU devices and provides capabilities for:
//!
//! - Accessing TPU-specific executable properties
//! - TPU-optimized execution features
//! - TPU compilation metadata
//!
//! ## Usage
//!
//! ```rust,ignore
//! use pjrt::TpuExecutableExtension;
//!
//! // Get the TPU executable extension if available
//! if let Some(ext) = api.get_extension::<TpuExecutableExtension>() {
//!     println!("TPU executable extension is available");
//! }
//! ```
//!
//! ## Note
//!
//! This extension is only available in TPU PJRT plugins.

use crate::extension::{Extension, ExtensionType};
use crate::Api;

/// Safe wrapper for PJRT TPU Executable extension.
///
/// This extension provides access to TPU-specific executable features,
/// including TPU-optimized execution and compilation metadata.
///
/// ## Availability
///
/// This extension is only available in TPU PJRT plugins.
pub struct TpuExecutableExtension {
    raw: *mut pjrt_sys::PJRT_Extension_Base,
    _api: Api,
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

        Some(Self {
            raw: ptr,
            _api: api.clone(),
        })
    }
}

impl TpuExecutableExtension {
    /// Returns the raw extension pointer.
    ///
    /// # Safety
    ///
    /// The returned pointer is valid only for the lifetime of this extension.
    pub fn raw_ptr(&self) -> *mut pjrt_sys::PJRT_Extension_Base {
        self.raw
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
        let mut base = pjrt_sys::PJRT_Extension_Base {
            struct_size: std::mem::size_of::<pjrt_sys::PJRT_Extension_Base>(),
            type_: ExtensionType::Example.to_raw(),
            next: std::ptr::null_mut(),
        };
        let result = unsafe {
            TpuExecutableExtension::from_raw(&mut base as *mut pjrt_sys::PJRT_Extension_Base, &api)
        };
        assert!(result.is_none());
    }

    #[test]
    fn test_from_raw_correct_type() {
        let api = unsafe { Api::empty_for_testing() };
        let mut base = pjrt_sys::PJRT_Extension_Base {
            struct_size: std::mem::size_of::<pjrt_sys::PJRT_Extension_Base>(),
            type_: ExtensionType::TpuExecutable.to_raw(),
            next: std::ptr::null_mut(),
        };
        let result = unsafe {
            TpuExecutableExtension::from_raw(&mut base as *mut pjrt_sys::PJRT_Extension_Base, &api)
        };
        assert!(result.is_some());
        let ext = result.unwrap();
        assert_eq!(
            ext.raw_ptr(),
            &mut base as *mut pjrt_sys::PJRT_Extension_Base
        );
    }

    #[test]
    fn test_debug_format() {
        let api = unsafe { Api::empty_for_testing() };
        let mut base = pjrt_sys::PJRT_Extension_Base {
            struct_size: std::mem::size_of::<pjrt_sys::PJRT_Extension_Base>(),
            type_: ExtensionType::TpuExecutable.to_raw(),
            next: std::ptr::null_mut(),
        };
        let ext = unsafe {
            TpuExecutableExtension::from_raw(&mut base as *mut pjrt_sys::PJRT_Extension_Base, &api)
        }
        .unwrap();
        let debug = format!("{:?}", ext);
        assert!(debug.contains("TpuExecutableExtension"));
        assert!(debug.contains("TpuExecutable"));
    }
}
