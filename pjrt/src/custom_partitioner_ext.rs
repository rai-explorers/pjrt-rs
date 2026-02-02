//! PJRT Custom Partitioner Extension
//!
//! This module provides safe Rust bindings for the PJRT Custom Partitioner extension.
//! The Custom Partitioner extension provides support for JAX custom call partitioning,
//! allowing custom operations to be partitioned across multiple devices.
//!
//! ## Usage
//!
//! ```rust,ignore
//! use pjrt::custom_partitioner::CustomPartitionerExtension;
//!
//! // Get the custom partitioner extension
//! let partitioner_ext = api.get_extension::<CustomPartitionerExtension>()?;
//!
//! // Register a custom partitioner
//! partitioner_ext.register_custom_partitioner("my_partitioner", callbacks)?;
//!
//! // Register a batch partitionable custom call
//! partitioner_ext.register_batch_partitionable("my_batch_op")?;
//! ```

use std::ffi::CString;
use std::rc::Rc;

use pjrt_sys::{
    PJRT_Custom_Partitioner_Extension, PJRT_Register_Batch_Partitionable_Args,
    PJRT_Register_Custom_Partitioner_Args,
};

use crate::extension::{Extension, ExtensionType};
use crate::{Api, Result};

/// Safe wrapper for PJRT Custom Partitioner extension
///
/// The Custom Partitioner extension provides support for JAX custom call
/// partitioning, allowing custom operations to be partitioned across
/// multiple devices for distributed execution.
pub struct CustomPartitionerExtension {
    raw: Rc<PJRT_Custom_Partitioner_Extension>,
    api: Api,
}

impl std::fmt::Debug for CustomPartitionerExtension {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CustomPartitionerExtension")
            .field("api_version", &1i32) // Version 1
            .finish()
    }
}

unsafe impl Extension for CustomPartitionerExtension {
    fn extension_type() -> ExtensionType {
        ExtensionType::CustomPartitioner
    }

    unsafe fn from_raw(ptr: *mut pjrt_sys::PJRT_Extension_Base, api: &Api) -> Option<Self>
    where
        Self: Sized,
    {
        if ptr.is_null() {
            return None;
        }

        let partitioner_ext = ptr as *mut PJRT_Custom_Partitioner_Extension;
        if (*partitioner_ext).base.type_ != ExtensionType::CustomPartitioner.to_raw() {
            return None;
        }

        Some(Self {
            raw: Rc::new(*partitioner_ext),
            api: api.clone(),
        })
    }
}

impl CustomPartitionerExtension {
    /// Register a custom partitioner
    ///
    /// Registers a custom partitioner with the specified name and callbacks.
    /// The callbacks handle partitioning logic, sharding inference, and
    /// sharding propagation.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the custom partitioner
    /// * `callbacks` - The callback functions for partitioning operations
    ///
    /// # Safety
    ///
    /// The callbacks must remain valid for the lifetime of the program.
    pub unsafe fn register_custom_partitioner(
        &self,
        name: &str,
        callbacks: *mut pjrt_sys::JAX_CustomCallPartitioner_Callbacks,
    ) -> Result<()> {
        let name_cstr = CString::new(name).expect("name contains null bytes");

        let mut args = unsafe { std::mem::zeroed::<PJRT_Register_Custom_Partitioner_Args>() };
        args.struct_size = std::mem::size_of::<PJRT_Register_Custom_Partitioner_Args>();
        args.name = name_cstr.as_ptr();
        args.name_size = name_cstr.count_bytes();
        args.callbacks = callbacks;

        let ext_fn = self
            .raw
            .register_custom_partitioner
            .expect("PJRT_Register_Custom_Partitioner not implemented");

        let err = unsafe { ext_fn(&mut args) };
        self.api.err_or(err, ())
    }

    /// Register a custom call as batch partitionable
    ///
    /// Registers a custom call that can be partitioned for batch processing
    /// across multiple devices.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the custom call
    pub fn register_batch_partitionable(&self, name: &str) -> Result<()> {
        let name_cstr = CString::new(name).expect("name contains null bytes");

        let mut args = unsafe { std::mem::zeroed::<PJRT_Register_Batch_Partitionable_Args>() };
        args.struct_size = std::mem::size_of::<PJRT_Register_Batch_Partitionable_Args>();
        args.name = name_cstr.as_ptr();
        args.name_size = name_cstr.count_bytes();

        let ext_fn = self
            .raw
            .register_batch_partitionable
            .expect("PJRT_Register_Batch_Partitionable not implemented");

        let err = unsafe { ext_fn(&mut args) };
        self.api.err_or(err, ())
    }
}
