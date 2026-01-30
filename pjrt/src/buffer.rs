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

    pub fn primitive_type(&self) -> PrimitiveType {
        let mut args = PJRT_Buffer_ElementType_Args::new();
        args.buffer = self.ptr;
        args = self
            .client
            .api()
            .PJRT_Buffer_ElementType(args)
            .expect("PJRT_Buffer_ElementType");
        PrimitiveType::try_from(args.type_).expect("PrimitiveType")
    }

    pub fn dims(&self) -> Vec<i64> {
        let mut args = PJRT_Buffer_Dimensions_Args::new();
        args.buffer = self.ptr;
        args = self
            .client
            .api()
            .PJRT_Buffer_Dimensions(args)
            .expect("PJRT_Buffer_Dimensions");
        if args.num_dims == 0 {
            return vec![];
        }
        let s = unsafe { std::slice::from_raw_parts(args.dims, args.num_dims) };
        s.to_owned()
    }

    pub fn unpadded_dims(&self) -> Vec<i64> {
        let mut args = PJRT_Buffer_UnpaddedDimensions_Args::new();
        args.buffer = self.ptr;
        args = self
            .client
            .api()
            .PJRT_Buffer_UnpaddedDimensions(args)
            .expect("PJRT_Buffer_UnpaddedDimensions");
        let s = unsafe { std::slice::from_raw_parts(args.unpadded_dims, args.num_dims) };
        s.to_owned()
    }

    pub fn dynamic_dims_indices(&self) -> Vec<usize> {
        let mut args = PJRT_Buffer_DynamicDimensionIndices_Args::new();
        args.buffer = self.ptr;
        args = self
            .client
            .api()
            .PJRT_Buffer_DynamicDimensionIndices(args)
            .expect("PJRT_Buffer_DynamicDimensionIndices");
        let s =
            unsafe { std::slice::from_raw_parts(args.dynamic_dim_indices, args.num_dynamic_dims) };
        s.to_owned()
    }

    pub fn layout(&self) -> MemoryLayout {
        let mut args = PJRT_Buffer_GetMemoryLayout_Args::new();
        args.buffer = self.ptr;
        args = self
            .client
            .api()
            .PJRT_Buffer_GetMemoryLayout(args)
            .expect("PJRT_Buffer_GetMemoryLayout");
        MemoryLayout::try_from(&args.layout).expect("layout")
    }

    pub fn on_device_size(&self) -> usize {
        let mut args = PJRT_Buffer_OnDeviceSizeInBytes_Args::new();
        args.buffer = self.ptr;
        args = self
            .client
            .api()
            .PJRT_Buffer_OnDeviceSizeInBytes(args)
            .expect("PJRT_Buffer_GetMemoryLayout");
        args.on_device_size_in_bytes
    }

    pub fn is_on_cpu(&self) -> bool {
        let mut args = PJRT_Buffer_IsOnCpu_Args::new();
        args.buffer = self.ptr;
        args = self
            .client
            .api()
            .PJRT_Buffer_IsOnCpu(args)
            .expect("PJRT_Buffer_IsOnCpu");
        args.is_on_cpu
    }

    pub fn device(&self) -> Device {
        let mut args = PJRT_Buffer_Device_Args::new();
        args.buffer = self.ptr;
        args = self
            .client
            .api()
            .PJRT_Buffer_Device(args)
            .expect("PJRT_Buffer_Device");
        Device::wrap(&self.client, args.device)
    }

    pub fn memory(&self) -> Memory {
        let mut args = PJRT_Buffer_Memory_Args::new();
        args.buffer = self.ptr;
        args = self
            .client
            .api()
            .PJRT_Buffer_Memory(args)
            .expect("PJRT_Buffer_Memory");
        Memory::wrap(&self.client, args.memory)
    }

    pub fn delete(self) {
        let mut args = PJRT_Buffer_Delete_Args::new();
        args.buffer = self.ptr;
        self.client
            .api()
            .PJRT_Buffer_Delete(args)
            .expect("PJRT_Buffer_Delete");
    }

    pub fn is_deleted(&self) -> bool {
        let mut args = PJRT_Buffer_IsDeleted_Args::new();
        args.buffer = self.ptr;
        args = self
            .client
            .api()
            .PJRT_Buffer_IsDeleted(args)
            .expect("PJRT_Buffer_IsDeleted");
        args.is_deleted
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
        let ty = self.primitive_type();
        let dims = self.dims();
        let layout = host_layout.unwrap_or_else(|| self.layout());
        HostBuffer::from_bytes(data, ty, Some(dims), Some(layout))
    }

    pub fn to_host_sync(&self, host_layout: Option<MemoryLayout>) -> Result<HostBuffer> {
        let (args, data) = self.call_copy_to_host(host_layout.as_ref())?;
        let event = Event::wrap(self.client.api(), args.event);
        event.wait()?;
        let ty = self.primitive_type();
        let dims = self.dims();
        let layout = host_layout.unwrap_or_else(|| self.layout());
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
    ///
    /// # Example
    /// ```
    /// let (future, callback_data, future_ready) = buffer.copy_raw_to_host_future(0, 1024)?;
    /// // Later, when ready to receive:
    /// let mut dst = vec![0u8; 1024];
    /// future_ready(&dst, None, None)?;
    /// future.await?;
    /// ```
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
