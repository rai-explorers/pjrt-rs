//! PJRT Raw Buffer Extension
//!
//! This module provides safe Rust bindings for the PJRT Raw Buffer extension.
//! The Raw Buffer extension provides capabilities for constructing raw buffers
//! that alias PJRT_Buffers, allowing direct memory access for zero-copy operations.
//!
//! ## Warning
//!
//! This extension is both optional and experimental. ABI-breaking and other
//! incompatible changes may be introduced at any time.
//!
//! ## Usage
//!
//! ```rust,ignore
//! use pjrt::raw_buffer::{RawBufferExtension, RawBuffer};
//!
//! // Get the raw buffer extension
//! let raw_ext = api.get_extension::<RawBufferExtension>()?;
//!
//! // Create a raw buffer alias of an existing buffer
//! let raw_buffer = raw_ext.create_raw_alias(&buffer)?;
//!
//! // Get the host pointer for direct access
//! let host_ptr = raw_buffer.get_host_pointer()?;
//!
//! // Copy data to/from the raw buffer
//! let event = raw_buffer.copy_raw_host_to_device(&src_data, 0, data_len)?;
//! ```

use std::marker::PhantomData;
use std::rc::Rc;

use pjrt_sys::{
    PJRT_Event, PJRT_Memory, PJRT_RawBuffer, PJRT_RawBuffer_CopyRawDeviceToHost_Args,
    PJRT_RawBuffer_CopyRawHostToDevice_Args, PJRT_RawBuffer_CreateRawAliasOfBuffer_Args,
    PJRT_RawBuffer_Destroy_Args, PJRT_RawBuffer_Extension, PJRT_RawBuffer_GetHostPointer_Args,
    PJRT_RawBuffer_GetMemorySpace_Args, PJRT_RawBuffer_GetOnDeviceSizeInBytes_Args,
};

use crate::extension::{Extension, ExtensionType};
use crate::{Api, Buffer, Event, Memory, Result};

/// Safe wrapper for PJRT Raw Buffer extension
///
/// The Raw Buffer extension provides capabilities for constructing raw buffers
/// that alias PJRT_Buffers. This allows direct memory access for zero-copy
/// operations.
///
/// # Warning
///
/// This extension is both optional and experimental. ABI-breaking and other
/// incompatible changes may be introduced at any time.
pub struct RawBufferExtension {
    raw: Rc<PJRT_RawBuffer_Extension>,
    api: Api,
}

impl std::fmt::Debug for RawBufferExtension {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RawBufferExtension")
            .field("api_version", &2i32) // Version 2
            .finish()
    }
}

unsafe impl Extension for RawBufferExtension {
    fn extension_type() -> ExtensionType {
        ExtensionType::RawBuffer
    }

    unsafe fn from_raw(ptr: *mut pjrt_sys::PJRT_Extension_Base, api: &Api) -> Option<Self>
    where
        Self: Sized,
    {
        if ptr.is_null() {
            return None;
        }

        let raw_ext = ptr as *mut PJRT_RawBuffer_Extension;
        if (*raw_ext).base.type_ != ExtensionType::RawBuffer.to_raw() {
            return None;
        }

        Some(Self {
            raw: Rc::new((*raw_ext).clone()),
            api: api.clone(),
        })
    }
}

impl RawBufferExtension {
    /// Create a raw buffer alias of an existing buffer
    ///
    /// The raw buffer provides a view into the underlying buffer that can be
    /// used for direct memory access.
    ///
    /// # Arguments
    ///
    /// * `buffer` - The buffer to create an alias of
    ///
    /// # Returns
    ///
    /// A `RawBuffer` that aliases the original buffer
    pub fn create_raw_alias(&self, buffer: &Buffer) -> Result<RawBuffer<'_>> {
        let mut args = unsafe { std::mem::zeroed::<PJRT_RawBuffer_CreateRawAliasOfBuffer_Args>() };
        args.struct_size = std::mem::size_of::<PJRT_RawBuffer_CreateRawAliasOfBuffer_Args>();
        args.buffer = buffer.ptr;

        let ext_fn = self
            .raw
            .PJRT_RawBuffer_CreateRawAliasOfBuffer
            .expect("PJRT_RawBuffer_CreateRawAliasOfBuffer not implemented");

        let err = unsafe { ext_fn(&mut args) };
        self.api.err_or(err, ())?;

        Ok(RawBuffer {
            raw: args.raw_buffer,
            _marker: PhantomData,
        })
    }
}

/// A raw buffer that aliases a PJRT buffer
///
/// This provides direct memory access to buffer storage on the device.
/// The raw buffer is only valid as long as the original buffer exists.
pub struct RawBuffer<'a> {
    raw: *mut PJRT_RawBuffer,
    _marker: PhantomData<&'a ()>,
}

impl<'a> RawBuffer<'a> {
    /// Get the host pointer for direct memory access
    ///
    /// If the buffer is visible to the host, returns the base pointer for
    /// direct access. Returns None if the buffer is not host-visible.
    ///
    /// # Safety
    ///
    /// The returned pointer is only valid as long as the RawBuffer exists.
    /// Direct memory access bypasses PJRT's safety mechanisms.
    pub unsafe fn get_host_pointer(&self) -> Result<*mut std::ffi::c_void> {
        let mut args = unsafe { std::mem::zeroed::<PJRT_RawBuffer_GetHostPointer_Args>() };
        args.struct_size = std::mem::size_of::<PJRT_RawBuffer_GetHostPointer_Args>();
        args.buffer = self.raw;

        // This function pointer would come from the extension
        // For now, this is a placeholder showing the API design
        todo!("get_host_pointer requires RawBufferExtension reference")
    }

    /// Get the on-device size in bytes
    ///
    /// Returns the number of bytes of the buffer storage on the device.
    pub fn on_device_size(&self) -> Result<usize> {
        todo!("on_device_size requires RawBufferExtension reference")
    }

    /// Get the memory space
    ///
    /// Returns the memory space that this buffer is attached to.
    pub fn memory_space(&self) -> Result<Memory> {
        todo!("memory_space requires RawBufferExtension reference")
    }

    /// Copy raw data from host to device
    ///
    /// Copies data from host memory to the raw buffer.
    ///
    /// # Arguments
    ///
    /// * `src` - Source data on the host
    /// * `offset` - Offset in bytes within the buffer
    /// * `transfer_size` - Number of bytes to transfer
    ///
    /// # Returns
    ///
    /// An `Event` that completes when the transfer is done
    pub fn copy_raw_host_to_device<T>(&self, src: &[T], offset: i64) -> Result<Event> {
        let transfer_size = (src.len() * std::mem::size_of::<T>()) as i64;

        let mut args = unsafe { std::mem::zeroed::<PJRT_RawBuffer_CopyRawHostToDevice_Args>() };
        args.struct_size = std::mem::size_of::<PJRT_RawBuffer_CopyRawHostToDevice_Args>();
        args.buffer = self.raw;
        args.src = src.as_ptr() as *const std::ffi::c_void;
        args.offset = offset;
        args.transfer_size = transfer_size;

        todo!("copy_raw_host_to_device requires RawBufferExtension reference")
    }

    /// Copy raw data from device to host
    ///
    /// Copies data from the raw buffer to host memory.
    ///
    /// # Arguments
    ///
    /// * `dst` - Destination buffer on the host
    /// * `offset` - Offset in bytes within the buffer
    /// * `transfer_size` - Number of bytes to transfer
    ///
    /// # Returns
    ///
    /// An `Event` that completes when the transfer is done
    pub fn copy_raw_device_to_host<T>(&self, dst: &mut [T], offset: i64) -> Result<Event> {
        let transfer_size = (dst.len() * std::mem::size_of::<T>()) as i64;

        let mut args = unsafe { std::mem::zeroed::<PJRT_RawBuffer_CopyRawDeviceToHost_Args>() };
        args.struct_size = std::mem::size_of::<PJRT_RawBuffer_CopyRawDeviceToHost_Args>();
        args.buffer = self.raw;
        args.dst = dst.as_mut_ptr() as *mut std::ffi::c_void;
        args.offset = offset;
        args.transfer_size = transfer_size;

        todo!("copy_raw_device_to_host requires RawBufferExtension reference")
    }
}

impl<'a> Drop for RawBuffer<'a> {
    fn drop(&mut self) {
        // Note: The raw buffer is just an alias, we don't need to free it
        // The underlying PJRT_Buffer manages the actual memory
    }
}
