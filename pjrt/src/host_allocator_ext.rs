//! PJRT Host Allocator Extension (Experimental)
//!
//! This module provides safe Rust bindings for the PJRT Host Allocator extension.
//! The Host Allocator extension allows customization of host memory allocation
//! strategies used by PJRT.
//!
//! ## Overview
//!
//! This **experimental** extension provides capabilities for:
//!
//! - Customizing host memory allocation strategies
//! - Providing custom allocators for host buffers
//! - Memory pool management
//!
//! ## Usage
//!
//! ```rust,ignore
//! use pjrt::HostAllocatorExtension;
//!
//! // Get the host allocator extension if available
//! if let Some(ext) = api.get_extension::<HostAllocatorExtension>() {
//!     println!("Host allocator extension is available");
//! }
//! ```
//!
//! ## Warning
//!
//! This extension is marked as **experimental** in the PJRT API and may change
//! or be removed in future versions without notice.

use crate::extension::{Extension, ExtensionType};
use crate::Api;

/// Safe wrapper for PJRT Host Allocator extension (Experimental).
///
/// This extension provides capabilities for customizing host memory allocation
/// strategies used by PJRT.
///
/// ## Warning
///
/// This extension is **experimental** and may change or be removed in future
/// PJRT versions.
///
/// ## Availability
///
/// This extension may not be available in all PJRT plugins.
pub struct HostAllocatorExtension {
    raw: *mut pjrt_sys::PJRT_Extension_Base,
    _api: Api,
}

impl std::fmt::Debug for HostAllocatorExtension {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("HostAllocatorExtension")
            .field("type", &"HostAllocator")
            .field("experimental", &true)
            .finish()
    }
}

unsafe impl Extension for HostAllocatorExtension {
    fn extension_type() -> ExtensionType {
        ExtensionType::HostAllocator
    }

    unsafe fn from_raw(ptr: *mut pjrt_sys::PJRT_Extension_Base, api: &Api) -> Option<Self>
    where
        Self: Sized,
    {
        if ptr.is_null() {
            return None;
        }

        if (*ptr).type_ != ExtensionType::HostAllocator.to_raw() {
            return None;
        }

        Some(Self {
            raw: ptr,
            _api: api.clone(),
        })
    }
}

impl HostAllocatorExtension {
    /// Returns the raw extension pointer.
    ///
    /// # Safety
    ///
    /// The returned pointer is valid only for the lifetime of this extension.
    pub fn raw_ptr(&self) -> *mut pjrt_sys::PJRT_Extension_Base {
        self.raw
    }

    /// Indicates that this extension is experimental.
    pub fn is_experimental(&self) -> bool {
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extension_type() {
        assert_eq!(
            HostAllocatorExtension::extension_type(),
            ExtensionType::HostAllocator
        );
    }

    #[test]
    fn test_from_raw_null_returns_none() {
        let api = unsafe { Api::empty_for_testing() };
        let result = unsafe { HostAllocatorExtension::from_raw(std::ptr::null_mut(), &api) };
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
            HostAllocatorExtension::from_raw(&mut base as *mut pjrt_sys::PJRT_Extension_Base, &api)
        };
        assert!(result.is_none());
    }

    #[test]
    fn test_from_raw_correct_type() {
        let api = unsafe { Api::empty_for_testing() };
        let mut base = pjrt_sys::PJRT_Extension_Base {
            struct_size: std::mem::size_of::<pjrt_sys::PJRT_Extension_Base>(),
            type_: ExtensionType::HostAllocator.to_raw(),
            next: std::ptr::null_mut(),
        };
        let result = unsafe {
            HostAllocatorExtension::from_raw(&mut base as *mut pjrt_sys::PJRT_Extension_Base, &api)
        };
        assert!(result.is_some());
        let ext = result.unwrap();
        assert_eq!(
            ext.raw_ptr(),
            &mut base as *mut pjrt_sys::PJRT_Extension_Base
        );
    }

    #[test]
    fn test_is_experimental() {
        let api = unsafe { Api::empty_for_testing() };
        let mut base = pjrt_sys::PJRT_Extension_Base {
            struct_size: std::mem::size_of::<pjrt_sys::PJRT_Extension_Base>(),
            type_: ExtensionType::HostAllocator.to_raw(),
            next: std::ptr::null_mut(),
        };
        let ext = unsafe {
            HostAllocatorExtension::from_raw(&mut base as *mut pjrt_sys::PJRT_Extension_Base, &api)
        }
        .unwrap();
        assert!(ext.is_experimental());
    }

    #[test]
    fn test_debug_format() {
        let api = unsafe { Api::empty_for_testing() };
        let mut base = pjrt_sys::PJRT_Extension_Base {
            struct_size: std::mem::size_of::<pjrt_sys::PJRT_Extension_Base>(),
            type_: ExtensionType::HostAllocator.to_raw(),
            next: std::ptr::null_mut(),
        };
        let ext = unsafe {
            HostAllocatorExtension::from_raw(&mut base as *mut pjrt_sys::PJRT_Extension_Base, &api)
        }
        .unwrap();
        let debug = format!("{:?}", ext);
        assert!(debug.contains("HostAllocatorExtension"));
        assert!(debug.contains("HostAllocator"));
        assert!(debug.contains("experimental"));
    }
}
