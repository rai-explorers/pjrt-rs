//! Asynchronous Host-to-Device Transfers
//!
//! This module provides the [`AsyncHostToDeviceTransferManager`] for managing
//! asynchronous data transfers from host to device memory. This enables overlapping
//! data transfers with computation, improving performance for large workloads.
//!
//! # Overview
//!
//! PJRT supports streaming data from host memory to device memory asynchronously.
//! This is crucial for:
//!
//! - **Large datasets**: Transfer data that doesn't fit in device memory
//! - **Pipelining**: Overlap data transfer with computation
//! - **Streaming**: Process data from disk/network in real-time
//! - **Responsiveness**: Non-blocking I/O for interactive applications
//!
//! # API Levels
//!
//! This module provides three levels of abstraction:
//!
//! ## 1. High-Level Builder API (Recommended)
//!
//! Use [`AsyncTransferBuilder`] for simple, one-shot transfers:
//!
//! ```rust,ignore
//! use pjrt::{AsyncTransferBuilder, F32};
//!
//! // Transfer typed data with automatic shape inference
//! let buffer = AsyncTransferBuilder::new(&client, &device)
//!     .typed::<F32>(&data, &[100, 100])
//!     .transfer()
//!     .await?;
//! ```
//!
//! ## 2. Mid-Level Convenience Methods
//!
//! Use methods on [`AsyncHostToDeviceTransferManager`] for common patterns:
//!
//! ```rust,ignore
//! // Transfer all data at once
//! manager.transfer_all(0, &bytes).await?;
//!
//! // Transfer typed data
//! manager.transfer_typed::<F32>(0, &data, &dims).await?;
//!
//! // Transfer with progress tracking
//! manager.transfer_chunked(0, &data, chunk_size, |done, total| {
//!     println!("{:.1}%", 100.0 * done as f64 / total as f64);
//! }).await?;
//! ```
//!
//! ## 3. Low-Level Control
//!
//! Use [`transfer_data`][AsyncHostToDeviceTransferManager::transfer_data] for full control:
//!
//! ```rust,ignore
//! // Manual chunk management
//! for (i, chunk) in data.chunks(chunk_size).enumerate() {
//!     let offset = i * chunk_size;
//!     let is_last = offset + chunk.len() >= total;
//!     
//!     manager.transfer_data(0)
//!         .data(chunk)
//!         .offset(offset as i64)
//!         .is_last_transfer(is_last)
//!         .transfer()
//!         .await?;
//! }
//! ```
//!
//! # Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────────┐
//! │                     Host Application                            │
//! ├─────────────────────────────────────────────────────────────────┤
//! │  AsyncTransferBuilder                                           │
//! │    └── Creates manager, transfers data, returns buffer          │
//! ├─────────────────────────────────────────────────────────────────┤
//! │  AsyncHostToDeviceTransferManager                               │
//! │    ├── transfer_all() / transfer_typed() / transfer_chunked()   │
//! │    └── transfer_data() / transfer_literal() (low-level)         │
//! ├─────────────────────────────────────────────────────────────────┤
//! │                      PJRT C API                                 │
//! ├─────────────────────────────────────────────────────────────────┤
//! │                   Device (CPU/GPU/TPU)                          │
//! └─────────────────────────────────────────────────────────────────┘
//! ```
//!
//! # Examples
//!
//! ## Simple Transfer with Builder
//!
//! ```rust,ignore
//! use pjrt::{AsyncTransferBuilder, Client, F32, Result};
//!
//! async fn simple_transfer(client: &Client) -> Result<()> {
//!     let device = client.addressable_devices().first().unwrap();
//!     let data: Vec<f32> = vec![1.0, 2.0, 3.0, 4.0];
//!
//!     // One-liner transfer
//!     let buffer = AsyncTransferBuilder::new(client, device)
//!         .typed::<F32>(&data, &[2, 2])
//!         .transfer()
//!         .await?;
//!
//!     println!("Buffer on device: {:?}", buffer.device().id());
//!     Ok(())
//! }
//! ```
//!
//! ## Multi-Buffer Transfer
//!
//! ```rust,ignore
//! use pjrt::{BufferShape, Client, PrimitiveType, Result};
//!
//! async fn multi_buffer_transfer(client: &Client) -> Result<()> {
//!     let device = client.addressable_devices().first().unwrap();
//!     let memory = device.default_memory()?;
//!
//!     // Define shapes for multiple buffers
//!     let shapes = vec![
//!         BufferShape::new(vec![100, 100], PrimitiveType::F32),
//!         BufferShape::new(vec![50], PrimitiveType::I32),
//!     ];
//!
//!     // Create transfer manager
//!     let manager = client.create_buffers_for_async_host_to_device(&shapes, &memory)?;
//!
//!     // Transfer data to each buffer
//!     let data1: Vec<f32> = vec![0.0; 10000];
//!     let data2: Vec<i32> = vec![1; 50];
//!
//!     manager.transfer_typed::<pjrt::F32>(0, &data1, &[100, 100]).await?;
//!     manager.transfer_typed::<pjrt::I32>(1, &data2, &[50]).await?;
//!
//!     // Retrieve all buffers
//!     let buffers = manager.retrieve_all_buffers()?;
//!     println!("Created {} buffers", buffers.len());
//!
//!     Ok(())
//! }
//! ```
//!
//! ## Streaming Large Data with Progress
//!
//! ```rust,ignore
//! use pjrt::{BufferShape, Client, PrimitiveType, Result};
//!
//! async fn streaming_transfer(client: &Client, large_data: &[u8]) -> Result<()> {
//!     let device = client.addressable_devices().first().unwrap();
//!     let memory = device.default_memory()?;
//!
//!     let num_elements = large_data.len() / std::mem::size_of::<f32>();
//!     let shapes = vec![BufferShape::new(vec![num_elements as i64], PrimitiveType::F32)];
//!
//!     let manager = client.create_buffers_for_async_host_to_device(&shapes, &memory)?;
//!
//!     // Transfer with progress tracking
//!     manager.transfer_chunked(
//!         0,
//!         large_data,
//!         1024 * 1024, // 1MB chunks
//!         |transferred, total| {
//!             println!("Progress: {:.1}%", 100.0 * transferred as f64 / total as f64);
//!         },
//!     ).await?;
//!
//!     let buffer = manager.retrieve_buffer(0)?;
//!     println!("Transfer complete: {} bytes", buffer.on_device_size_in_bytes()?);
//!
//!     Ok(())
//! }
//! ```

use std::ffi::c_void;
use std::marker::PhantomData;

use bon::bon;
use pjrt_sys::{
    PJRT_AsyncHostToDeviceTransferManager, PJRT_AsyncHostToDeviceTransferManager_AddMetadata_Args,
    PJRT_AsyncHostToDeviceTransferManager_BufferCount_Args,
    PJRT_AsyncHostToDeviceTransferManager_BufferSize_Args,
    PJRT_AsyncHostToDeviceTransferManager_Destroy_Args,
    PJRT_AsyncHostToDeviceTransferManager_Device_Args,
    PJRT_AsyncHostToDeviceTransferManager_RetrieveBuffer_Args,
    PJRT_AsyncHostToDeviceTransferManager_SetBufferError_Args,
    PJRT_AsyncHostToDeviceTransferManager_TransferData_Args,
    PJRT_AsyncHostToDeviceTransferManager_TransferLiteral_Args, PJRT_Buffer_Type, PJRT_ShapeSpec,
};

use crate::{
    Buffer, Client, Device, ErrorCode, Event, Memory, MemoryLayout, NamedValue, PrimitiveType,
    Result,
};

/// Manages asynchronous transfers from host to device memory.
///
/// This provides a way to transfer data to the device asynchronously,
/// allowing overlapping of data transfer with computation.
pub struct AsyncHostToDeviceTransferManager {
    client: Client,
    ptr: *mut PJRT_AsyncHostToDeviceTransferManager,
}

impl Drop for AsyncHostToDeviceTransferManager {
    fn drop(&mut self) {
        let mut args = PJRT_AsyncHostToDeviceTransferManager_Destroy_Args::new();
        args.transfer_manager = self.ptr;
        self.client
            .api()
            .PJRT_AsyncHostToDeviceTransferManager_Destroy(args)
            .expect("PJRT_AsyncHostToDeviceTransferManager_Destroy");
    }
}

impl std::fmt::Debug for AsyncHostToDeviceTransferManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AsyncHostToDeviceTransferManager")
            .field("ptr", &self.ptr)
            .finish()
    }
}

#[bon]
impl AsyncHostToDeviceTransferManager {
    pub(crate) fn wrap(client: &Client, ptr: *mut PJRT_AsyncHostToDeviceTransferManager) -> Self {
        assert!(!ptr.is_null());
        Self {
            client: client.clone(),
            ptr,
        }
    }

    pub fn client(&self) -> &Client {
        &self.client
    }

    #[allow(dead_code)]
    pub(crate) fn ptr(&self) -> *mut PJRT_AsyncHostToDeviceTransferManager {
        self.ptr
    }

    /// Returns the device associated with this transfer manager.
    pub fn device(&self) -> Result<Device> {
        let mut args = PJRT_AsyncHostToDeviceTransferManager_Device_Args::new();
        args.transfer_manager = self.ptr;
        args = self
            .client
            .api()
            .PJRT_AsyncHostToDeviceTransferManager_Device(args)?;
        Ok(Device::wrap(&self.client, args.device_out))
    }

    /// Returns the number of buffers managed by this transfer manager.
    pub fn buffer_count(&self) -> Result<usize> {
        let mut args = PJRT_AsyncHostToDeviceTransferManager_BufferCount_Args::new();
        args.transfer_manager = self.ptr;
        args = self
            .client
            .api()
            .PJRT_AsyncHostToDeviceTransferManager_BufferCount(args)?;
        Ok(args.buffer_count)
    }

    /// Returns the size (in bytes) of the buffer at the given index.
    pub fn buffer_size(&self, buffer_index: i32) -> Result<usize> {
        let mut args = PJRT_AsyncHostToDeviceTransferManager_BufferSize_Args::new();
        args.transfer_manager = self.ptr;
        args.buffer_index = buffer_index;
        args = self
            .client
            .api()
            .PJRT_AsyncHostToDeviceTransferManager_BufferSize(args)?;
        Ok(args.buffer_size)
    }

    /// Transfers data to the buffer at the given index.
    ///
    /// # Arguments
    /// * `buffer_index` - The index of the buffer to transfer to
    /// * `data` - The data to transfer
    /// * `offset` - The offset within the buffer to start writing
    /// * `is_last_transfer` - Whether this is the last transfer for this buffer
    ///
    /// Returns an event that is triggered when the transfer is complete.
    #[builder(finish_fn = transfer)]
    pub fn transfer_data(
        &self,
        #[builder(start_fn)] buffer_index: i32,
        data: &[u8],
        #[builder(default = 0)] offset: i64,
        #[builder(default = false)] is_last_transfer: bool,
    ) -> Result<Event> {
        let mut args = PJRT_AsyncHostToDeviceTransferManager_TransferData_Args::new();
        args.transfer_manager = self.ptr;
        args.buffer_index = buffer_index;
        args.data = data.as_ptr() as *const c_void;
        args.offset = offset;
        args.transfer_size = data.len() as i64;
        args.is_last_transfer = is_last_transfer;
        let args = self
            .client
            .api()
            .PJRT_AsyncHostToDeviceTransferManager_TransferData(args)?;
        Ok(Event::wrap(self.client.api(), args.done_with_h2d_transfer))
    }

    /// Transfers a literal to the buffer at the given index.
    ///
    /// This is useful for transferring data with a specific shape and type.
    #[builder(finish_fn = transfer_literal)]
    pub fn transfer_literal<T: crate::Type>(
        &self,
        #[builder(start_fn)] buffer_index: i32,
        data: &[T::ElemType],
        dims: &[i64],
        layout: Option<&MemoryLayout>,
    ) -> Result<Event> {
        let mut args = PJRT_AsyncHostToDeviceTransferManager_TransferLiteral_Args::new();
        args.transfer_manager = self.ptr;
        args.buffer_index = buffer_index;
        args.data = data.as_ptr() as *const c_void;
        args.shape_dims = dims.as_ptr();
        args.shape_num_dims = dims.len();
        args.shape_element_type = T::PRIMITIVE_TYPE as PJRT_Buffer_Type;

        let mut layout_c = layout.map(pjrt_sys::PJRT_Buffer_MemoryLayout::from);
        if let Some(ref mut l) = layout_c {
            args.shape_layout = l as *mut _;
        }

        let args = self
            .client
            .api()
            .PJRT_AsyncHostToDeviceTransferManager_TransferLiteral(args)?;
        Ok(Event::wrap(self.client.api(), args.done_with_h2d_transfer))
    }

    /// Retrieves the buffer at the given index.
    ///
    /// This should be called after all transfers for the buffer are complete.
    pub fn retrieve_buffer(&self, buffer_index: i32) -> Result<Buffer> {
        let mut args = PJRT_AsyncHostToDeviceTransferManager_RetrieveBuffer_Args::new();
        args.transfer_manager = self.ptr;
        args.buffer_index = buffer_index;
        let args = self
            .client
            .api()
            .PJRT_AsyncHostToDeviceTransferManager_RetrieveBuffer(args)?;
        Ok(Buffer::wrap(&self.client, args.buffer_out))
    }

    /// Sets an error for the buffer at the given index.
    ///
    /// This is useful for signaling that a transfer failed.
    pub fn set_buffer_error(
        &self,
        buffer_index: i32,
        error_code: ErrorCode,
        message: &str,
    ) -> Result<()> {
        let mut args = PJRT_AsyncHostToDeviceTransferManager_SetBufferError_Args::new();
        args.transfer_manager = self.ptr;
        args.buffer_index = buffer_index;
        args.error_code = error_code as pjrt_sys::PJRT_Error_Code;
        args.error_message = message.as_ptr() as *const i8;
        args.error_message_size = message.len();
        self.client
            .api()
            .PJRT_AsyncHostToDeviceTransferManager_SetBufferError(args)
            .map(|_| ())
    }

    /// Adds metadata to this transfer manager.
    pub fn add_metadata(&self, metadata: &[NamedValue]) -> Result<()> {
        let mut args = PJRT_AsyncHostToDeviceTransferManager_AddMetadata_Args::new();
        args.transfer_manager = self.ptr;
        let metadata_c: Vec<pjrt_sys::PJRT_NamedValue> = metadata
            .iter()
            .map(pjrt_sys::PJRT_NamedValue::from)
            .collect();
        args.transfer_metadata = metadata_c.as_ptr();
        args.num_metadata = metadata_c.len();
        self.client
            .api()
            .PJRT_AsyncHostToDeviceTransferManager_AddMetadata(args)
            .map(|_| ())
    }

    // ==================== High-Level Convenience Methods ====================

    /// Transfers all data to a buffer in a single operation.
    ///
    /// This is a convenience method that wraps the low-level `transfer_data` API.
    /// It transfers the entire data slice to the buffer at once, marking it as
    /// the last transfer.
    ///
    /// # Arguments
    ///
    /// * `buffer_index` - The index of the buffer to transfer to
    /// * `data` - The data to transfer (as raw bytes)
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let data: Vec<f32> = vec![1.0, 2.0, 3.0, 4.0];
    /// let bytes = bytemuck::cast_slice(&data);
    /// manager.transfer_all(0, bytes).await?;
    /// ```
    pub async fn transfer_all(&self, buffer_index: i32, data: &[u8]) -> Result<()> {
        let event = self
            .transfer_data(buffer_index)
            .data(data)
            .is_last_transfer(true)
            .transfer()?;
        event.await
    }

    /// Transfers all data to a buffer synchronously.
    ///
    /// This is a convenience method that wraps the low-level `transfer_data` API.
    /// It blocks until the transfer is complete.
    ///
    /// # Arguments
    ///
    /// * `buffer_index` - The index of the buffer to transfer to
    /// * `data` - The data to transfer (as raw bytes)
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let data: Vec<f32> = vec![1.0, 2.0, 3.0, 4.0];
    /// let bytes = bytemuck::cast_slice(&data);
    /// manager.transfer_all_sync(0, bytes)?;
    /// ```
    pub fn transfer_all_sync(&self, buffer_index: i32, data: &[u8]) -> Result<()> {
        let event = self
            .transfer_data(buffer_index)
            .data(data)
            .is_last_transfer(true)
            .transfer()?;
        event.wait()
    }

    /// Transfers typed data to a buffer.
    ///
    /// This is a high-level method that handles type conversion and transfers
    /// the entire typed slice to the buffer.
    ///
    /// # Type Parameters
    ///
    /// * `T` - A PJRT type (e.g., `F32`, `I32`, etc.)
    ///
    /// # Arguments
    ///
    /// * `buffer_index` - The index of the buffer to transfer to
    /// * `data` - The typed data to transfer
    /// * `dims` - The dimensions of the data
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use pjrt::F32;
    ///
    /// let data: Vec<f32> = vec![1.0, 2.0, 3.0, 4.0];
    /// let dims = vec![2, 2];
    /// manager.transfer_typed::<F32>(0, &data, &dims).await?;
    /// ```
    pub async fn transfer_typed<T: crate::Type>(
        &self,
        buffer_index: i32,
        data: &[T::ElemType],
        dims: &[i64],
    ) -> Result<()> {
        let event = self
            .transfer_literal::<T>(buffer_index)
            .data(data)
            .dims(dims)
            .transfer_literal()?;
        event.await
    }

    /// Transfers typed data to a buffer synchronously.
    ///
    /// This is a high-level method that handles type conversion and transfers
    /// the entire typed slice to the buffer, blocking until complete.
    ///
    /// # Type Parameters
    ///
    /// * `T` - A PJRT type (e.g., `F32`, `I32`, etc.)
    ///
    /// # Arguments
    ///
    /// * `buffer_index` - The index of the buffer to transfer to
    /// * `data` - The typed data to transfer
    /// * `dims` - The dimensions of the data
    pub fn transfer_typed_sync<T: crate::Type>(
        &self,
        buffer_index: i32,
        data: &[T::ElemType],
        dims: &[i64],
    ) -> Result<()> {
        let event = self
            .transfer_literal::<T>(buffer_index)
            .data(data)
            .dims(dims)
            .transfer_literal()?;
        event.wait()
    }

    /// Transfers data in chunks with a callback for progress tracking.
    ///
    /// This method is useful for large transfers where you want to track
    /// progress or perform other operations between chunks.
    ///
    /// # Arguments
    ///
    /// * `buffer_index` - The index of the buffer to transfer to
    /// * `data` - The complete data to transfer
    /// * `chunk_size` - Size of each chunk in bytes
    /// * `on_progress` - Callback called after each chunk with (bytes_transferred, total_bytes)
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// manager.transfer_chunked(
    ///     0,
    ///     &large_data,
    ///     1024 * 1024, // 1MB chunks
    ///     |transferred, total| {
    ///         println!("Progress: {:.1}%", 100.0 * transferred as f64 / total as f64);
    ///     },
    /// ).await?;
    /// ```
    pub async fn transfer_chunked<F>(
        &self,
        buffer_index: i32,
        data: &[u8],
        chunk_size: usize,
        mut on_progress: F,
    ) -> Result<()>
    where
        F: FnMut(usize, usize),
    {
        let total = data.len();
        let mut transferred = 0;

        for chunk in data.chunks(chunk_size) {
            let is_last = transferred + chunk.len() >= total;

            let event = self
                .transfer_data(buffer_index)
                .data(chunk)
                .offset(transferred as i64)
                .is_last_transfer(is_last)
                .transfer()?;

            event.await?;
            transferred += chunk.len();
            on_progress(transferred, total);
        }

        Ok(())
    }

    /// Retrieves all buffers managed by this transfer manager.
    ///
    /// This is a convenience method that retrieves all buffers at once.
    /// Should only be called after all transfers are complete.
    ///
    /// # Returns
    ///
    /// A vector of all buffers managed by this transfer manager.
    pub fn retrieve_all_buffers(&self) -> Result<Vec<Buffer>> {
        let count = self.buffer_count()?;
        let mut buffers = Vec::with_capacity(count);
        for i in 0..count {
            buffers.push(self.retrieve_buffer(i as i32)?);
        }
        Ok(buffers)
    }
}

/// Specifies the shape of a buffer to be created.
pub struct BufferShape {
    dims: Vec<i64>,
    element_type: PrimitiveType,
    layout: Option<MemoryLayout>,
}

impl std::fmt::Debug for BufferShape {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BufferShape")
            .field("dims", &self.dims)
            .field("element_type", &self.element_type)
            .field("layout", &self.layout)
            .finish()
    }
}

impl BufferShape {
    pub fn new(dims: Vec<i64>, element_type: PrimitiveType) -> Self {
        Self {
            dims,
            element_type,
            layout: None,
        }
    }

    pub fn with_layout(mut self, layout: MemoryLayout) -> Self {
        self.layout = Some(layout);
        self
    }

    pub(crate) fn to_spec(&self) -> PJRT_ShapeSpec {
        let mut spec = PJRT_ShapeSpec::new();
        spec.dims = self.dims.as_ptr();
        spec.num_dims = self.dims.len();
        spec.element_type = self.element_type as PJRT_Buffer_Type;
        spec
    }

    pub fn dims(&self) -> &[i64] {
        &self.dims
    }

    pub fn element_type(&self) -> PrimitiveType {
        self.element_type
    }

    pub fn layout(&self) -> Option<&MemoryLayout> {
        self.layout.as_ref()
    }
}

// =============================================================================
// High-Level Async Transfer API
// =============================================================================

/// A high-level builder for simple asynchronous host-to-device transfers.
///
/// `AsyncTransferBuilder` provides an ergonomic API for the common case of
/// transferring a single buffer to a device. It handles buffer creation,
/// transfer management, and cleanup automatically.
///
/// # Features
///
/// - **Simple API**: One-liner transfers for common cases
/// - **Type-safe**: Compile-time type checking with generics
/// - **Automatic cleanup**: RAII-based resource management
/// - **Flexible**: Supports raw bytes, typed data, or custom layouts
///
/// # Basic Usage
///
/// ```rust,ignore
/// use pjrt::{AsyncTransferBuilder, Client, F32, Result};
///
/// async fn transfer_data(client: &Client) -> Result<pjrt::Buffer> {
///     let device = client.addressable_devices().first().unwrap();
///     let data: Vec<f32> = vec![1.0, 2.0, 3.0, 4.0];
///
///     AsyncTransferBuilder::new(client, device)
///         .typed::<F32>(&data, &[2, 2])
///         .transfer()
///         .await
/// }
/// ```
///
/// # Advanced Usage with Custom Layout
///
/// ```rust,ignore
/// use pjrt::{AsyncTransferBuilder, Client, F32, MemoryLayout, Result};
///
/// async fn transfer_with_layout(client: &Client) -> Result<pjrt::Buffer> {
///     let device = client.addressable_devices().first().unwrap();
///     let data: Vec<f32> = vec![1.0, 2.0, 3.0, 4.0];
///     let layout = MemoryLayout::from_strides(vec![8, 4]); // Custom strides
///
///     AsyncTransferBuilder::new(client, device)
///         .typed::<F32>(&data, &[2, 2])
///         .layout(layout)
///         .transfer()
///         .await
/// }
/// ```
///
/// # Sync Transfer
///
/// For blocking transfers, use `transfer_sync()`:
///
/// ```rust,ignore
/// let buffer = AsyncTransferBuilder::new(client, device)
///     .typed::<F32>(&data, &[2, 2])
///     .transfer_sync()?;
/// ```
pub struct AsyncTransferBuilder<'a> {
    client: &'a Client,
    device: &'a Device,
    memory: Option<&'a Memory>,
}

impl<'a> AsyncTransferBuilder<'a> {
    /// Creates a new async transfer builder.
    ///
    /// # Arguments
    ///
    /// * `client` - The PJRT client to use
    /// * `device` - The target device for the transfer
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let builder = AsyncTransferBuilder::new(&client, &device);
    /// ```
    pub fn new(client: &'a Client, device: &'a Device) -> Self {
        Self {
            client,
            device,
            memory: None,
        }
    }

    /// Specifies the target memory for the transfer.
    ///
    /// If not specified, the device's default memory is used.
    ///
    /// # Arguments
    ///
    /// * `memory` - The target memory region
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let memory = device.default_memory()?;
    /// let builder = AsyncTransferBuilder::new(&client, &device)
    ///     .memory(&memory);
    /// ```
    pub fn memory(mut self, memory: &'a Memory) -> Self {
        self.memory = Some(memory);
        self
    }

    /// Configures the transfer with typed data.
    ///
    /// This method returns a [`TypedAsyncTransfer`] that can complete the transfer.
    ///
    /// # Type Parameters
    ///
    /// * `T` - A PJRT type marker (e.g., [`F32`][crate::F32], [`I32`][crate::I32])
    ///
    /// # Arguments
    ///
    /// * `data` - The typed data to transfer
    /// * `dims` - The dimensions of the tensor
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use pjrt::F32;
    ///
    /// let data: Vec<f32> = vec![1.0, 2.0, 3.0, 4.0];
    /// let buffer = AsyncTransferBuilder::new(&client, &device)
    ///     .typed::<F32>(&data, &[2, 2])
    ///     .transfer()
    ///     .await?;
    /// ```
    pub fn typed<T: crate::Type>(
        self,
        data: &'a [T::ElemType],
        dims: &'a [i64],
    ) -> TypedAsyncTransfer<'a, T> {
        TypedAsyncTransfer {
            client: self.client,
            device: self.device,
            memory: self.memory,
            data,
            dims,
            layout: None,
            _marker: PhantomData,
        }
    }

    /// Configures the transfer with raw bytes.
    ///
    /// This method returns a [`RawAsyncTransfer`] that can complete the transfer.
    /// Use this when you have raw bytes and know the element type at runtime.
    ///
    /// # Arguments
    ///
    /// * `data` - The raw bytes to transfer
    /// * `dims` - The dimensions of the tensor
    /// * `element_type` - The primitive type of the elements
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use pjrt::PrimitiveType;
    ///
    /// let bytes: Vec<u8> = /* raw f32 data */;
    /// let buffer = AsyncTransferBuilder::new(&client, &device)
    ///     .raw(&bytes, &[2, 2], PrimitiveType::F32)
    ///     .transfer()
    ///     .await?;
    /// ```
    pub fn raw(
        self,
        data: &'a [u8],
        dims: &'a [i64],
        element_type: PrimitiveType,
    ) -> RawAsyncTransfer<'a> {
        RawAsyncTransfer {
            client: self.client,
            device: self.device,
            memory: self.memory,
            data,
            dims,
            element_type,
            layout: None,
        }
    }
}

/// A typed async transfer operation ready to execute.
///
/// This struct is created by [`AsyncTransferBuilder::typed`] and holds all
/// the configuration needed to perform a type-safe async transfer.
///
/// # Example
///
/// ```rust,ignore
/// use pjrt::{AsyncTransferBuilder, F32};
///
/// let transfer = AsyncTransferBuilder::new(&client, &device)
///     .typed::<F32>(&data, &[2, 2])
///     .layout(custom_layout);
///
/// // Async transfer
/// let buffer = transfer.transfer().await?;
///
/// // Or sync transfer
/// let buffer = transfer.transfer_sync()?;
/// ```
pub struct TypedAsyncTransfer<'a, T: crate::Type> {
    client: &'a Client,
    device: &'a Device,
    memory: Option<&'a Memory>,
    data: &'a [T::ElemType],
    dims: &'a [i64],
    layout: Option<MemoryLayout>,
    _marker: PhantomData<T>,
}

impl<'a, T: crate::Type> TypedAsyncTransfer<'a, T> {
    /// Specifies a custom memory layout for the device buffer.
    ///
    /// # Arguments
    ///
    /// * `layout` - The memory layout to use on the device
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let layout = MemoryLayout::from_strides(vec![8, 4]);
    /// let transfer = builder.typed::<F32>(&data, &dims).layout(layout);
    /// ```
    pub fn layout(mut self, layout: MemoryLayout) -> Self {
        self.layout = Some(layout);
        self
    }

    /// Performs the transfer asynchronously.
    ///
    /// This method creates the transfer manager, transfers the data, and
    /// returns the resulting device buffer. All resources are cleaned up
    /// automatically.
    ///
    /// # Returns
    ///
    /// The device buffer containing the transferred data.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The device has no default memory
    /// - Buffer creation fails
    /// - The transfer fails
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let buffer = AsyncTransferBuilder::new(&client, &device)
    ///     .typed::<F32>(&data, &[2, 2])
    ///     .transfer()
    ///     .await?;
    /// ```
    pub async fn transfer(self) -> Result<Buffer> {
        let default_memory;
        let memory = match self.memory {
            Some(m) => m,
            None => {
                default_memory = self.device.default_memory()?;
                &default_memory
            }
        };

        let mut shape = BufferShape::new(self.dims.to_vec(), T::PRIMITIVE_TYPE);
        if let Some(layout) = self.layout {
            shape = shape.with_layout(layout);
        }

        let manager = self
            .client
            .create_buffers_for_async_host_to_device(&[shape], memory)?;

        manager.transfer_typed::<T>(0, self.data, self.dims).await?;

        manager.retrieve_buffer(0)
    }

    /// Performs the transfer synchronously.
    ///
    /// This method blocks until the transfer is complete.
    ///
    /// # Returns
    ///
    /// The device buffer containing the transferred data.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let buffer = AsyncTransferBuilder::new(&client, &device)
    ///     .typed::<F32>(&data, &[2, 2])
    ///     .transfer_sync()?;
    /// ```
    pub fn transfer_sync(self) -> Result<Buffer> {
        let default_memory;
        let memory = match self.memory {
            Some(m) => m,
            None => {
                default_memory = self.device.default_memory()?;
                &default_memory
            }
        };

        let mut shape = BufferShape::new(self.dims.to_vec(), T::PRIMITIVE_TYPE);
        if let Some(layout) = self.layout {
            shape = shape.with_layout(layout);
        }

        let manager = self
            .client
            .create_buffers_for_async_host_to_device(&[shape], memory)?;

        manager.transfer_typed_sync::<T>(0, self.data, self.dims)?;

        manager.retrieve_buffer(0)
    }
}

impl<T: crate::Type> std::fmt::Debug for TypedAsyncTransfer<'_, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TypedAsyncTransfer")
            .field("type", &T::NAME)
            .field("dims", &self.dims)
            .field("data_len", &self.data.len())
            .field("layout", &self.layout)
            .finish()
    }
}

/// A raw bytes async transfer operation ready to execute.
///
/// This struct is created by [`AsyncTransferBuilder::raw`] and holds all
/// the configuration needed to perform an async transfer of raw bytes.
///
/// Use this when the element type is only known at runtime.
///
/// # Example
///
/// ```rust,ignore
/// use pjrt::{AsyncTransferBuilder, PrimitiveType};
///
/// let bytes: Vec<u8> = /* raw data */;
/// let buffer = AsyncTransferBuilder::new(&client, &device)
///     .raw(&bytes, &[10, 10], PrimitiveType::F32)
///     .transfer()
///     .await?;
/// ```
pub struct RawAsyncTransfer<'a> {
    client: &'a Client,
    device: &'a Device,
    memory: Option<&'a Memory>,
    data: &'a [u8],
    dims: &'a [i64],
    element_type: PrimitiveType,
    layout: Option<MemoryLayout>,
}

impl<'a> RawAsyncTransfer<'a> {
    /// Specifies a custom memory layout for the device buffer.
    ///
    /// # Arguments
    ///
    /// * `layout` - The memory layout to use on the device
    pub fn layout(mut self, layout: MemoryLayout) -> Self {
        self.layout = Some(layout);
        self
    }

    /// Performs the transfer asynchronously.
    ///
    /// # Returns
    ///
    /// The device buffer containing the transferred data.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The device has no default memory
    /// - Buffer creation fails
    /// - The transfer fails
    pub async fn transfer(self) -> Result<Buffer> {
        let default_memory;
        let memory = match self.memory {
            Some(m) => m,
            None => {
                default_memory = self.device.default_memory()?;
                &default_memory
            }
        };

        let mut shape = BufferShape::new(self.dims.to_vec(), self.element_type);
        if let Some(layout) = self.layout {
            shape = shape.with_layout(layout);
        }

        let manager = self
            .client
            .create_buffers_for_async_host_to_device(&[shape], memory)?;

        manager.transfer_all(0, self.data).await?;

        manager.retrieve_buffer(0)
    }

    /// Performs the transfer synchronously.
    ///
    /// This method blocks until the transfer is complete.
    ///
    /// # Returns
    ///
    /// The device buffer containing the transferred data.
    pub fn transfer_sync(self) -> Result<Buffer> {
        let default_memory;
        let memory = match self.memory {
            Some(m) => m,
            None => {
                default_memory = self.device.default_memory()?;
                &default_memory
            }
        };

        let mut shape = BufferShape::new(self.dims.to_vec(), self.element_type);
        if let Some(layout) = self.layout {
            shape = shape.with_layout(layout);
        }

        let manager = self
            .client
            .create_buffers_for_async_host_to_device(&[shape], memory)?;

        manager.transfer_all_sync(0, self.data)?;

        manager.retrieve_buffer(0)
    }
}

impl std::fmt::Debug for RawAsyncTransfer<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RawAsyncTransfer")
            .field("element_type", &self.element_type)
            .field("dims", &self.dims)
            .field("data_len", &self.data.len())
            .field("layout", &self.layout)
            .finish()
    }
}

// =============================================================================
// Multi-Buffer Transfer Builder
// =============================================================================

/// A builder for transferring multiple buffers in a single operation.
///
/// `MultiBufTransfer` provides a convenient way to set up and execute
/// transfers of multiple tensors to a device. It manages the
/// [`AsyncHostToDeviceTransferManager`] internally and provides a clean API.
///
/// # Features
///
/// - Transfer multiple buffers in one operation
/// - Type-safe API with compile-time checks
/// - Automatic shape inference from data
/// - Supports mixed types in a single transfer
///
/// # Example
///
/// ```rust,ignore
/// use pjrt::{Client, MultiBufTransfer, F32, I32, Result};
///
/// async fn transfer_multiple(client: &Client) -> Result<Vec<pjrt::Buffer>> {
///     let device = client.addressable_devices().first().unwrap();
///     let memory = device.default_memory()?;
///
///     let weights: Vec<f32> = vec![0.1, 0.2, 0.3, 0.4];
///     let indices: Vec<i32> = vec![0, 1, 2, 3];
///
///     MultiBufTransfer::new(client, &memory)
///         .add_typed::<F32>(&weights, &[2, 2])
///         .add_typed::<I32>(&indices, &[4])
///         .transfer()
///         .await
/// }
/// ```
pub struct MultiBufTransfer<'a> {
    client: &'a Client,
    memory: &'a Memory,
    shapes: Vec<BufferShape>,
    transfers: Vec<PendingTransfer<'a>>,
}

/// Internal representation of a pending transfer.
enum PendingTransfer<'a> {
    /// Raw bytes transfer
    Raw { data: &'a [u8] },
    /// Typed transfer stored as bytes with element size
    Typed {
        data_ptr: *const c_void,
        data_len: usize,
        element_size: usize,
    },
}

impl<'a> MultiBufTransfer<'a> {
    /// Creates a new multi-buffer transfer builder.
    ///
    /// # Arguments
    ///
    /// * `client` - The PJRT client
    /// * `memory` - The target memory for all buffers
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let memory = device.default_memory()?;
    /// let transfer = MultiBufTransfer::new(&client, &memory);
    /// ```
    pub fn new(client: &'a Client, memory: &'a Memory) -> Self {
        Self {
            client,
            memory,
            shapes: Vec::new(),
            transfers: Vec::new(),
        }
    }

    /// Adds a typed buffer to the transfer.
    ///
    /// # Type Parameters
    ///
    /// * `T` - A PJRT type marker (e.g., [`F32`][crate::F32], [`I32`][crate::I32])
    ///
    /// # Arguments
    ///
    /// * `data` - The typed data to transfer
    /// * `dims` - The dimensions of the tensor
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// transfer
    ///     .add_typed::<F32>(&weights, &[100, 100])
    ///     .add_typed::<I32>(&indices, &[1000]);
    /// ```
    pub fn add_typed<T: crate::Type>(mut self, data: &'a [T::ElemType], dims: &[i64]) -> Self {
        self.shapes
            .push(BufferShape::new(dims.to_vec(), T::PRIMITIVE_TYPE));
        self.transfers.push(PendingTransfer::Typed {
            data_ptr: data.as_ptr() as *const c_void,
            data_len: data.len(),
            element_size: T::SIZE,
        });
        self
    }

    /// Adds a raw bytes buffer to the transfer.
    ///
    /// # Arguments
    ///
    /// * `data` - The raw bytes to transfer
    /// * `dims` - The dimensions of the tensor
    /// * `element_type` - The primitive type of the elements
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// transfer.add_raw(&bytes, &[10, 10], PrimitiveType::F32);
    /// ```
    pub fn add_raw(mut self, data: &'a [u8], dims: &[i64], element_type: PrimitiveType) -> Self {
        self.shapes
            .push(BufferShape::new(dims.to_vec(), element_type));
        self.transfers.push(PendingTransfer::Raw { data });
        self
    }

    /// Executes all transfers asynchronously and returns the buffers.
    ///
    /// # Returns
    ///
    /// A vector of device buffers in the same order as they were added.
    ///
    /// # Errors
    ///
    /// Returns an error if any transfer fails.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let buffers = MultiBufTransfer::new(&client, &memory)
    ///     .add_typed::<F32>(&data1, &[100])
    ///     .add_typed::<I32>(&data2, &[50])
    ///     .transfer()
    ///     .await?;
    /// ```
    pub async fn transfer(self) -> Result<Vec<Buffer>> {
        if self.shapes.is_empty() {
            return Ok(Vec::new());
        }

        let manager = self
            .client
            .create_buffers_for_async_host_to_device(&self.shapes, self.memory)?;

        for (i, pending) in self.transfers.into_iter().enumerate() {
            match pending {
                PendingTransfer::Raw { data } => {
                    manager.transfer_all(i as i32, data).await?;
                }
                PendingTransfer::Typed {
                    data_ptr,
                    data_len,
                    element_size,
                } => {
                    // Convert to bytes for transfer
                    let byte_len = data_len * element_size;
                    let bytes =
                        unsafe { std::slice::from_raw_parts(data_ptr as *const u8, byte_len) };
                    manager
                        .transfer_data(i as i32)
                        .data(bytes)
                        .is_last_transfer(true)
                        .transfer()?
                        .await?;
                    // Note: We use transfer_data instead of transfer_literal to avoid
                    // needing to know the concrete type T at this point. The shape
                    // already encodes the element type.
                }
            }
        }

        manager.retrieve_all_buffers()
    }

    /// Executes all transfers synchronously and returns the buffers.
    ///
    /// # Returns
    ///
    /// A vector of device buffers in the same order as they were added.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let buffers = MultiBufTransfer::new(&client, &memory)
    ///     .add_typed::<F32>(&data1, &[100])
    ///     .transfer_sync()?;
    /// ```
    pub fn transfer_sync(self) -> Result<Vec<Buffer>> {
        if self.shapes.is_empty() {
            return Ok(Vec::new());
        }

        let manager = self
            .client
            .create_buffers_for_async_host_to_device(&self.shapes, self.memory)?;

        for (i, pending) in self.transfers.into_iter().enumerate() {
            match pending {
                PendingTransfer::Raw { data } => {
                    manager.transfer_all_sync(i as i32, data)?;
                }
                PendingTransfer::Typed {
                    data_ptr,
                    data_len,
                    element_size,
                } => {
                    let byte_len = data_len * element_size;
                    let bytes =
                        unsafe { std::slice::from_raw_parts(data_ptr as *const u8, byte_len) };
                    manager
                        .transfer_data(i as i32)
                        .data(bytes)
                        .is_last_transfer(true)
                        .transfer()?
                        .wait()?;
                }
            }
        }

        manager.retrieve_all_buffers()
    }
}

impl std::fmt::Debug for MultiBufTransfer<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MultiBufTransfer")
            .field("num_buffers", &self.shapes.len())
            .field("shapes", &self.shapes)
            .finish()
    }
}
