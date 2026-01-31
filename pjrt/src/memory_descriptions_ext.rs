//! PJRT Memory Descriptions Extension
//!
//! This module provides safe Rust bindings for the PJRT Memory Descriptions extension.
//! This optional extension allows retrieving all supported types of memory for a given
//! device description, which is useful for specifying non-default memories in AOT computations.
//!
//! ## Usage
//!
//! ```rust,ignore
//! use pjrt::memory_descriptions::MemoryDescriptionsExtension;
//!
//! // Get the memory descriptions extension
//! let mem_ext = device_description.extension::<MemoryDescriptionsExtension>()?;
//!
//! // Get memory descriptions for the device
//! let descriptions = mem_ext.get_memory_descriptions(&device_description)?;
//!
//! for desc in descriptions {
//!     println!("Memory kind: {}", desc.kind()?);
//! }
//! ```

use std::rc::Rc;

use pjrt_sys::{
    PJRT_DeviceDescription, PJRT_DeviceDescription_MemoryDescriptions_Args, PJRT_MemoryDescription,
    PJRT_MemoryDescription_Kind_Args, PJRT_MemoryDescriptions_Extension,
};

use crate::extension::{Extension, ExtensionType};
use crate::{Api, DeviceDescription, Result};

/// Safe wrapper for PJRT Memory Descriptions extension
///
/// This extension allows retrieving all supported memory types for a device description,
/// which is useful for AOT computations with non-default memories.
pub struct MemoryDescriptionsExtension {
    raw: Rc<PJRT_MemoryDescriptions_Extension>,
    api: Api,
}

impl std::fmt::Debug for MemoryDescriptionsExtension {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MemoryDescriptionsExtension")
            .field("api_version", &1i32)
            .finish()
    }
}

unsafe impl Extension for MemoryDescriptionsExtension {
    fn extension_type() -> ExtensionType {
        ExtensionType::MemoryDescriptions
    }

    unsafe fn from_raw(ptr: *mut pjrt_sys::PJRT_Extension_Base, api: &Api) -> Option<Self>
    where
        Self: Sized,
    {
        if ptr.is_null() {
            return None;
        }

        let ext = ptr as *mut PJRT_MemoryDescriptions_Extension;
        if (*ext).base.type_ != ExtensionType::MemoryDescriptions.to_raw() {
            return None;
        }

        Some(Self {
            raw: Rc::new((*ext).clone()),
            api: api.clone(),
        })
    }
}

/// Represents a memory description
pub struct MemoryDescription {
    raw: *const PJRT_MemoryDescription,
    ext: MemoryDescriptionsExtension,
}

impl std::fmt::Debug for MemoryDescription {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MemoryDescription")
            .field("ptr", &self.raw)
            .finish()
    }
}

/// Information about a memory kind
#[derive(Debug, Clone)]
pub struct MemoryKind {
    /// String identifier for the memory kind
    pub kind: String,
    /// Numeric ID for the memory kind
    pub kind_id: i32,
}

impl MemoryDescription {
    /// Get the kind information for this memory description
    ///
    /// Returns a platform-dependent string and numeric ID that uniquely
    /// identifies the kind of memory space among those possible on this platform.
    pub fn kind(&self) -> Result<MemoryKind> {
        let mut args = PJRT_MemoryDescription_Kind_Args {
            struct_size: std::mem::size_of::<PJRT_MemoryDescription_Kind_Args>(),
            extension_start: std::ptr::null_mut(),
            memory_description: self.raw,
            kind: std::ptr::null(),
            kind_size: 0,
            kind_id: 0,
        };

        let ext_fn = self
            .ext
            .raw
            .PJRT_MemoryDescription_Kind
            .expect("PJRT_MemoryDescription_Kind not implemented");

        let err = unsafe { ext_fn(&mut args) };
        self.ext.api.err_or(err, ())?;

        let kind_str = if args.kind.is_null() || args.kind_size == 0 {
            String::new()
        } else {
            unsafe {
                let slice = std::slice::from_raw_parts(args.kind as *const u8, args.kind_size);
                String::from_utf8_lossy(slice).to_string()
            }
        };

        Ok(MemoryKind {
            kind: kind_str,
            kind_id: args.kind_id,
        })
    }
}

/// Memory descriptions information for a device
#[derive(Debug)]
pub struct DeviceMemoryDescriptions {
    /// List of memory descriptions
    pub descriptions: Vec<MemoryDescription>,
    /// Index of the default memory (-1 if there's no default)
    pub default_memory_index: isize,
}

impl MemoryDescriptionsExtension {
    /// Get all memory descriptions for a device description
    ///
    /// Returns all memory descriptions attached to this device.
    /// The memories are in no particular order.
    ///
    /// # Arguments
    ///
    /// * `device_description` - The device description to query
    pub fn get_memory_descriptions(
        &self,
        device_description: &DeviceDescription,
    ) -> Result<DeviceMemoryDescriptions> {
        let mut args = PJRT_DeviceDescription_MemoryDescriptions_Args {
            struct_size: std::mem::size_of::<PJRT_DeviceDescription_MemoryDescriptions_Args>(),
            extension_start: std::ptr::null_mut(),
            device_description: device_description.ptr as *mut PJRT_DeviceDescription,
            memory_descriptions: std::ptr::null(),
            num_memory_descriptions: 0,
            default_memory_index: 0,
        };

        let ext_fn = self
            .raw
            .PJRT_DeviceDescription_MemoryDescriptions
            .expect("PJRT_DeviceDescription_MemoryDescriptions not implemented");

        let err = unsafe { ext_fn(&mut args) };
        self.api.err_or(err, ())?;

        let descriptions =
            if args.memory_descriptions.is_null() || args.num_memory_descriptions == 0 {
                Vec::new()
            } else {
                unsafe {
                    let ptrs = std::slice::from_raw_parts(
                        args.memory_descriptions,
                        args.num_memory_descriptions,
                    );
                    ptrs.iter()
                        .map(|&ptr| MemoryDescription {
                            raw: ptr,
                            ext: MemoryDescriptionsExtension {
                                raw: self.raw.clone(),
                                api: self.api.clone(),
                            },
                        })
                        .collect()
                }
            };

        Ok(DeviceMemoryDescriptions {
            descriptions,
            default_memory_index: if args.default_memory_index == usize::MAX {
                -1
            } else {
                args.default_memory_index as isize
            },
        })
    }
}
