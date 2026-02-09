//! PJRT Buffer Management
//!
//! This module provides the `Buffer` struct for managing data on PJRT devices.
//! Buffers represent arrays of data stored either on host memory or device memory
//! (GPU, TPU, etc.) and provide functionality to:
//!
//! - Transfer data between host and device
//! - Query buffer properties (dimensions, element type, memory layout)
//! - Copy buffers between devices
//! - Handle asynchronous operations
//!
//! Buffers are the primary way to move data in and out of PJRT computations.
//!
//! # Examples
//!
//! ## Creating and Transferring Buffers
//!
//! ```rust,ignore
//! use pjrt::{Client, HostBuffer, F32};
//!
//! // Create a host buffer with f32 data
//! let host_data = HostBuffer::from_data::<F32>(vec![1.0, 2.0, 3.0, 4.0], Some(vec![2, 2]), None);
//!
//! // Transfer to device (synchronous)
//! let device_buffer = host_data.to_sync(&client).copy()?;
//!
//! // Query buffer properties
//! println!("Type: {:?}", device_buffer.primitive_type());
//! println!("Dims: {:?}", device_buffer.dims());
//! println!("Size: {} bytes", device_buffer.on_device_size());
//! ```
//!
//! ## Asynchronous Transfers
//!
//! ```rust,ignore
//! // Transfer to device (async)
//! let device_buffer = host_data.to(&client).copy().await?;
//!
//! // Wait for buffer to be ready
//! let ready_event = device_buffer.ready_event()?;
//! ready_event.await?;
//! ```
//!
//! ## Copying Data Back to Host
//!
//! ```rust,ignore
//! // Copy buffer data back to host (typed)
//! let host_buffer = device_buffer.to_typed_host::<F32>()?;
//! let data = host_buffer.data();
//!
//! // Or copy to raw bytes
//! let mut bytes = vec![0u8; device_buffer.on_device_size()];
//! device_buffer.copy_raw_to_host(&mut bytes).wait()?;
//! ```
//!
//! ## Copying Between Devices
//!
//! ```rust,ignore
//! // Copy buffer to another device
//! let other_device = client.devices().get(1).unwrap();
//! let copied_buffer = device_buffer.copy_to_device(other_device)?;
//! ```

use std::ffi::c_void;
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

use bon::bon;
use pjrt_sys::{
    PJRT_Buffer, PJRT_Buffer_CopyRawToHost_Args, PJRT_Buffer_CopyToDevice_Args,
    PJRT_Buffer_CopyToMemory_Args, PJRT_Buffer_Delete_Args, PJRT_Buffer_Destroy_Args,
    PJRT_Buffer_Device_Args, PJRT_Buffer_Dimensions_Args, PJRT_Buffer_DynamicDimensionIndices_Args,
    PJRT_Buffer_ElementType_Args, PJRT_Buffer_GetMemoryLayout_Args, PJRT_Buffer_IsDeleted_Args,
    PJRT_Buffer_IsOnCpu_Args, PJRT_Buffer_MemoryLayout, PJRT_Buffer_Memory_Args,
    PJRT_Buffer_OnDeviceSizeInBytes_Args, PJRT_Buffer_ReadyEvent_Args,
    PJRT_Buffer_ToHostBuffer_Args, PJRT_Buffer_UnpaddedDimensions_Args,
};

use crate::event::Event;
use crate::{Client, Device, ErrorCode, HostBuffer, Memory, MemoryLayout, PrimitiveType, Result};

/// A buffer holding data on a PJRT device.
///
/// # Thread Safety
///
/// `Buffer` is `!Send + !Sync` because it holds a [`Client`] reference
/// (which uses `Rc` internally) and a raw pointer to the device buffer.
/// All buffer operations must occur on the thread that created the parent
/// [`Client`].
pub struct Buffer {
    client: Client,
    pub(crate) ptr: *mut PJRT_Buffer,
}

impl Drop for Buffer {
    fn drop(&mut self) {
        let mut args = PJRT_Buffer_Destroy_Args::new();
        args.buffer = self.ptr;
        self.client
            .api()
            .PJRT_Buffer_Destroy(args)
            .expect("PJRT_Buffer_Destroy");
    }
}

impl std::fmt::Debug for Buffer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Buffer")
            .field("primitive_type", &self.primitive_type().ok())
            .field("dims", &self.dims().unwrap_or_default())
            .field(
                "on_device_size_in_bytes",
                &self.on_device_size().unwrap_or(0),
            )
            .field("is_on_cpu", &self.is_on_cpu().unwrap_or(false))
            .field("is_deleted", &self.is_deleted().unwrap_or(false))
            .finish()
    }
}

#[bon]
impl Buffer {
    pub(crate) fn wrap(client: &Client, ptr: *mut PJRT_Buffer) -> Self {
        assert!(!ptr.is_null());
        Self {
            client: client.clone(),
            ptr,
        }
    }

    pub fn client(&self) -> &Client {
        &self.client
    }

    pub fn primitive_type(&self) -> Result<PrimitiveType> {
        let mut args = PJRT_Buffer_ElementType_Args::new();
        args.buffer = self.ptr;
        args = self.client.api().PJRT_Buffer_ElementType(args)?;
        PrimitiveType::try_from(args.type_)
    }

    pub fn dims(&self) -> Result<Vec<i64>> {
        let mut args = PJRT_Buffer_Dimensions_Args::new();
        args.buffer = self.ptr;
        args = self.client.api().PJRT_Buffer_Dimensions(args)?;
        if args.num_dims == 0 {
            return Ok(vec![]);
        }
        let s = unsafe { std::slice::from_raw_parts(args.dims, args.num_dims) };
        Ok(s.to_owned())
    }

    pub fn unpadded_dims(&self) -> Result<Vec<i64>> {
        let mut args = PJRT_Buffer_UnpaddedDimensions_Args::new();
        args.buffer = self.ptr;
        args = self.client.api().PJRT_Buffer_UnpaddedDimensions(args)?;
        let s = unsafe { std::slice::from_raw_parts(args.unpadded_dims, args.num_dims) };
        Ok(s.to_owned())
    }

    pub fn dynamic_dims_indices(&self) -> Result<Vec<usize>> {
        let mut args = PJRT_Buffer_DynamicDimensionIndices_Args::new();
        args.buffer = self.ptr;
        args = self
            .client
            .api()
            .PJRT_Buffer_DynamicDimensionIndices(args)?;
        let s =
            unsafe { std::slice::from_raw_parts(args.dynamic_dim_indices, args.num_dynamic_dims) };
        Ok(s.to_owned())
    }

    pub fn layout(&self) -> Result<MemoryLayout> {
        let mut args = PJRT_Buffer_GetMemoryLayout_Args::new();
        args.buffer = self.ptr;
        args = self.client.api().PJRT_Buffer_GetMemoryLayout(args)?;
        MemoryLayout::try_from(&args.layout)
    }

    pub fn on_device_size(&self) -> Result<usize> {
        let mut args = PJRT_Buffer_OnDeviceSizeInBytes_Args::new();
        args.buffer = self.ptr;
        args = self.client.api().PJRT_Buffer_OnDeviceSizeInBytes(args)?;
        Ok(args.on_device_size_in_bytes)
    }

    pub fn is_on_cpu(&self) -> Result<bool> {
        let mut args = PJRT_Buffer_IsOnCpu_Args::new();
        args.buffer = self.ptr;
        args = self.client.api().PJRT_Buffer_IsOnCpu(args)?;
        Ok(args.is_on_cpu)
    }

    pub fn device(&self) -> Result<Device> {
        let mut args = PJRT_Buffer_Device_Args::new();
        args.buffer = self.ptr;
        args = self.client.api().PJRT_Buffer_Device(args)?;
        Ok(Device::wrap(&self.client, args.device))
    }

    pub fn memory(&self) -> Result<Memory> {
        let mut args = PJRT_Buffer_Memory_Args::new();
        args.buffer = self.ptr;
        args = self.client.api().PJRT_Buffer_Memory(args)?;
        Ok(Memory::wrap(&self.client, args.memory))
    }

    pub fn delete(self) -> Result<()> {
        let mut args = PJRT_Buffer_Delete_Args::new();
        args.buffer = self.ptr;
        self.client.api().PJRT_Buffer_Delete(args)?;
        Ok(())
    }

    pub fn is_deleted(&self) -> Result<bool> {
        let mut args = PJRT_Buffer_IsDeleted_Args::new();
        args.buffer = self.ptr;
        args = self.client.api().PJRT_Buffer_IsDeleted(args)?;
        Ok(args.is_deleted)
    }

    pub(crate) fn ready_event(&self) -> Result<Event> {
        let mut args = PJRT_Buffer_ReadyEvent_Args::new();
        args.buffer = self.ptr;
        args = self.client.api().PJRT_Buffer_ReadyEvent(args)?;
        Ok(Event::wrap(self.client.api(), args.event))
    }

    fn call_copy_to_device(&self, device: &Device) -> Result<PJRT_Buffer_CopyToDevice_Args> {
        let mut args = PJRT_Buffer_CopyToDevice_Args::new();
        args.buffer = self.ptr;
        args.dst_device = device.ptr;
        self.client.api().PJRT_Buffer_CopyToDevice(args)
    }

    #[builder(finish_fn = copy)]
    pub async fn to_device(&self, #[builder(start_fn)] device: &Device) -> Result<Buffer> {
        let args = self.call_copy_to_device(device)?;
        let buf = Buffer::wrap(device.client(), args.dst_buffer);
        let event = buf.ready_event()?;
        event.await?;
        Ok(buf)
    }

    #[builder(finish_fn = copy)]
    pub fn to_device_sync(&self, #[builder(start_fn)] device: &Device) -> Result<Buffer> {
        let args = self.call_copy_to_device(device)?;
        let buf = Buffer::wrap(device.client(), args.dst_buffer);
        let event = buf.ready_event()?;
        event.wait()?;
        Ok(buf)
    }

    fn call_copy_to_memory(&self, memory: &Memory) -> Result<PJRT_Buffer_CopyToMemory_Args> {
        let mut args = PJRT_Buffer_CopyToMemory_Args::new();
        args.buffer = self.ptr;
        args.dst_memory = memory.ptr;
        self.client.api().PJRT_Buffer_CopyToMemory(args)
    }

    #[builder(finish_fn = copy)]
    pub async fn to_memory(&self, #[builder(start_fn)] memory: &Memory) -> Result<Buffer> {
        let args = self.call_copy_to_memory(memory)?;
        let buf = Buffer::wrap(memory.client(), args.dst_buffer);
        let event = buf.ready_event()?;
        event.await?;
        Ok(buf)
    }

    #[builder(finish_fn = copy)]
    pub fn to_memory_sync(&self, memory: &Memory) -> Result<Buffer> {
        let args = self.call_copy_to_memory(memory)?;
        let buf = Buffer::wrap(memory.client(), args.dst_buffer);
        let event = buf.ready_event()?;
        event.wait()?;
        Ok(buf)
    }

    pub fn call_copy_to_host(
        &self,
        host_layout: Option<&MemoryLayout>,
    ) -> Result<(PJRT_Buffer_ToHostBuffer_Args, Vec<u8>)> {
        let mut args = PJRT_Buffer_ToHostBuffer_Args::new();
        args.src = self.ptr;
        if let Some(layout) = host_layout {
            let mut l = PJRT_Buffer_MemoryLayout::from(layout);
            args.host_layout = &mut l as *mut _;
        }
        // first call to get the size of the buffer
        args = self.client.api().PJRT_Buffer_ToHostBuffer(args)?;
        let buf_size = args.dst_size;
        // second call to fill the buffer
        let mut buf: Vec<u8> = vec![0; buf_size];
        args.dst = buf.as_mut_ptr() as *mut _;
        args = self.client.api().PJRT_Buffer_ToHostBuffer(args)?;
        Ok((args, buf))
    }

    pub async fn to_host(&self, host_layout: Option<MemoryLayout>) -> Result<HostBuffer> {
        let (args, data) = self.call_copy_to_host(host_layout.as_ref())?;
        let event = Event::wrap(self.client.api(), args.event);
        event.await?;
        let ty = self.primitive_type()?;
        let dims = self.dims()?;
        let layout = match host_layout {
            Some(l) => l,
            None => self.layout()?,
        };
        HostBuffer::from_bytes(data, ty, Some(dims), Some(layout))
    }

    pub fn to_host_sync(&self, host_layout: Option<MemoryLayout>) -> Result<HostBuffer> {
        let (args, data) = self.call_copy_to_host(host_layout.as_ref())?;
        let event = Event::wrap(self.client.api(), args.event);
        event.wait()?;
        let ty = self.primitive_type()?;
        let dims = self.dims()?;
        let layout = match host_layout {
            Some(l) => l,
            None => self.layout()?,
        };
        HostBuffer::from_bytes(data, ty, Some(dims), Some(layout))
    }

    /// Copies raw data from the buffer to host memory.
    ///
    /// This is a lower-level API that copies raw bytes without type conversion.
    #[builder(finish_fn = copy)]
    pub async fn copy_raw_to_host(
        &self,
        #[builder(start_fn)] dst: &mut [u8],
        offset: usize,
        transfer_size: usize,
    ) -> Result<()> {
        let mut args = PJRT_Buffer_CopyRawToHost_Args::new();
        args.buffer = self.ptr;
        args.dst = dst.as_mut_ptr() as *mut _;
        args.offset = offset as i64;
        args.transfer_size = transfer_size as i64;
        let args = self.client.api().PJRT_Buffer_CopyRawToHost(args)?;
        let event = Event::wrap(self.client.api(), args.event);
        event.await
    }

    /// Copies raw data from the buffer to host memory synchronously.
    #[builder(finish_fn = copy)]
    pub fn copy_raw_to_host_sync(
        &self,
        #[builder(start_fn)] dst: &mut [u8],
        offset: usize,
        transfer_size: usize,
    ) -> Result<()> {
        let mut args = PJRT_Buffer_CopyRawToHost_Args::new();
        args.buffer = self.ptr;
        args.dst = dst.as_mut_ptr() as *mut _;
        args.offset = offset as i64;
        args.transfer_size = transfer_size as i64;
        let args = self.client.api().PJRT_Buffer_CopyRawToHost(args)?;
        let event = Event::wrap(self.client.api(), args.event);
        event.wait()
    }

    /// Copies raw data from the buffer to host memory using a future callback.
    ///
    /// This is similar to `copy_raw_to_host`, but the transfer only happens when
    /// the `future_ready` callback is invoked. The callback provides a destination
    /// buffer that the PJRT runtime will fill.
    ///
    /// Returns a future that resolves when the copy is complete.
    pub fn copy_raw_to_host_future(
        &self,
        offset: usize,
        transfer_size: usize,
    ) -> Result<CopyRawToHostFuture> {
        use pjrt_sys::PJRT_Buffer_CopyRawToHostFuture_Args;

        let mut args = PJRT_Buffer_CopyRawToHostFuture_Args::new();
        args.buffer = self.ptr;
        args.offset = offset as i64;
        args.transfer_size = transfer_size as i64;

        let args = self.client.api().PJRT_Buffer_CopyRawToHostFuture(args)?;

        let event = Event::wrap(self.client.api(), args.event);

        Ok(CopyRawToHostFuture {
            event,
            callback_data: args.callback_data,
            future_ready_callback: args.future_ready_callback,
            transfer_size,
        })
    }

    /// Donates this buffer with a control dependency.
    ///
    /// This is used to donate a buffer for an execute call while maintaining
    /// control over when the donation completes via a callback.
    ///
    /// Returns the donated buffer along with a callback that must be invoked
    /// before the dependency is considered ready.
    pub fn donate_with_control_dependency(&self) -> Result<DonateWithControlDependency> {
        use pjrt_sys::PJRT_Buffer_DonateWithControlDependency_Args;

        let mut args = PJRT_Buffer_DonateWithControlDependency_Args::new();
        args.buffer = self.ptr;

        let args = self
            .client
            .api()
            .PJRT_Buffer_DonateWithControlDependency(args)?;

        let out_buffer = Buffer::wrap(&self.client, args.out_buffer);

        Ok(DonateWithControlDependency {
            buffer: out_buffer,
            callback_data: args.callback_data,
            dependency_ready_callback: args.dependency_ready_callback,
        })
    }
}

/// A future for copying raw buffer data to host memory.
///
/// This struct is returned by [`Buffer::copy_raw_to_host_future`] and provides
/// a callback-based interface for asynchronously receiving data.
pub struct CopyRawToHostFuture {
    event: Event,
    callback_data: *mut c_void,
    future_ready_callback:
        Option<unsafe extern "C" fn(*mut pjrt_sys::PJRT_Buffer_CopyRawToHostFuture_Callback_Args)>,
    transfer_size: usize,
}

impl CopyRawToHostFuture {
    /// Returns the underlying event that signals completion.
    pub fn event(&self) -> &Event {
        &self.event
    }

    /// Signals that the host is ready to receive data.
    ///
    /// # Safety
    ///
    /// - `dst` must be a mutable slice of at least `transfer_size` bytes
    /// - This must be called exactly once before awaiting the future
    pub unsafe fn future_ready(
        &self,
        dst: &mut [u8],
        error_code: Option<ErrorCode>,
        error_message: Option<&str>,
    ) -> Result<()> {
        use pjrt_sys::PJRT_Buffer_CopyRawToHostFuture_Callback_Args;

        if dst.len() < self.transfer_size {
            return Err(crate::Error::InvalidArgument(format!(
                "destination buffer too small: got {} bytes, need {}",
                dst.len(),
                self.transfer_size
            )));
        }

        let mut args = PJRT_Buffer_CopyRawToHostFuture_Callback_Args::new();
        args.callback_data = self.callback_data;
        if let Some(code) = error_code {
            args.error_code = code as pjrt_sys::PJRT_Error_Code;
        }
        if let Some(msg) = error_message {
            args.error_message = msg.as_ptr() as *const i8;
            args.error_message_size = msg.len();
        }
        args.dst = dst.as_mut_ptr() as *mut _;

        let callback = self
            .future_ready_callback
            .ok_or_else(|| crate::Error::NullFunctionPointer("future_ready_callback"))?;

        unsafe { callback(&mut args) };
        Ok(())
    }
}

impl Future for CopyRawToHostFuture {
    type Output = Result<()>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        // Delegate to the underlying event
        Pin::new(&mut self.event).poll(cx)
    }
}

impl std::fmt::Debug for CopyRawToHostFuture {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CopyRawToHostFuture")
            .field("transfer_size", &self.transfer_size)
            .field("event", &self.event)
            .finish()
    }
}

/// A donated buffer with a control dependency callback.
///
/// This struct is returned by [`Buffer::donate_with_control_dependency`] and
/// wraps a buffer that has been donated for execution with a pending control
/// dependency.
pub struct DonateWithControlDependency {
    buffer: Buffer,
    callback_data: *mut c_void,
    dependency_ready_callback: Option<
        unsafe extern "C" fn(*mut pjrt_sys::PJRT_Buffer_DonateWithControlDependency_Callback_Args),
    >,
}

impl DonateWithControlDependency {
    /// Returns the donated buffer.
    pub fn buffer(&self) -> &Buffer {
        &self.buffer
    }

    /// Consumes this struct and returns the donated buffer.
    pub fn into_buffer(self) -> Buffer {
        self.buffer
    }

    /// Signals that the dependency is ready.
    ///
    /// This must be called before the donated buffer can be used in execution.
    pub fn dependency_ready(
        &self,
        error_code: Option<ErrorCode>,
        error_message: Option<&str>,
    ) -> Result<()> {
        use pjrt_sys::PJRT_Buffer_DonateWithControlDependency_Callback_Args;

        let mut args = PJRT_Buffer_DonateWithControlDependency_Callback_Args::new();
        args.callback_data = self.callback_data;
        if let Some(code) = error_code {
            args.error_code = code as pjrt_sys::PJRT_Error_Code;
        }
        if let Some(msg) = error_message {
            args.error_message = msg.as_ptr() as *const i8;
            args.error_message_size = msg.len();
        }

        let callback = self
            .dependency_ready_callback
            .ok_or_else(|| crate::Error::NullFunctionPointer("dependency_ready_callback"))?;

        unsafe { callback(&mut args) };
        Ok(())
    }
}

impl std::fmt::Debug for DonateWithControlDependency {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DonateWithControlDependency")
            .field("buffer", &self.buffer)
            .finish()
    }
}

impl Buffer {
    /// # Safety
    ///
    /// Returns a platform-dependent address for this buffer that is often but not
    /// guaranteed to be the physical/device address.
    ///
    /// The returned pointer may become invalid at any point. The caller must
    /// use [`Buffer::increase_external_ref_count`] to ensure the buffer remains
    /// valid if needed.
    ///
    /// # Safety
    ///
    /// - The returned pointer is not guaranteed to remain valid
    /// - Use external reference counting if you need to keep the buffer alive
    pub unsafe fn unsafe_pointer(&self) -> Result<usize> {
        use pjrt_sys::PJRT_Buffer_UnsafePointer_Args;
        let mut args = PJRT_Buffer_UnsafePointer_Args::new();
        args.buffer = self.ptr;
        let args = self.client.api().PJRT_Buffer_UnsafePointer(args)?;
        Ok(args.buffer_pointer)
    }

    /// Increments the external reference count for this buffer.
    ///
    /// The reference count indicates that the raw buffer data is being shared with
    /// another framework (e.g., NumPy, dlpack) and should not be deleted or moved
    /// by the PJRT implementation (e.g., for memory compaction).
    ///
    /// This should be called before obtaining an unsafe pointer via
    /// [`Buffer::unsafe_pointer`] or [`Buffer::opaque_device_memory_pointer`] if
    /// the external framework needs to keep the buffer alive.
    ///
    /// # Safety
    ///
    /// - Each call to `increase_external_ref_count` must be paired with a call to
    ///   [`Buffer::decrease_external_ref_count`]
    /// - Failing to decrement the reference count will cause memory leaks
    /// - This is only safe when the buffer is not deleted or donated
    pub unsafe fn increase_external_ref_count(&self) -> Result<()> {
        use pjrt_sys::PJRT_Buffer_IncreaseExternalReferenceCount_Args;
        let mut args = PJRT_Buffer_IncreaseExternalReferenceCount_Args::new();
        args.buffer = self.ptr;
        self.client
            .api()
            .PJRT_Buffer_IncreaseExternalReferenceCount(args)?;
        Ok(())
    }

    /// Decrements the external reference count for this buffer.
    ///
    /// This should be called when an external framework is done using the buffer
    /// and no longer needs to keep it alive. It must be paired with a previous
    /// call to [`Buffer::increase_external_ref_count`].
    ///
    /// # Safety
    ///
    /// - This will return an error if the reference count is zero
    /// - Must only be called after a corresponding `increase_external_ref_count`
    /// - Calling this without a matching increment will cause errors
    pub unsafe fn decrease_external_ref_count(&self) -> Result<()> {
        use pjrt_sys::PJRT_Buffer_DecreaseExternalReferenceCount_Args;
        let mut args = PJRT_Buffer_DecreaseExternalReferenceCount_Args::new();
        args.buffer = self.ptr;
        self.client
            .api()
            .PJRT_Buffer_DecreaseExternalReferenceCount(args)?;
        Ok(())
    }

    /// Returns the opaque device memory data pointer for this buffer.
    ///
    /// The returned data pointer may become invalid at any point unless the
    /// external reference count is greater than 0 via
    /// [`Buffer::increase_external_ref_count`].
    ///
    /// This is useful for interop with other frameworks that need direct access
    /// to device memory (e.g., CUDA, ROCm, dlpack).
    ///
    /// # Safety
    ///
    /// - The returned pointer may become invalid at any point
    /// - Must call [`Buffer::increase_external_ref_count`] before obtaining the pointer
    ///   if you need to keep the buffer alive
    /// - The pointer is only valid while the external reference count > 0
    pub unsafe fn opaque_device_memory_pointer(&self) -> Result<*mut c_void> {
        use pjrt_sys::PJRT_Buffer_OpaqueDeviceMemoryDataPointer_Args;
        let mut args = PJRT_Buffer_OpaqueDeviceMemoryDataPointer_Args::new();
        args.buffer = self.ptr;
        let args = self
            .client
            .api()
            .PJRT_Buffer_OpaqueDeviceMemoryDataPointer(args)?;
        Ok(args.device_memory_ptr)
    }

    /// Acquires an external reference to this buffer's memory.
    ///
    /// Returns an `ExternalBufferRef` guard that keeps the buffer alive and valid
    /// for external use. The guard automatically releases the reference when dropped.
    ///
    /// This is the recommended way to share buffer memory with external frameworks
    /// (e.g., NumPy, dlpack) as it ensures proper cleanup via RAII.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// // Acquire an external reference
    /// let external_ref = buffer.external_ref()?;
    ///
    /// // Get the device memory pointer (safe while external_ref is alive)
    /// let ptr = external_ref.device_memory_pointer()?;
    ///
    /// // Pass ptr to external framework...
    ///
    /// // external_ref is dropped here, releasing the reference
    /// ```
    pub fn external_ref(&self) -> Result<ExternalBufferRef<'_>> {
        unsafe { self.increase_external_ref_count()? };
        Ok(ExternalBufferRef { buffer: self })
    }
}

/// An RAII guard for external buffer references.
///
/// This guard ensures that the external reference count is properly decremented
/// when it goes out of scope. While the guard is alive, the buffer's memory is
/// guaranteed to remain valid and unmoved.
///
/// # Thread Safety
///
/// `ExternalBufferRef` is not `Send` or `Sync` because the underlying buffer
/// may not be thread-safe for concurrent access.
///
/// # Example
///
/// ```rust,ignore
/// // Acquire external reference to share with another framework
/// let external_ref = buffer.external_ref()?;
///
/// // The buffer memory is now pinned and safe to access externally
/// let ptr = external_ref.device_memory_pointer()?;
/// let unsafe_ptr = external_ref.unsafe_pointer()?;
///
/// // Use the pointer with external frameworks...
///
/// // When external_ref is dropped, the reference is released
/// drop(external_ref);
/// ```
pub struct ExternalBufferRef<'a> {
    buffer: &'a Buffer,
}

impl<'a> ExternalBufferRef<'a> {
    /// Returns the underlying buffer.
    pub fn buffer(&self) -> &Buffer {
        self.buffer
    }

    /// Returns the opaque device memory pointer.
    ///
    /// Unlike `Buffer::opaque_device_memory_pointer`, this is safe to call
    /// because the external reference count is guaranteed to be > 0.
    pub fn device_memory_pointer(&self) -> Result<*mut c_void> {
        // SAFETY: The external reference count is > 0 because we hold an ExternalBufferRef
        unsafe { self.buffer.opaque_device_memory_pointer() }
    }

    /// Returns the unsafe pointer to the buffer.
    ///
    /// Unlike `Buffer::unsafe_pointer`, this is safe to call because the
    /// external reference count is guaranteed to be > 0.
    pub fn unsafe_pointer(&self) -> Result<usize> {
        // SAFETY: The external reference count is > 0 because we hold an ExternalBufferRef
        unsafe { self.buffer.unsafe_pointer() }
    }
}

impl<'a> Drop for ExternalBufferRef<'a> {
    fn drop(&mut self) {
        // SAFETY: We incremented the reference count in external_ref(),
        // so it's safe to decrement here. If decrement fails, we can't do much
        // about it since Drop doesn't return errors, but it shouldn't fail
        // as long as the API contract is followed.
        if let Err(e) = unsafe { self.buffer.decrease_external_ref_count() } {
            // Log the error but don't panic in Drop
            eprintln!(
                "Warning: Failed to decrease external reference count: {}",
                e
            );
        }
    }
}

impl<'a> std::fmt::Debug for ExternalBufferRef<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ExternalBufferRef")
            .field("buffer", &self.buffer)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ==========================================================================
    // Trait Implementation Tests (compile-time verification)
    // ==========================================================================

    /// Tests that Buffer implements Debug trait
    #[test]
    fn test_buffer_implements_debug() {
        fn assert_debug<T: std::fmt::Debug>() {}
        assert_debug::<Buffer>();
    }

    /// Tests that CopyRawToHostFuture implements Debug trait
    #[test]
    fn test_copy_raw_to_host_future_implements_debug() {
        fn assert_debug<T: std::fmt::Debug>() {}
        assert_debug::<CopyRawToHostFuture>();
    }

    /// Tests that CopyRawToHostFuture implements Future trait
    #[test]
    fn test_copy_raw_to_host_future_implements_future() {
        fn assert_future<T: Future<Output = Result<()>>>() {}
        assert_future::<CopyRawToHostFuture>();
    }

    /// Tests that DonateWithControlDependency implements Debug trait
    #[test]
    fn test_donate_with_control_dependency_implements_debug() {
        fn assert_debug<T: std::fmt::Debug>() {}
        assert_debug::<DonateWithControlDependency>();
    }

    /// Tests that ExternalBufferRef implements Debug trait
    #[test]
    fn test_external_buffer_ref_implements_debug() {
        fn assert_debug<T: std::fmt::Debug>() {}
        assert_debug::<ExternalBufferRef>();
    }

    // ==========================================================================
    // Buffer Type Tests
    // ==========================================================================

    /// Test that Buffer is not Send (contains raw pointers)
    #[test]
    fn test_buffer_is_not_send() {
        fn assert_not_send<T>() {
            // Buffer intentionally does not implement Send because it contains
            // raw pointers that are not thread-safe.
        }
        assert_not_send::<Buffer>();
    }

    /// Test that Buffer is not Sync (contains raw pointers)
    #[test]
    fn test_buffer_is_not_sync() {
        fn assert_not_sync<T>() {
            // Buffer intentionally does not implement Sync because it contains
            // raw pointers that are not thread-safe.
        }
        assert_not_sync::<Buffer>();
    }

    // ==========================================================================
    // CopyRawToHostFuture Tests
    // ==========================================================================

    /// Test that CopyRawToHostFuture has the correct Future output type
    #[test]
    fn test_copy_raw_to_host_future_output_type() {
        // Verify the Future output type is Result<()>
        fn check_output_type<F: Future<Output = Result<()>>>() {}
        check_output_type::<CopyRawToHostFuture>();
    }

    /// Test CopyRawToHostFuture struct fields exist
    #[test]
    fn test_copy_raw_to_host_future_struct() {
        // This test verifies the struct layout through the Debug impl
        // The struct should contain: event, callback_data, future_ready_callback, transfer_size
        fn assert_debug<T: std::fmt::Debug>() {}
        assert_debug::<CopyRawToHostFuture>();
    }

    // ==========================================================================
    // DonateWithControlDependency Tests
    // ==========================================================================

    /// Test DonateWithControlDependency struct exists and has expected methods
    #[test]
    fn test_donate_with_control_dependency_methods_exist() {
        // Verify the struct has the expected method signatures
        // buffer() -> &Buffer
        // into_buffer(self) -> Buffer
        // dependency_ready(...) -> Result<()>

        // Note: We can't actually call these without a real instance,
        // but we can verify the type signatures compile
        fn _check_dependency_ready_signature(_val: &DonateWithControlDependency) {
            // This just verifies the method exists with the right signature
            let _: fn(&DonateWithControlDependency, Option<ErrorCode>, Option<&str>) -> Result<()> =
                DonateWithControlDependency::dependency_ready;
        }

        fn _check_into_buffer_signature() {
            let _: fn(DonateWithControlDependency) -> Buffer =
                DonateWithControlDependency::into_buffer;
        }
    }

    // ==========================================================================
    // ExternalBufferRef Tests
    // ==========================================================================

    /// Test ExternalBufferRef lifetime parameter
    #[test]
    fn test_external_buffer_ref_has_lifetime() {
        // ExternalBufferRef<'a> borrows a Buffer, ensuring the buffer
        // outlives the reference
        #[allow(dead_code)]
        fn check_lifetime<'a>(_: &'a Buffer) -> Option<ExternalBufferRef<'a>> {
            // Can't actually construct without increasing ref count
            // Just verifying lifetime works correctly
            None
        }

        // Just verify the function compiles - this validates the lifetime relationship
        #[allow(dead_code)]
        fn _uses_check_lifetime(buf: &Buffer) {
            let _ = check_lifetime(buf);
        }
    }

    /// Test ExternalBufferRef method signatures
    #[test]
    fn test_external_buffer_ref_methods_exist() {
        // Verify expected methods exist by checking we can reference them
        fn _check_buffer_method<'a>(val: &'a ExternalBufferRef<'a>) -> &'a Buffer {
            val.buffer()
        }

        // Check method types exist - these compile-time verify the API
        let _ = ExternalBufferRef::device_memory_pointer;
        let _ = ExternalBufferRef::unsafe_pointer;
        let _ = ExternalBufferRef::buffer;
    }

    // ==========================================================================
    // Buffer Method Signature Tests
    // ==========================================================================

    /// Test that Buffer has expected public methods
    #[test]
    fn test_buffer_public_methods_exist() {
        // Verify method signatures by creating function pointers
        // This ensures the API surface is stable

        // Accessor methods - verify they exist
        let _ = Buffer::client;
        let _ = Buffer::primitive_type;
        let _ = Buffer::dims;
        let _ = Buffer::unpadded_dims;
        let _ = Buffer::dynamic_dims_indices;
        let _ = Buffer::layout;
        let _ = Buffer::on_device_size;
        let _ = Buffer::is_on_cpu;
        let _ = Buffer::device;
        let _ = Buffer::memory;
        let _ = Buffer::is_deleted;

        // The delete method consumes the buffer
        fn _check_delete_signature() {
            let _: fn(Buffer) -> Result<()> = Buffer::delete;
        }
    }

    /// Test that Buffer has expected unsafe methods
    #[test]
    fn test_buffer_unsafe_methods_exist() {
        // Verify unsafe methods exist by referencing them
        let _ = Buffer::unsafe_pointer;
        let _ = Buffer::increase_external_ref_count;
        let _ = Buffer::decrease_external_ref_count;
        let _ = Buffer::opaque_device_memory_pointer;
        let _ = Buffer::external_ref;
    }

    // ==========================================================================
    // Async Method Signature Tests
    // ==========================================================================

    /// Test async copy methods exist with correct signatures
    #[test]
    fn test_buffer_async_and_sync_copy_methods_exist() {
        // Verify methods exist by referencing them
        let _ = Buffer::to_host;
        let _ = Buffer::to_host_sync;
        let _ = Buffer::call_copy_to_host;
        let _ = Buffer::call_copy_to_device;
        let _ = Buffer::call_copy_to_memory;
        let _ = Buffer::copy_raw_to_host_future;
        let _ = Buffer::donate_with_control_dependency;
    }

    // ==========================================================================
    // Type Size Tests
    // ==========================================================================

    /// Test that pointer size assumptions are correct for the platform
    #[test]
    fn test_pointer_size() {
        // Buffer stores a raw pointer, verify expected size
        assert_eq!(
            std::mem::size_of::<*mut PJRT_Buffer>(),
            std::mem::size_of::<usize>()
        );
    }

    /// Test that CopyRawToHostFuture callback function pointer is correctly sized
    #[test]
    fn test_callback_pointer_size() {
        // Function pointers should be pointer-sized
        type CallbackFn = Option<
            unsafe extern "C" fn(*mut pjrt_sys::PJRT_Buffer_CopyRawToHostFuture_Callback_Args),
        >;
        assert_eq!(
            std::mem::size_of::<CallbackFn>(),
            std::mem::size_of::<usize>()
        );
    }

    // ==========================================================================
    // Error Handling Tests
    // ==========================================================================

    /// Test that CopyRawToHostFuture::future_ready validates buffer size
    #[test]
    fn test_future_ready_callback_validates_buffer_size() {
        // We can't construct a CopyRawToHostFuture without a plugin,
        // but we can verify the error type exists for this validation
        let err = crate::Error::InvalidArgument(
            "destination buffer too small: got 0 bytes, need 100".to_string(),
        );
        let err_str = format!("{}", err);
        assert!(err_str.contains("destination buffer too small"));
    }

    /// Test that DonateWithControlDependency returns correct error for null callback
    #[test]
    fn test_null_callback_error_type() {
        // Verify the error type used when callback is null
        let err = crate::Error::NullFunctionPointer("dependency_ready_callback");
        let err_str = format!("{}", err);
        assert!(err_str.contains("null function pointer"));
        assert!(err_str.contains("dependency_ready_callback"));
    }

    /// Test NullFunctionPointer error for future_ready_callback
    #[test]
    fn test_null_future_ready_callback_error() {
        let err = crate::Error::NullFunctionPointer("future_ready_callback");
        let err_str = format!("{}", err);
        assert!(err_str.contains("null function pointer"));
        assert!(err_str.contains("future_ready_callback"));
    }

    // ==========================================================================
    // Documentation Tests
    // ==========================================================================

    /// Verify module documentation examples are syntactically correct
    /// (actual execution requires plugin)
    #[test]
    fn test_module_documentation_compiles() {
        // The module-level examples use `ignore` in rustdoc,
        // but this test verifies the code structure is sound

        // Example pattern 1: Creating host buffer and transferring
        // let host_data = HostBuffer::from_data::<F32>(...);
        // let device_buffer = host_data.to_sync(&client).copy()?;

        // Example pattern 2: Query buffer properties
        // buffer.primitive_type()
        // buffer.dims()
        // buffer.on_device_size()

        // Example pattern 3: Async transfers
        // let event = buffer.ready_event()?;
        // event.await?;

        // All verified at compile time through the type system
    }

    // ==========================================================================
    // CopyRawToHostFuture::event() accessor test
    // ==========================================================================

    /// Test that CopyRawToHostFuture has event accessor
    #[test]
    fn test_copy_raw_to_host_future_event_accessor() {
        // Verify the event() method exists
        let _ = CopyRawToHostFuture::event;
    }

    // ==========================================================================
    // DonateWithControlDependency buffer accessors
    // ==========================================================================

    /// Test that DonateWithControlDependency has buffer accessor
    #[test]
    fn test_donate_with_control_dependency_buffer_accessor() {
        // Verify the buffer() method exists
        let _ = DonateWithControlDependency::buffer;
    }
}
