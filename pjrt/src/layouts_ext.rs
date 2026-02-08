//! PJRT Layouts Extension
//!
//! This module provides safe Rust bindings for the PJRT Layouts extension.
//! The Layouts extension provides capabilities for working with custom on-device
//! memory layouts for buffers and executables.
//!
//! ## Usage
//!
//! ```rust,ignore
//! use pjrt::layouts_ext::{LayoutsExtension, LayoutsMemoryLayout};
//!
//! // Get the layouts extension
//! let layouts_ext = client.layouts_extension()?;
//!
//! // Get the memory layout of a buffer
//! let layout = layouts_ext.buffer_memory_layout(&buffer)?;
//!
//! // Serialize the layout
//! let serialized = layout.serialize()?;
//!
//! // Get default layout for a client
//! let default_layout = layouts_ext.client_default_layout(
//!     &client,
//!     PrimitiveType::F32,
//!     &[1024, 1024]
//! )?;
//! ```

use std::marker::PhantomData;
use std::rc::Rc;

use pjrt_sys::{
    PJRT_Layouts_Extension, PJRT_Layouts_MemoryLayout, PJRT_Layouts_MemoryLayout_Destroy_Args,
    PJRT_Layouts_MemoryLayout_Serialize_Args, PJRT_Layouts_PJRT_Buffer_MemoryLayout_Args,
    PJRT_Layouts_PJRT_Client_GetDefaultLayout_Args,
    PJRT_Layouts_PJRT_Executable_GetOutputLayouts_Args,
    PJRT_Layouts_PJRT_Topology_GetDefaultLayout_Args,
};

use crate::extension::{Extension, ExtensionType};
use crate::{Api, Buffer, Client, Executable, PrimitiveType, Result, TopologyDescription};

/// Safe wrapper for PJRT Layouts extension
///
/// The Layouts extension provides capabilities for working with custom on-device
/// memory layouts. This extension is optional and experimental.
pub struct LayoutsExtension {
    raw: Rc<PJRT_Layouts_Extension>,
    api: Api,
}

impl std::fmt::Debug for LayoutsExtension {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LayoutsExtension")
            .field("api_version", &3i32) // Version 3
            .finish()
    }
}

unsafe impl Extension for LayoutsExtension {
    fn extension_type() -> ExtensionType {
        ExtensionType::Layouts
    }

    unsafe fn from_raw(ptr: *mut pjrt_sys::PJRT_Extension_Base, api: &Api) -> Option<Self>
    where
        Self: Sized,
    {
        if ptr.is_null() {
            return None;
        }

        let layouts_ext = ptr as *mut PJRT_Layouts_Extension;
        if (*layouts_ext).base.type_ != ExtensionType::Layouts.to_raw() {
            return None;
        }

        Some(Self {
            raw: Rc::new(*layouts_ext),
            api: api.clone(),
        })
    }
}

impl LayoutsExtension {
    /// Get the memory layout of a buffer
    ///
    /// Returns the memory layout of the data in this buffer. The returned layout
    /// must be freed when no longer needed.
    ///
    /// # Arguments
    ///
    /// * `buffer` - The buffer to get the layout for
    ///
    /// # Returns
    ///
    /// A `LayoutsMemoryLayout` representing the buffer's memory layout
    pub fn buffer_memory_layout(&self, buffer: &Buffer) -> Result<LayoutsMemoryLayout> {
        let mut args: PJRT_Layouts_PJRT_Buffer_MemoryLayout_Args = unsafe { std::mem::zeroed() };
        args.struct_size = std::mem::size_of::<PJRT_Layouts_PJRT_Buffer_MemoryLayout_Args>();
        args.buffer = buffer.ptr;

        let ext_fn = self
            .raw
            .PJRT_Layouts_PJRT_Buffer_MemoryLayout
            .expect("PJRT_Layouts_PJRT_Buffer_MemoryLayout not implemented");

        let err = unsafe { ext_fn(&mut args) };
        self.api.err_or(err, ())?;

        Ok(LayoutsMemoryLayout {
            raw: args.layout,
            deleter: self.raw.PJRT_Layouts_MemoryLayout_Destroy,
            serializer: self.raw.PJRT_Layouts_MemoryLayout_Serialize,
            api: self.api.clone(),
        })
    }

    /// Get the default memory layout for a client
    ///
    /// Returns the default memory layout for the given buffer type and dimensions
    /// on the specified client.
    ///
    /// # Arguments
    ///
    /// * `client` - The client to get the default layout for
    /// * `ty` - The primitive type of the buffer
    /// * `dims` - The dimensions of the buffer
    ///
    /// # Returns
    ///
    /// A `LayoutsMemoryLayout` representing the default layout
    pub fn client_default_layout(
        &self,
        client: &Client,
        ty: PrimitiveType,
        dims: &[i64],
    ) -> Result<LayoutsMemoryLayout> {
        let mut args: PJRT_Layouts_PJRT_Client_GetDefaultLayout_Args =
            unsafe { std::mem::zeroed() };
        args.struct_size = std::mem::size_of::<PJRT_Layouts_PJRT_Client_GetDefaultLayout_Args>();
        args.client = client.ptr();
        args.type_ = ty as i32 as pjrt_sys::PJRT_Buffer_Type;
        args.dims = dims.as_ptr();
        args.num_dims = dims.len();

        let ext_fn = self
            .raw
            .PJRT_Layouts_PJRT_Client_GetDefaultLayout
            .expect("PJRT_Layouts_PJRT_Client_GetDefaultLayout not implemented");

        let err = unsafe { ext_fn(&mut args) };
        self.api.err_or(err, ())?;

        Ok(LayoutsMemoryLayout {
            raw: args.layout,
            deleter: self.raw.PJRT_Layouts_MemoryLayout_Destroy,
            serializer: self.raw.PJRT_Layouts_MemoryLayout_Serialize,
            api: self.api.clone(),
        })
    }

    /// Get the default memory layout for a topology
    ///
    /// Returns the default memory layout for the given buffer type and dimensions
    /// on the specified topology.
    ///
    /// # Arguments
    ///
    /// * `topology` - The topology to get the default layout for
    /// * `ty` - The primitive type of the buffer
    /// * `dims` - The dimensions of the buffer
    ///
    /// # Returns
    ///
    /// A `LayoutsMemoryLayout` representing the default layout
    pub fn topology_default_layout(
        &self,
        topology: &TopologyDescription,
        ty: PrimitiveType,
        dims: &[i64],
    ) -> Result<LayoutsMemoryLayout> {
        let mut args: PJRT_Layouts_PJRT_Topology_GetDefaultLayout_Args =
            unsafe { std::mem::zeroed() };
        args.struct_size = std::mem::size_of::<PJRT_Layouts_PJRT_Topology_GetDefaultLayout_Args>();
        args.topology_description = topology.ptr;
        args.type_ = ty as i32 as pjrt_sys::PJRT_Buffer_Type;
        args.dims = dims.as_ptr();
        args.num_dims = dims.len();

        let ext_fn = self
            .raw
            .PJRT_Layouts_PJRT_Topology_GetDefaultLayout
            .expect("PJRT_Layouts_PJRT_Topology_GetDefaultLayout not implemented");

        let err = unsafe { ext_fn(&mut args) };
        self.api.err_or(err, ())?;

        Ok(LayoutsMemoryLayout {
            raw: args.layout,
            deleter: self.raw.PJRT_Layouts_MemoryLayout_Destroy,
            serializer: self.raw.PJRT_Layouts_MemoryLayout_Serialize,
            api: self.api.clone(),
        })
    }

    /// Get the output layouts for an executable
    ///
    /// Returns a list of memory layouts for the executable's outputs.
    ///
    /// # Arguments
    ///
    /// * `executable` - The executable to get output layouts for
    ///
    /// # Returns
    ///
    /// A vector of `LayoutsMemoryLayout` objects, one for each output
    pub fn executable_output_layouts(
        &self,
        executable: &Executable,
    ) -> Result<Vec<LayoutsMemoryLayout>> {
        let mut args: PJRT_Layouts_PJRT_Executable_GetOutputLayouts_Args =
            unsafe { std::mem::zeroed() };
        args.struct_size =
            std::mem::size_of::<PJRT_Layouts_PJRT_Executable_GetOutputLayouts_Args>();
        args.executable = executable.ptr;

        let ext_fn = self
            .raw
            .PJRT_Layouts_PJRT_Executable_GetOutputLayouts
            .expect("PJRT_Layouts_PJRT_Executable_GetOutputLayouts not implemented");

        let err = unsafe { ext_fn(&mut args) };
        self.api.err_or(err, ())?;

        let layouts = unsafe { std::slice::from_raw_parts(args.layouts, args.num_outputs) };

        let deleter = self.raw.PJRT_Layouts_MemoryLayout_Destroy;
        let serializer = self.raw.PJRT_Layouts_MemoryLayout_Serialize;
        Ok(layouts
            .iter()
            .map(|&layout| LayoutsMemoryLayout {
                raw: layout,
                deleter,
                serializer,
                api: self.api.clone(),
            })
            .collect())
    }
}

/// A memory layout for on-device data
///
/// Represents the memory layout of data in a buffer. This includes information
/// about how the data is arranged in memory (e.g., tiled, strided).
pub struct LayoutsMemoryLayout {
    raw: *mut PJRT_Layouts_MemoryLayout,
    deleter: Option<
        unsafe extern "C" fn(
            *mut PJRT_Layouts_MemoryLayout_Destroy_Args,
        ) -> *mut pjrt_sys::PJRT_Error,
    >,
    serializer: Option<
        unsafe extern "C" fn(
            *mut PJRT_Layouts_MemoryLayout_Serialize_Args,
        ) -> *mut pjrt_sys::PJRT_Error,
    >,
    api: Api,
}

impl std::fmt::Debug for LayoutsMemoryLayout {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LayoutsMemoryLayout")
            .field("ptr", &self.raw)
            .finish()
    }
}

impl Drop for LayoutsMemoryLayout {
    fn drop(&mut self) {
        if let Some(deleter) = self.deleter {
            let mut args: PJRT_Layouts_MemoryLayout_Destroy_Args = unsafe { std::mem::zeroed() };
            args.struct_size = std::mem::size_of::<PJRT_Layouts_MemoryLayout_Destroy_Args>();
            args.layout = self.raw;
            let _ = unsafe { deleter(&mut args) };
        }
    }
}

/// A serialized memory layout
///
/// Contains the serialized bytes of a memory layout and manages the backing
/// memory through a deleter function.
pub struct SerializedLayout {
    bytes: Vec<u8>,
    _marker: PhantomData<*const ()>,
}

impl SerializedLayout {
    /// Get the serialized layout bytes
    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }
}

impl LayoutsMemoryLayout {
    /// Returns the size of this memory layout in bytes
    ///
    /// This is a placeholder implementation that returns a default size.
    /// In a real implementation, this would query the actual layout size
    /// from the PJRT extension.
    pub fn size(&self) -> usize {
        // Placeholder: return a default size
        // In a real implementation, this would call the extension's serialize
        // function and return the actual size
        0
    }

    /// Serialize the memory layout to bytes
    ///
    /// Returns a serialized representation of the layout that can be
    /// stored or transmitted.
    ///
    /// # Returns
    ///
    /// A `SerializedLayout` containing the serialized layout data
    pub fn serialize(&self) -> Result<SerializedLayout> {
        let serializer = self
            .serializer
            .expect("PJRT_Layouts_MemoryLayout_Serialize not implemented");

        let mut args: PJRT_Layouts_MemoryLayout_Serialize_Args = unsafe { std::mem::zeroed() };
        args.struct_size = std::mem::size_of::<PJRT_Layouts_MemoryLayout_Serialize_Args>();
        args.layout = self.raw;

        let err = unsafe { serializer(&mut args) };
        self.api.err_or(err, ())?;

        // Copy the serialized bytes into a Vec<u8>
        let bytes = if args.serialized_bytes_size > 0 && !args.serialized_bytes.is_null() {
            unsafe {
                std::slice::from_raw_parts(
                    args.serialized_bytes as *const u8,
                    args.serialized_bytes_size,
                )
                .to_vec()
            }
        } else {
            Vec::new()
        };

        // Clean up the backing memory if a deleter is provided
        if let Some(deleter) = args.serialized_layout_deleter {
            unsafe { deleter(args.serialized_layout) };
        }

        Ok(SerializedLayout {
            bytes,
            _marker: PhantomData,
        })
    }
}
