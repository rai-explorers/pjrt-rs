//! PJRT Stream Extension
//!
//! This module provides safe Rust bindings for the PJRT Stream extension.
//! The Stream extension allows for external buffer synchronization using
//! platform-specific stream handles.
//!
//! ## Usage
//!
//! ```rust,ignore
//! use pjrt::stream_ext::{StreamExtension, DeviceStream};
//!
//! // Get a stream for tracking external buffer readiness
//! let stream = client.stream_for_external_ready_events(&device)?;
//!
//! // Wait until a buffer is ready on the stream
//! stream.wait_until_buffer_ready(&buffer)?;
//! ```

use std::marker::PhantomData;
use std::rc::Rc;

use pjrt_sys::{
    PJRT_Get_Stream_For_External_Ready_Events_Args, PJRT_Stream_Extension,
    PJRT_Wait_Until_Buffer_Ready_On_Stream_Args,
};

use crate::extension::{Extension, ExtensionType};
use crate::{Api, Buffer, Device, Result};

/// Safe wrapper for PJRT Stream extension
///
/// The Stream extension provides capabilities for managing platform-specific
/// streams that can be used to track when externally-managed buffers are
/// ready to use on a device.
pub struct StreamExtension {
    raw: Rc<PJRT_Stream_Extension>,
    api: Api,
}

impl std::fmt::Debug for StreamExtension {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("StreamExtension")
            .field("api_version", &0i32) // Version 0
            .finish()
    }
}

unsafe impl Extension for StreamExtension {
    fn extension_type() -> ExtensionType {
        ExtensionType::Stream
    }

    unsafe fn from_raw(ptr: *mut pjrt_sys::PJRT_Extension_Base, api: &Api) -> Option<Self>
    where
        Self: Sized,
    {
        if ptr.is_null() {
            return None;
        }

        // Cast the base extension to Stream extension
        let stream_ext = ptr as *mut PJRT_Stream_Extension;
        if (*stream_ext).base.type_ != ExtensionType::Stream.to_raw() {
            return None;
        }

        Some(Self {
            raw: Rc::new(*stream_ext),
            api: api.clone(),
        })
    }
}

impl StreamExtension {
    /// Get a platform-specific stream handle for tracking external buffer readiness
    ///
    /// This stream handle should be used to track when an externally-managed buffer
    /// is ready to use on the specified device. The returned stream is platform-specific
    /// (e.g., CUDA stream for NVIDIA GPUs).
    ///
    /// # Arguments
    ///
    /// * `device` - The device on which the stream will be used
    ///
    /// # Returns
    ///
    /// A `DeviceStream` that wraps the platform-specific stream handle
    pub fn stream_for_external_ready_events(&self, device: &Device) -> Result<DeviceStream> {
        let mut args: PJRT_Get_Stream_For_External_Ready_Events_Args =
            unsafe { std::mem::zeroed() };
        args.struct_size = std::mem::size_of::<PJRT_Get_Stream_For_External_Ready_Events_Args>();
        args.device = device.ptr;

        let ext_fn = self
            .raw
            .get_stream
            .expect("PJRT_Get_Stream_For_External_Ready_Events not implemented");

        let err = unsafe { ext_fn(&mut args) };
        self.api.err_or(err, ())?;

        Ok(DeviceStream {
            stream: args.stream,
            waiter: self.raw.wait_stream,
            api: self.api.clone(),
            _marker: PhantomData,
        })
    }
}

/// A platform-specific stream handle for tracking buffer readiness
///
/// This represents a handle to a platform-specific stream (e.g., CUDA stream)
/// that can be used to synchronize external buffer operations.
pub struct DeviceStream {
    stream: isize, // intptr_t
    waiter: Option<
        unsafe extern "C" fn(
            *mut PJRT_Wait_Until_Buffer_Ready_On_Stream_Args,
        ) -> *mut pjrt_sys::PJRT_Error,
    >,
    api: Api,
    _marker: PhantomData<*const ()>, // Not Send + Sync
}

impl DeviceStream {
    /// Wait until the specified buffer is ready on this stream
    ///
    /// This method blocks until the buffer's data is ready for use on the
    /// platform-specific stream represented by this handle.
    ///
    /// # Arguments
    ///
    /// * `buffer` - The buffer to wait for
    ///
    /// # Returns
    ///
    /// `Ok(())` when the buffer is ready, or an error if the wait fails
    pub fn wait_until_buffer_ready(&self, buffer: &Buffer) -> Result<()> {
        let waiter = self
            .waiter
            .expect("PJRT_Wait_Until_Buffer_Ready_On_Stream not implemented");

        let mut args: PJRT_Wait_Until_Buffer_Ready_On_Stream_Args = unsafe { std::mem::zeroed() };
        args.struct_size = std::mem::size_of::<PJRT_Wait_Until_Buffer_Ready_On_Stream_Args>();
        args.stream = self.stream;
        args.buffer = buffer.ptr;

        let err = unsafe { waiter(&mut args) };
        self.api.err_or(err, ())
    }
}

/// Extension trait for accessing stream extension from Api
pub trait StreamExt {
    /// Get the Stream extension if available
    fn stream_extension(&self) -> Option<StreamExtension>;
}

impl StreamExt for Api {
    fn stream_extension(&self) -> Option<StreamExtension> {
        let ext_start = self.extension_start();
        unsafe { StreamExtension::from_raw(ext_start, self) }
    }
}
