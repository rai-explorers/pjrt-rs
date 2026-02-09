//! PJRT Cross-Host Transfers Extension
//!
//! This module provides safe Rust bindings for the PJRT Cross-Host Transfers extension.
//! The Cross-Host Transfers extension enables efficient data transfer between hosts
//! in distributed training scenarios.
//!
//! ## Overview
//!
//! This extension is primarily used in multi-host distributed setups where data needs
//! to be transferred between different machines. It provides mechanisms for:
//!
//! - Efficient inter-host communication
//! - Coordinated data transfers in distributed training
//! - Low-latency host-to-host buffer transfers
//!
//! ## Usage
//!
//! ```rust,ignore
//! use pjrt::CrossHostTransfersExtension;
//!
//! // Get the cross-host transfers extension if available
//! if let Some(ext) = api.get_extension::<CrossHostTransfersExtension>() {
//!     println!("Cross-host transfers extension is available");
//! }
//! ```
//!
//! ## Note
//!
//! This extension is not implemented in all PJRT plugins. It is primarily
//! available in plugins that support multi-host distributed execution.

use crate::extension::{Extension, ExtensionType};
use crate::Api;

/// Safe wrapper for PJRT Cross-Host Transfers extension.
///
/// This extension provides capabilities for transferring data between hosts
/// in distributed training scenarios.
///
/// ## Availability
///
/// This extension is typically only available in PJRT plugins that support
/// multi-host distributed execution, such as TPU plugins.
pub struct CrossHostTransfersExtension {
    raw: *mut pjrt_sys::PJRT_Extension_Base,
    _api: Api,
}

impl std::fmt::Debug for CrossHostTransfersExtension {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CrossHostTransfersExtension")
            .field("type", &"CrossHostTransfers")
            .finish()
    }
}

unsafe impl Extension for CrossHostTransfersExtension {
    fn extension_type() -> ExtensionType {
        ExtensionType::CrossHostTransfers
    }

    unsafe fn from_raw(ptr: *mut pjrt_sys::PJRT_Extension_Base, api: &Api) -> Option<Self>
    where
        Self: Sized,
    {
        if ptr.is_null() {
            return None;
        }

        if (*ptr).type_ != ExtensionType::CrossHostTransfers.to_raw() {
            return None;
        }

        Some(Self {
            raw: ptr,
            _api: api.clone(),
        })
    }
}

impl CrossHostTransfersExtension {
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
            CrossHostTransfersExtension::extension_type(),
            ExtensionType::CrossHostTransfers
        );
    }

    #[test]
    fn test_from_raw_null_returns_none() {
        let api = unsafe { Api::empty_for_testing() };
        let result = unsafe { CrossHostTransfersExtension::from_raw(std::ptr::null_mut(), &api) };
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
            CrossHostTransfersExtension::from_raw(
                &mut base as *mut pjrt_sys::PJRT_Extension_Base,
                &api,
            )
        };
        assert!(result.is_none());
    }

    #[test]
    fn test_from_raw_correct_type() {
        let api = unsafe { Api::empty_for_testing() };
        let mut base = pjrt_sys::PJRT_Extension_Base {
            struct_size: std::mem::size_of::<pjrt_sys::PJRT_Extension_Base>(),
            type_: ExtensionType::CrossHostTransfers.to_raw(),
            next: std::ptr::null_mut(),
        };
        let result = unsafe {
            CrossHostTransfersExtension::from_raw(
                &mut base as *mut pjrt_sys::PJRT_Extension_Base,
                &api,
            )
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
            type_: ExtensionType::CrossHostTransfers.to_raw(),
            next: std::ptr::null_mut(),
        };
        let ext = unsafe {
            CrossHostTransfersExtension::from_raw(
                &mut base as *mut pjrt_sys::PJRT_Extension_Base,
                &api,
            )
        }
        .unwrap();
        let debug = format!("{:?}", ext);
        assert!(debug.contains("CrossHostTransfersExtension"));
        assert!(debug.contains("CrossHostTransfers"));
    }
}
