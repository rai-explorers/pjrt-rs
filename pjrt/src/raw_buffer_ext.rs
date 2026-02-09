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
//! use pjrt::raw_buffer_ext::{RawBufferExtension, RawBuffer};
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
//! let event = unsafe { raw_buffer.copy_raw_host_to_device(&src_data, 0)? };
//! ```

use std::marker::PhantomData;
use std::rc::Rc;

use pjrt_sys::{
    PJRT_RawBuffer, PJRT_RawBuffer_CopyRawDeviceToHost_Args,
    PJRT_RawBuffer_CopyRawHostToDevice_Args, PJRT_RawBuffer_CreateRawAliasOfBuffer_Args,
    PJRT_RawBuffer_Extension, PJRT_RawBuffer_GetHostPointer_Args,
    PJRT_RawBuffer_GetMemorySpace_Args, PJRT_RawBuffer_GetOnDeviceSizeInBytes_Args,
};

use crate::extension::{Extension, ExtensionType};
use crate::{Api, Buffer, Client, Error, Event, Memory, Result};

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
            raw: Rc::new(*raw_ext),
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

        let ext_fn =
            self.raw
                .PJRT_RawBuffer_CreateRawAliasOfBuffer
                .ok_or(Error::NullFunctionPointer(
                    "PJRT_RawBuffer_CreateRawAliasOfBuffer",
                ))?;

        let err = unsafe { ext_fn(&mut args) };
        self.api.err_or(err, ())?;

        Ok(RawBuffer {
            raw: args.raw_buffer,
            ext: Rc::clone(&self.raw),
            client: buffer.client().clone(),
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
    ext: Rc<PJRT_RawBuffer_Extension>,
    client: Client,
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
        let ext_fn = self
            .ext
            .PJRT_RawBuffer_GetHostPointer
            .ok_or(Error::NullFunctionPointer("PJRT_RawBuffer_GetHostPointer"))?;

        let mut args: PJRT_RawBuffer_GetHostPointer_Args = unsafe { std::mem::zeroed() };
        args.struct_size = std::mem::size_of::<PJRT_RawBuffer_GetHostPointer_Args>();
        args.buffer = self.raw;

        let err = unsafe { ext_fn(&mut args) };
        self.client.api().err_or(err, ())?;

        Ok(args.host_pointer)
    }

    /// Get the on-device size in bytes
    ///
    /// Returns the number of bytes of the buffer storage on the device.
    pub fn on_device_size(&self) -> Result<usize> {
        let ext_fn =
            self.ext
                .PJRT_RawBuffer_GetOnDeviceSizeInBytes
                .ok_or(Error::NullFunctionPointer(
                    "PJRT_RawBuffer_GetOnDeviceSizeInBytes",
                ))?;

        let mut args: PJRT_RawBuffer_GetOnDeviceSizeInBytes_Args = unsafe { std::mem::zeroed() };
        args.struct_size = std::mem::size_of::<PJRT_RawBuffer_GetOnDeviceSizeInBytes_Args>();
        args.buffer = self.raw;

        let err = unsafe { ext_fn(&mut args) };
        self.client.api().err_or(err, ())?;

        Ok(args.on_device_size_in_bytes)
    }

    /// Get the memory space
    ///
    /// Returns the memory space that this buffer is attached to.
    pub fn memory_space(&self) -> Result<Memory> {
        let ext_fn = self
            .ext
            .PJRT_RawBuffer_GetMemorySpace
            .ok_or(Error::NullFunctionPointer("PJRT_RawBuffer_GetMemorySpace"))?;

        let mut args: PJRT_RawBuffer_GetMemorySpace_Args = unsafe { std::mem::zeroed() };
        args.struct_size = std::mem::size_of::<PJRT_RawBuffer_GetMemorySpace_Args>();
        args.buffer = self.raw;

        let err = unsafe { ext_fn(&mut args) };
        self.client.api().err_or(err, ())?;

        Ok(Memory::wrap(&self.client, args.memory_space))
    }

    /// Copy raw data from host to device
    ///
    /// Copies data from host memory to the raw buffer.
    ///
    /// # Arguments
    ///
    /// * `src` - Source data on the host
    /// * `offset` - Offset in bytes within the buffer
    ///
    /// # Returns
    ///
    /// An `Event` that completes when the transfer is done
    ///
    /// # Safety
    ///
    /// - `offset + size_of_val(src)` must not exceed the buffer's on-device size.
    /// - The caller must ensure no concurrent writes to the same region.
    pub unsafe fn copy_raw_host_to_device<T: Copy>(&self, src: &[T], offset: i64) -> Result<Event> {
        let ext_fn =
            self.ext
                .PJRT_RawBuffer_CopyRawHostToDevice
                .ok_or(Error::NullFunctionPointer(
                    "PJRT_RawBuffer_CopyRawHostToDevice",
                ))?;

        let mut args: PJRT_RawBuffer_CopyRawHostToDevice_Args = unsafe { std::mem::zeroed() };
        args.struct_size = std::mem::size_of::<PJRT_RawBuffer_CopyRawHostToDevice_Args>();
        args.buffer = self.raw;
        args.src = src.as_ptr() as *const std::ffi::c_void;
        args.offset = offset;
        args.transfer_size = std::mem::size_of_val(src) as i64;

        let err = unsafe { ext_fn(&mut args) };
        self.client.api().err_or(err, ())?;

        Ok(Event::wrap(self.client.api(), args.event))
    }

    /// Copy raw data from device to host
    ///
    /// Copies data from the raw buffer to host memory.
    ///
    /// # Arguments
    ///
    /// * `dst` - Destination buffer on the host
    /// * `offset` - Offset in bytes within the buffer
    ///
    /// # Returns
    ///
    /// An `Event` that completes when the transfer is done
    ///
    /// # Safety
    ///
    /// - `offset + size_of_val(dst)` must not exceed the buffer's on-device size.
    /// - The caller must ensure no concurrent writes to the same region.
    /// - The resulting bytes must represent valid `T` values.
    pub unsafe fn copy_raw_device_to_host<T: Copy>(
        &self,
        dst: &mut [T],
        offset: i64,
    ) -> Result<Event> {
        let ext_fn =
            self.ext
                .PJRT_RawBuffer_CopyRawDeviceToHost
                .ok_or(Error::NullFunctionPointer(
                    "PJRT_RawBuffer_CopyRawDeviceToHost",
                ))?;

        let mut args: PJRT_RawBuffer_CopyRawDeviceToHost_Args = unsafe { std::mem::zeroed() };
        args.struct_size = std::mem::size_of::<PJRT_RawBuffer_CopyRawDeviceToHost_Args>();
        args.buffer = self.raw;
        args.dst = dst.as_mut_ptr() as *mut std::ffi::c_void;
        args.offset = offset;
        args.transfer_size = std::mem::size_of_val(dst) as i64;

        let err = unsafe { ext_fn(&mut args) };
        self.client.api().err_or(err, ())?;

        Ok(Event::wrap(self.client.api(), args.event))
    }
}

impl<'a> Drop for RawBuffer<'a> {
    fn drop(&mut self) {
        // Free the raw buffer using the extension's destroy function
        if let Some(destroy_fn) = self.ext.PJRT_RawBuffer_Destroy {
            let mut args: pjrt_sys::PJRT_RawBuffer_Destroy_Args = unsafe { std::mem::zeroed() };
            args.struct_size = std::mem::size_of::<pjrt_sys::PJRT_RawBuffer_Destroy_Args>();
            args.buffer = self.raw;
            let _ = unsafe { destroy_fn(&mut args) };
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extension_type() {
        assert_eq!(
            RawBufferExtension::extension_type(),
            ExtensionType::RawBuffer
        );
    }

    #[test]
    fn test_from_raw_null_returns_none() {
        let api = unsafe { Api::empty_for_testing() };
        let result = unsafe { RawBufferExtension::from_raw(std::ptr::null_mut(), &api) };
        assert!(result.is_none());
    }

    #[test]
    fn test_from_raw_wrong_type_returns_none() {
        let api = unsafe { Api::empty_for_testing() };
        let mut ext = unsafe { std::mem::zeroed::<PJRT_RawBuffer_Extension>() };
        ext.base.type_ = ExtensionType::Example.to_raw();
        let result = unsafe {
            RawBufferExtension::from_raw(
                &mut ext as *mut PJRT_RawBuffer_Extension as *mut pjrt_sys::PJRT_Extension_Base,
                &api,
            )
        };
        assert!(result.is_none());
    }

    #[test]
    fn test_from_raw_correct_type() {
        let api = unsafe { Api::empty_for_testing() };
        let mut ext = unsafe { std::mem::zeroed::<PJRT_RawBuffer_Extension>() };
        ext.base.type_ = ExtensionType::RawBuffer.to_raw();
        let result = unsafe {
            RawBufferExtension::from_raw(
                &mut ext as *mut PJRT_RawBuffer_Extension as *mut pjrt_sys::PJRT_Extension_Base,
                &api,
            )
        };
        assert!(result.is_some());
    }

    #[test]
    fn test_debug_format() {
        let api = unsafe { Api::empty_for_testing() };
        let mut ext = unsafe { std::mem::zeroed::<PJRT_RawBuffer_Extension>() };
        ext.base.type_ = ExtensionType::RawBuffer.to_raw();
        let rb = unsafe {
            RawBufferExtension::from_raw(
                &mut ext as *mut PJRT_RawBuffer_Extension as *mut pjrt_sys::PJRT_Extension_Base,
                &api,
            )
        }
        .unwrap();
        let debug = format!("{:?}", rb);
        assert!(debug.contains("RawBufferExtension"));
        assert!(debug.contains("api_version"));
    }
}
