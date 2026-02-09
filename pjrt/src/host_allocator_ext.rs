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

use std::ffi::c_void;
use std::rc::Rc;

use pjrt_sys::{
    PJRT_HostAllocator_Allocate_Args, PJRT_HostAllocator_Extension, PJRT_HostAllocator_Free_Args,
    PJRT_HostAllocator_GetPreferredAlignment_Args,
};

use crate::extension::{Extension, ExtensionType};
use crate::{Api, Client, Error, Result};

/// Safe wrapper for PJRT Host Allocator extension (Experimental).
///
/// This extension provides capabilities for customizing host memory allocation
/// strategies used by PJRT, including:
///
/// - Querying preferred alignment for host allocations
/// - Allocating aligned host memory through the PJRT plugin
/// - Freeing plugin-allocated host memory
///
/// ## Warning
///
/// This extension is marked as **experimental** in the PJRT API and may change
/// or be removed in future versions without notice.
///
/// ## Availability
///
/// This extension may not be available in all PJRT plugins.
pub struct HostAllocatorExtension {
    raw: Rc<PJRT_HostAllocator_Extension>,
    api: Api,
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

        let ext = ptr as *mut PJRT_HostAllocator_Extension;
        Some(Self {
            raw: Rc::new(*ext),
            api: api.clone(),
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
        &self.raw.base as *const pjrt_sys::PJRT_Extension_Base as *mut pjrt_sys::PJRT_Extension_Base
    }

    /// Indicates that this extension is experimental.
    pub fn is_experimental(&self) -> bool {
        true
    }

    /// Get the preferred alignment for host memory allocations.
    ///
    /// Plugins may require specific alignment for host buffers to enable
    /// efficient DMA transfers or other optimized operations.
    ///
    /// # Arguments
    ///
    /// * `client` - The PJRT client to query
    ///
    /// # Returns
    ///
    /// The preferred alignment in bytes
    pub fn get_preferred_alignment(&self, client: &Client) -> Result<usize> {
        let mut args: PJRT_HostAllocator_GetPreferredAlignment_Args = unsafe { std::mem::zeroed() };
        args.struct_size = std::mem::size_of::<PJRT_HostAllocator_GetPreferredAlignment_Args>();
        args.extension_start = self.raw_ptr();
        args.client = client.ptr();

        let ext_fn = self
            .raw
            .get_preferred_alignment
            .ok_or(Error::NullFunctionPointer(
                "PJRT_HostAllocator_GetPreferredAlignment",
            ))?;

        let err = unsafe { ext_fn(&mut args) };
        self.api.err_or(err, ())?;

        Ok(args.preferred_alignment)
    }

    /// Allocate host memory through the PJRT plugin.
    ///
    /// This allocates memory with the specified size and alignment using
    /// the plugin's preferred allocation strategy.
    ///
    /// # Arguments
    ///
    /// * `client` - The PJRT client
    /// * `size` - Number of bytes to allocate
    /// * `alignment` - Required alignment in bytes (use `get_preferred_alignment` to query)
    ///
    /// # Returns
    ///
    /// A raw pointer to the allocated memory
    ///
    /// # Safety
    ///
    /// The caller must ensure the returned memory is freed with [`free`](Self::free).
    pub fn allocate(&self, client: &Client, size: usize, alignment: usize) -> Result<*mut c_void> {
        let mut args: PJRT_HostAllocator_Allocate_Args = unsafe { std::mem::zeroed() };
        args.struct_size = std::mem::size_of::<PJRT_HostAllocator_Allocate_Args>();
        args.extension_start = self.raw_ptr();
        args.client = client.ptr();
        args.size = size;
        args.alignment = alignment;

        let ext_fn = self
            .raw
            .allocate
            .ok_or(Error::NullFunctionPointer("PJRT_HostAllocator_Allocate"))?;

        let err = unsafe { ext_fn(&mut args) };
        self.api.err_or(err, ())?;

        Ok(args.ptr)
    }

    /// Free host memory previously allocated by [`allocate`](Self::allocate).
    ///
    /// # Arguments
    ///
    /// * `client` - The PJRT client
    /// * `ptr` - The pointer returned by a previous `allocate` call
    ///
    /// # Safety
    ///
    /// The caller must ensure `ptr` was allocated by this extension's `allocate` method.
    pub fn free(&self, client: &Client, ptr: *mut c_void) -> Result<()> {
        let mut args: PJRT_HostAllocator_Free_Args = unsafe { std::mem::zeroed() };
        args.struct_size = std::mem::size_of::<PJRT_HostAllocator_Free_Args>();
        args.extension_start = self.raw_ptr();
        args.client = client.ptr();
        args.ptr = ptr;

        let ext_fn = self
            .raw
            .free
            .ok_or(Error::NullFunctionPointer("PJRT_HostAllocator_Free"))?;

        let err = unsafe { ext_fn(&mut args) };
        self.api.err_or(err, ())?;

        Ok(())
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
    fn test_is_experimental() {
        let api = unsafe { Api::empty_for_testing() };
        let mut ext: PJRT_HostAllocator_Extension = unsafe { std::mem::zeroed() };
        ext.base.struct_size = std::mem::size_of::<PJRT_HostAllocator_Extension>();
        ext.base.type_ = ExtensionType::HostAllocator.to_raw();
        let wrapper = unsafe {
            HostAllocatorExtension::from_raw(
                &mut ext.base as *mut pjrt_sys::PJRT_Extension_Base,
                &api,
            )
        }
        .unwrap();
        assert!(wrapper.is_experimental());
    }

    #[test]
    fn test_debug_format() {
        let api = unsafe { Api::empty_for_testing() };
        let mut ext: PJRT_HostAllocator_Extension = unsafe { std::mem::zeroed() };
        ext.base.struct_size = std::mem::size_of::<PJRT_HostAllocator_Extension>();
        ext.base.type_ = ExtensionType::HostAllocator.to_raw();
        let wrapper = unsafe {
            HostAllocatorExtension::from_raw(
                &mut ext.base as *mut pjrt_sys::PJRT_Extension_Base,
                &api,
            )
        }
        .unwrap();
        let debug = format!("{:?}", wrapper);
        assert!(debug.contains("HostAllocatorExtension"));
        assert!(debug.contains("HostAllocator"));
        assert!(debug.contains("experimental"));
    }
}
