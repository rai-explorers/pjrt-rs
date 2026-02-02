//! PJRT Host Buffer Management
//!
//! This module provides host-side buffer management for PJRT. Host buffers represent
//! data stored in host (CPU) memory that can be transferred to and from PJRT devices.
//!
//! The module provides two main types:
//!
//! - `TypedHostBuffer<T>`: A type-safe buffer with a specific element type
//! - `HostBuffer`: An enum that can hold any supported type
//!
//! These buffers support:
//! - Creating buffers from typed data or raw bytes
//! - Transferring data to devices (async and sync)
//! - Various memory semantics for optimization
//!
//! # Examples
//!
//! ## Creating Typed Host Buffers
//!
//! ```rust
//! use pjrt::{TypedHostBuffer, F32, F64, I32};
//!
//! // Create from a scalar
//! let scalar = TypedHostBuffer::<F32>::from_scalar(42.0f32);
//! assert_eq!(scalar.data(), &[42.0f32]);
//! assert!(scalar.dims().is_empty()); // Scalar has no dimensions
//!
//! // Create from a vector with automatic shape inference
//! let vector = TypedHostBuffer::<F32>::from_data(vec![1.0, 2.0, 3.0, 4.0], None, None);
//! assert_eq!(vector.dims(), &[4]); // 1D shape inferred
//!
//! // Create with explicit shape
//! let matrix = TypedHostBuffer::<F64>::from_data(
//!     vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0],
//!     Some(vec![2, 3]),  // 2x3 matrix
//!     None
//! );
//! assert_eq!(matrix.dims(), &[2, 3]);
//! ```
//!
//! ## Transferring to Device
//!
//! ```rust,ignore
//! use pjrt::{TypedHostBuffer, F32, Client};
//!
//! let host_buffer = TypedHostBuffer::<F32>::from_data(
//!     vec![1.0, 2.0, 3.0, 4.0],
//!     Some(vec![2, 2]),
//!     None
//! );
//!
//! // Synchronous transfer (blocking)
//! let device_buffer = host_buffer.to_sync(&client).copy()?;
//!
//! // Asynchronous transfer (non-blocking)
//! let device_buffer = host_buffer.to(&client).copy().await?;
//!
//! // Transfer to a specific device
//! let device = client.addressable_devices().first().unwrap();
//! let device_buffer = host_buffer.to_sync(device).copy()?;
//!
//! // Transfer to a specific memory
//! let memory = device.default_memory();
//! let device_buffer = host_buffer.to_sync(&memory).copy()?;
//! ```
//!
//! ## Using the HostBuffer Enum
//!
//! ```rust
//! use pjrt::{HostBuffer, TypedHostBuffer, F32, I32};
//!
//! // Convert typed buffers to the generic enum
//! let f32_buf: HostBuffer = TypedHostBuffer::<F32>::from_scalar(1.5).into();
//! let i32_buf: HostBuffer = TypedHostBuffer::<I32>::from_scalar(42).into();
//!
//! // The enum can hold any supported type
//! let buffers: Vec<HostBuffer> = vec![f32_buf, i32_buf];
//! ```
//!
//! ## Creating from Raw Bytes
//!
//! ```rust
//! use pjrt::{TypedHostBuffer, F32};
//!
//! // When you have raw bytes (e.g., from a file)
//! let bytes: Vec<u8> = vec![0, 0, 128, 63, 0, 0, 0, 64]; // [1.0f32, 2.0f32] in little-endian
//! let buffer = TypedHostBuffer::<F32>::from_bytes(bytes, Some(vec![2]), None);
//! ```

use std::ffi::c_void;
use std::mem;
use std::rc::Rc;

use bon::bon;
use pjrt_sys::{
    PJRT_Buffer_MemoryLayout, PJRT_Buffer_Type, PJRT_Client_BufferFromHostBuffer_Args,
    PJRT_HostBufferSemantics,
    PJRT_HostBufferSemantics_PJRT_HostBufferSemantics_kImmutableOnlyDuringCall,
    PJRT_HostBufferSemantics_PJRT_HostBufferSemantics_kImmutableUntilTransferCompletes,
    PJRT_HostBufferSemantics_PJRT_HostBufferSemantics_kImmutableZeroCopy,
    PJRT_HostBufferSemantics_PJRT_HostBufferSemantics_kMutableZeroCopy,
};

use crate::event::Event;
use crate::{
    utils, Buffer, Client, Device, ElemType, Error, Memory, MemoryLayout, PrimitiveType, Result,
    Type, BF16, C128, C64, F16, F32, F64, I16, I32, I64, I8, U16, U32, U64, U8,
};

/// A type-safe host buffer with a specific element type.
///
/// `TypedHostBuffer` provides compile-time type safety for host buffers,
/// ensuring that the data type is known and consistent throughout operations.
///
/// The type parameter `T` must implement the `Type` trait, which includes
/// all supported PJRT types like `F32`, `F64`, `I32`, etc.
#[derive(Debug)]
pub struct TypedHostBuffer<T: Type> {
    data: Rc<Vec<T::ElemType>>,
    dims: Vec<i64>,
    layout: MemoryLayout,
}

#[bon]
impl<T: Type> TypedHostBuffer<T> {
    pub fn from_data(
        data: Vec<T::ElemType>,
        dims: Option<Vec<i64>>,
        layout: Option<MemoryLayout>,
    ) -> Self {
        let dims = dims.unwrap_or_else(|| vec![data.len() as i64]);
        let layout = layout
            .unwrap_or_else(|| MemoryLayout::from_strides(utils::byte_strides(&dims, T::SIZE)));
        Self {
            data: Rc::new(data),
            dims,
            layout,
        }
    }

    pub fn from_bytes(
        bytes: Vec<u8>,
        dims: Option<Vec<i64>>,
        layout: Option<MemoryLayout>,
    ) -> Self {
        let length = bytes.len() / T::SIZE;
        let capacity = bytes.capacity() / T::SIZE;
        let ptr = bytes.as_ptr() as *mut T::ElemType;
        let data = unsafe { Vec::from_raw_parts(ptr, length, capacity) };
        mem::forget(bytes);
        let dims = dims.unwrap_or_else(|| vec![length as i64]);
        assert!(dims.iter().product::<i64>() == length as i64);
        let layout = layout
            .unwrap_or_else(|| MemoryLayout::from_strides(utils::byte_strides(&dims, T::SIZE)));
        Self {
            data: Rc::new(data),
            dims,
            layout,
        }
    }

    pub fn from_scalar(data: T::ElemType) -> Self {
        let data = vec![data];
        let dims = vec![];
        let layout = MemoryLayout::from_strides(vec![]);
        Self {
            data: Rc::new(data),
            dims,
            layout,
        }
    }

    pub fn data(&self) -> &[T::ElemType] {
        &self.data
    }

    pub fn dims(&self) -> &[i64] {
        &self.dims
    }

    pub fn layout(&self) -> &MemoryLayout {
        &self.layout
    }

    pub(crate) fn call_copy_to<D>(
        &self,
        dest: &D,
        byte_strides: Option<Vec<i64>>,
        device_layout: Option<MemoryLayout>,
    ) -> Result<PJRT_Client_BufferFromHostBuffer_Args>
    where
        D: HostBufferCopyToDest,
    {
        let client = dest.client();
        let mut args = PJRT_Client_BufferFromHostBuffer_Args::new();
        args.client = client.ptr();
        args.data = self.data.as_ptr() as *const c_void;
        args.type_ = T::PRIMITIVE_TYPE as PJRT_Buffer_Type;
        args.dims = self.dims.as_ptr();
        args.num_dims = self.dims.len();
        args.host_buffer_semantics =
            HostBufferSemantics::ImmutableUntilTransferCompletes as PJRT_HostBufferSemantics;
        if let Some(byte_strides) = &byte_strides {
            args.byte_strides = byte_strides.as_ptr() as *const _;
            args.num_byte_strides = byte_strides.len();
        }
        if let Some(device_layout) = &device_layout {
            let mut device_layout = PJRT_Buffer_MemoryLayout::from(device_layout);
            args.device_layout = &mut device_layout as *mut _;
        }
        dest.set_args(&mut args)?;
        client.api().PJRT_Client_BufferFromHostBuffer(args)
    }

    #[builder(finish_fn = copy)]
    pub fn to_sync<D>(
        &self,
        #[builder(start_fn)] dest: &D,
        byte_strides: Option<Vec<i64>>,
        device_layout: Option<MemoryLayout>,
    ) -> Result<Buffer>
    where
        D: HostBufferCopyToDest,
    {
        let args = self.call_copy_to(dest, byte_strides, device_layout)?;
        let done_with_host_event = Event::wrap(dest.client().api(), args.done_with_host_buffer);
        done_with_host_event.wait()?;
        let buf = Buffer::wrap(dest.client(), args.buffer);
        let buf_ready_event = buf.ready_event()?;
        buf_ready_event.wait()?;
        Ok(buf)
    }

    #[builder(finish_fn = copy)]
    pub async fn to<D>(
        &self,
        #[builder(start_fn)] dest: &D,
        byte_strides: Option<Vec<i64>>,
        device_layout: Option<MemoryLayout>,
    ) -> Result<Buffer>
    where
        D: HostBufferCopyToDest,
    {
        let args = self.call_copy_to(dest, byte_strides, device_layout)?;
        let done_with_host_event = Event::wrap(dest.client().api(), args.done_with_host_buffer);
        done_with_host_event.await?;
        let buf = Buffer::wrap(dest.client(), args.buffer);
        let buf_ready_event = buf.ready_event()?;
        buf_ready_event.await?;
        Ok(buf)
    }
}

macro_rules! impl_from_typed_buffer {
    ($T:ident) => {
        impl From<TypedHostBuffer<$T>> for HostBuffer {
            fn from(buf: TypedHostBuffer<$T>) -> Self {
                Self::$T(buf)
            }
        }
    };
}

impl_from_typed_buffer!(BF16);
impl_from_typed_buffer!(F16);
impl_from_typed_buffer!(F32);
impl_from_typed_buffer![F64];
impl_from_typed_buffer![I8];
impl_from_typed_buffer![I16];
impl_from_typed_buffer![I32];
impl_from_typed_buffer![I64];
impl_from_typed_buffer![U8];
impl_from_typed_buffer![U16];
impl_from_typed_buffer![U32];
impl_from_typed_buffer![U64];
impl_from_typed_buffer![C64];
impl_from_typed_buffer![C128];

/// An enum representing a host buffer of any supported type.
///
/// `HostBuffer` provides a way to work with buffers of different types
/// in a uniform manner. Each variant wraps a `TypedHostBuffer` of the
/// corresponding type.
///
/// This is useful when the type is only known at runtime, or when you
/// need to store buffers of different types in a collection.
#[derive(Debug)]
pub enum HostBuffer {
    BF16(TypedHostBuffer<BF16>),
    F16(TypedHostBuffer<F16>),
    F32(TypedHostBuffer<F32>),
    F64(TypedHostBuffer<F64>),
    I8(TypedHostBuffer<I8>),
    I16(TypedHostBuffer<I16>),
    I32(TypedHostBuffer<I32>),
    I64(TypedHostBuffer<I64>),
    U8(TypedHostBuffer<U8>),
    U16(TypedHostBuffer<U16>),
    U32(TypedHostBuffer<U32>),
    U64(TypedHostBuffer<U64>),
    C64(TypedHostBuffer<C64>),
    C128(TypedHostBuffer<C128>),
}

#[bon]
impl HostBuffer {
    pub fn from_data<E>(data: Vec<E>, dims: Option<Vec<i64>>, layout: Option<MemoryLayout>) -> Self
    where
        E: ElemType,
        Self: From<TypedHostBuffer<E::Type>>,
    {
        let buf = TypedHostBuffer::<E::Type>::from_data(data, dims, layout);
        Self::from(buf)
    }

    pub fn from_bytes(
        bytes: Vec<u8>,
        ty: PrimitiveType,
        dims: Option<Vec<i64>>,
        layout: Option<MemoryLayout>,
    ) -> Result<Self> {
        match ty {
            PrimitiveType::BF16 => Ok(Self::BF16(TypedHostBuffer::from_bytes(bytes, dims, layout))),
            PrimitiveType::F16 => Ok(Self::F16(TypedHostBuffer::from_bytes(bytes, dims, layout))),
            PrimitiveType::F32 => Ok(Self::F32(TypedHostBuffer::from_bytes(bytes, dims, layout))),
            PrimitiveType::F64 => Ok(Self::F64(TypedHostBuffer::from_bytes(bytes, dims, layout))),
            PrimitiveType::S8 => Ok(Self::I8(TypedHostBuffer::from_bytes(bytes, dims, layout))),
            PrimitiveType::S16 => Ok(Self::I16(TypedHostBuffer::from_bytes(bytes, dims, layout))),
            PrimitiveType::S32 => Ok(Self::I32(TypedHostBuffer::from_bytes(bytes, dims, layout))),
            PrimitiveType::S64 => Ok(Self::I64(TypedHostBuffer::from_bytes(bytes, dims, layout))),
            PrimitiveType::U8 => Ok(Self::U8(TypedHostBuffer::from_bytes(bytes, dims, layout))),
            PrimitiveType::U16 => Ok(Self::U16(TypedHostBuffer::from_bytes(bytes, dims, layout))),
            PrimitiveType::U32 => Ok(Self::U32(TypedHostBuffer::from_bytes(bytes, dims, layout))),
            PrimitiveType::U64 => Ok(Self::U64(TypedHostBuffer::from_bytes(bytes, dims, layout))),
            PrimitiveType::C64 => Ok(Self::C64(TypedHostBuffer::from_bytes(bytes, dims, layout))),
            PrimitiveType::C128 => Ok(Self::C128(TypedHostBuffer::from_bytes(bytes, dims, layout))),
            _ => Err(Error::NotSupportedType(ty)),
        }
    }

    pub fn from_scalar<E>(data: E) -> Self
    where
        E: ElemType,
        Self: From<TypedHostBuffer<E::Type>>,
    {
        let buf = TypedHostBuffer::<E::Type>::from_scalar(data);
        Self::from(buf)
    }

    pub fn dims(&self) -> &[i64] {
        match self {
            Self::BF16(buf) => buf.dims(),
            Self::F16(buf) => buf.dims(),
            Self::F32(buf) => buf.dims(),
            Self::F64(buf) => buf.dims(),
            Self::I8(buf) => buf.dims(),
            Self::I16(buf) => buf.dims(),
            Self::I32(buf) => buf.dims(),
            Self::I64(buf) => buf.dims(),
            Self::U8(buf) => buf.dims(),
            Self::U16(buf) => buf.dims(),
            Self::U32(buf) => buf.dims(),
            Self::U64(buf) => buf.dims(),
            Self::C64(buf) => buf.dims(),
            Self::C128(buf) => buf.dims(),
        }
    }

    pub fn layout(&self) -> &MemoryLayout {
        match self {
            Self::BF16(buf) => buf.layout(),
            Self::F16(buf) => buf.layout(),
            Self::F32(buf) => buf.layout(),
            Self::F64(buf) => buf.layout(),
            Self::I8(buf) => buf.layout(),
            Self::I16(buf) => buf.layout(),
            Self::I32(buf) => buf.layout(),
            Self::I64(buf) => buf.layout(),
            Self::U8(buf) => buf.layout(),
            Self::U16(buf) => buf.layout(),
            Self::U32(buf) => buf.layout(),
            Self::U64(buf) => buf.layout(),
            Self::C64(buf) => buf.layout(),
            Self::C128(buf) => buf.layout(),
        }
    }

    /// Read the buffer data as f32 values
    ///
    /// Returns the data as a slice of f32 if the buffer contains F32 data.
    /// Returns an error if the buffer contains a different type.
    pub fn read_f32(&self) -> Result<&[f32]> {
        match self {
            Self::F32(buf) => Ok(buf.data()),
            _ => Err(crate::Error::InvalidArgument(format!(
                "Cannot read {:?} buffer as f32",
                self.primitive_type()
            ))),
        }
    }

    /// Get the primitive type of this buffer
    pub fn primitive_type(&self) -> PrimitiveType {
        match self {
            Self::BF16(_) => PrimitiveType::BF16,
            Self::F16(_) => PrimitiveType::F16,
            Self::F32(_) => PrimitiveType::F32,
            Self::F64(_) => PrimitiveType::F64,
            Self::I8(_) => PrimitiveType::S8,
            Self::I16(_) => PrimitiveType::S16,
            Self::I32(_) => PrimitiveType::S32,
            Self::I64(_) => PrimitiveType::S64,
            Self::U8(_) => PrimitiveType::U8,
            Self::U16(_) => PrimitiveType::U16,
            Self::U32(_) => PrimitiveType::U32,
            Self::U64(_) => PrimitiveType::U64,
            Self::C64(_) => PrimitiveType::C64,
            Self::C128(_) => PrimitiveType::C128,
        }
    }
    pub(crate) fn call_copy_to<D>(
        &self,
        dest: &D,
        byte_strides: Option<Vec<i64>>,
        device_layout: Option<MemoryLayout>,
    ) -> Result<PJRT_Client_BufferFromHostBuffer_Args>
    where
        D: HostBufferCopyToDest,
    {
        match self {
            Self::BF16(buf) => buf.call_copy_to(dest, byte_strides, device_layout),
            Self::F16(buf) => buf.call_copy_to(dest, byte_strides, device_layout),
            Self::F32(buf) => buf.call_copy_to(dest, byte_strides, device_layout),
            Self::F64(buf) => buf.call_copy_to(dest, byte_strides, device_layout),
            Self::I8(buf) => buf.call_copy_to(dest, byte_strides, device_layout),
            Self::I16(buf) => buf.call_copy_to(dest, byte_strides, device_layout),
            Self::I32(buf) => buf.call_copy_to(dest, byte_strides, device_layout),
            Self::I64(buf) => buf.call_copy_to(dest, byte_strides, device_layout),
            Self::U8(buf) => buf.call_copy_to(dest, byte_strides, device_layout),
            Self::U16(buf) => buf.call_copy_to(dest, byte_strides, device_layout),
            Self::U32(buf) => buf.call_copy_to(dest, byte_strides, device_layout),
            Self::U64(buf) => buf.call_copy_to(dest, byte_strides, device_layout),
            Self::C64(buf) => buf.call_copy_to(dest, byte_strides, device_layout),
            Self::C128(buf) => buf.call_copy_to(dest, byte_strides, device_layout),
        }
    }

    #[builder(finish_fn = copy)]
    pub fn to_sync<D>(
        &self,
        #[builder(start_fn)] dest: &D,
        byte_strides: Option<Vec<i64>>,
        device_layout: Option<MemoryLayout>,
    ) -> Result<Buffer>
    where
        D: HostBufferCopyToDest,
    {
        let client = dest.client().clone();
        let args = self.call_copy_to(dest, byte_strides, device_layout)?;
        let done_with_host_event = Event::wrap(client.api(), args.done_with_host_buffer);
        done_with_host_event.wait()?;
        let buf = Buffer::wrap(&client, args.buffer);
        let buf_ready_event = buf.ready_event()?;
        buf_ready_event.wait()?;
        Ok(buf)
    }

    #[builder(finish_fn = copy)]
    pub async fn to<D>(
        &self,
        #[builder(start_fn)] dest: &D,
        byte_strides: Option<Vec<i64>>,
        device_layout: Option<MemoryLayout>,
    ) -> Result<Buffer>
    where
        D: HostBufferCopyToDest,
    {
        let client = dest.client().clone();
        let args = self.call_copy_to(dest, byte_strides, device_layout)?;
        let done_with_host_event = Event::wrap(client.api(), args.done_with_host_buffer);
        done_with_host_event.await?;
        let buf = Buffer::wrap(&client, args.buffer);
        let buf_ready_event = buf.ready_event()?;
        buf_ready_event.await?;
        Ok(buf)
    }
}

/// Specifies how the PJRT runtime should handle host buffer memory during transfers.
///
/// `HostBufferSemantics` controls the ownership and mutability guarantees between
/// the caller and the PJRT runtime when transferring data from host to device.
/// Choosing the right semantics can significantly impact performance and correctness.
///
/// # Overview
///
/// When you transfer data from host (CPU) memory to a PJRT device (CPU, GPU, TPU),
/// the runtime needs to know how long it can access your source data and whether
/// it can modify it. These semantics form a contract between your code and the runtime.
///
/// # Memory Safety Contract
///
/// Each semantic variant establishes a different contract:
///
/// | Semantic | Data Lifetime | Mutability | Copy Behavior |
/// |----------|--------------|------------|---------------|
/// | [`ImmutableOnlyDuringCall`][Self::ImmutableOnlyDuringCall] | API call duration | Immutable | Always copies |
/// | [`ImmutableUntilTransferCompletes`][Self::ImmutableUntilTransferCompletes] | Until event fires | Immutable | May copy async |
/// | [`ImmutableZeroCopy`][Self::ImmutableZeroCopy] | Buffer lifetime | Immutable | Zero-copy on CPU |
/// | [`MutableZeroCopy`][Self::MutableZeroCopy] | Buffer lifetime | Runtime may mutate | Zero-copy on CPU |
///
/// # Performance Comparison
///
/// From fastest to slowest on CPU backends:
///
/// 1. **`ImmutableZeroCopy`** / **`MutableZeroCopy`** - No copy, direct memory aliasing
/// 2. **`ImmutableUntilTransferCompletes`** - Async copy, non-blocking
/// 3. **`ImmutableOnlyDuringCall`** - Sync copy, blocks until complete
///
/// On GPU/TPU backends, all semantics require a copy to device memory, but
/// `ImmutableUntilTransferCompletes` allows async DMA transfers.
///
/// # Decision Guide
///
/// ```text
///                  ┌─────────────────────────────────────────┐
///                  │   Do you need to reuse/free data       │
///                  │   immediately after the API call?      │
///                  └───────────────┬─────────────────────────┘
///                           │
///              ┌────────────┴────────────┐
///              │ YES                     │ NO
///              ▼                         ▼
///   ┌──────────────────┐     ┌─────────────────────────────┐
///   │ ImmutableOnly-   │     │  Is the data long-lived and │
///   │ DuringCall       │     │  can outlive the buffer?    │
///   └──────────────────┘     └──────────────┬──────────────┘
///                                     │
///                        ┌────────────┴────────────┐
///                        │ YES                     │ NO
///                        ▼                         ▼
///             ┌────────────────────┐    ┌────────────────────┐
///             │ Are you on CPU and │    │ ImmutableUntil-    │
///             │ want zero-copy?    │    │ TransferCompletes  │
///             └─────────┬──────────┘    └────────────────────┘
///                       │
///          ┌────────────┴────────────┐
///          │ YES                     │ NO
///          ▼                         ▼
///   ┌─────────────────┐    ┌────────────────────┐
///   │ Does runtime    │    │ ImmutableUntil-    │
///   │ need to write?  │    │ TransferCompletes  │
///   └────────┬────────┘    └────────────────────┘
///            │
///   ┌────────┴────────┐
///   │ YES             │ NO
///   ▼                 ▼
/// ┌───────────┐  ┌─────────────┐
/// │ Mutable-  │  │ Immutable-  │
/// │ ZeroCopy  │  │ ZeroCopy    │
/// └───────────┘  └─────────────┘
/// ```
///
/// # Examples
///
/// ## Scenario 1: Quick transfer with immediate data reuse
///
/// Use [`ImmutableOnlyDuringCall`][Self::ImmutableOnlyDuringCall] when you need to
/// modify or free the source data right after initiating the transfer:
///
/// ```rust,ignore
/// use pjrt::HostBufferSemantics;
///
/// let mut data = vec![1.0f32, 2.0, 3.0, 4.0];
/// let host_buffer = HostBuffer::from_data(data.clone(), None, None);
///
/// // Transfer with ImmutableOnlyDuringCall semantics
/// // Runtime copies data synchronously during this call
/// let device_buffer = host_buffer.to_sync(&client).copy()?;
///
/// // Safe to immediately reuse the data vector!
/// data[0] = 999.0; // This is safe
/// ```
///
/// ## Scenario 2: Async transfer with event-based synchronization
///
/// Use [`ImmutableUntilTransferCompletes`][Self::ImmutableUntilTransferCompletes]
/// for the best balance of safety and performance (this is the default):
///
/// ```rust,ignore
/// use std::sync::Arc;
///
/// // Keep data alive until transfer completes
/// let data = Arc::new(vec![1.0f32, 2.0, 3.0, 4.0]);
/// let host_buffer = HostBuffer::from_data((*data).clone(), None, None);
///
/// // Async transfer - runtime may access data after call returns
/// let device_buffer = host_buffer.to(&client).copy().await?;
/// // The await ensures done_with_host_buffer event has fired
/// // Now safe to drop or modify data
/// ```
///
/// ## Scenario 3: Zero-copy on CPU for large, long-lived data
///
/// Use [`ImmutableZeroCopy`][Self::ImmutableZeroCopy] on CPU backends when your
/// data will outlive the device buffer and you want to avoid copying:
///
/// ```rust,ignore
/// use std::sync::Arc;
///
/// // Large dataset that lives for the program's duration
/// static WEIGHTS: &[f32] = &[/* ... large const array ... */];
///
/// // On CPU: buffer directly aliases WEIGHTS, no copy!
/// // On GPU/TPU: behaves like ImmutableUntilTransferCompletes
/// let device_buffer = /* transfer with ImmutableZeroCopy */;
///
/// // WEIGHTS must not be freed or modified while device_buffer exists
/// ```
///
/// ## Scenario 4: In-place output buffers (advanced)
///
/// Use [`MutableZeroCopy`][Self::MutableZeroCopy] for advanced scenarios where the
/// runtime needs to write results back to host memory:
///
/// ```rust,ignore
/// // WARNING: Advanced use case - understand synchronization first!
///
/// // Pre-allocate output buffer
/// let mut output_data = vec![0.0f32; 1000];
///
/// // Create buffer with MutableZeroCopy - runtime can write here
/// let output_buffer = /* create with MutableZeroCopy */;
///
/// // Execute computation that writes to output_buffer
/// executable.execute(&[input_buffer], &[output_buffer])?;
///
/// // Wait for execution to complete before reading
/// output_buffer.ready_event()?.wait()?;
///
/// // Now safe to read results from output_data
/// println!("First result: {}", output_data[0]);
/// ```
///
/// # Platform-Specific Behavior
///
/// | Platform | ImmutableOnlyDuringCall | ImmutableUntilTransferCompletes | ImmutableZeroCopy | MutableZeroCopy |
/// |----------|------------------------|--------------------------------|-------------------|-----------------|
/// | **CPU**  | Sync copy              | Async copy                     | Zero-copy alias   | Zero-copy alias |
/// | **GPU**  | Sync H2D copy          | Async DMA H2D                  | Async DMA H2D     | Async DMA H2D   |
/// | **TPU**  | Sync H2D copy          | Async H2D                      | Async H2D         | Async H2D       |
///
/// # Default Semantics
///
/// The high-level transfer APIs ([`TypedHostBuffer::to`], [`TypedHostBuffer::to_sync`],
/// [`HostBuffer::to`], [`HostBuffer::to_sync`]) use `ImmutableUntilTransferCompletes`
/// by default, which provides a good balance of safety and performance.
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[allow(dead_code)]
pub enum HostBufferSemantics {
    /// The runtime may not hold references to `data` after the API call completes.
    ///
    /// # Memory Safety
    ///
    /// This is the **safest** semantic for the caller because you can immediately
    /// reuse or free the source data after the transfer call returns. The runtime
    /// must copy the data synchronously during the API call.
    ///
    /// # When to Use
    ///
    /// - You need to modify the source data array immediately after the transfer
    /// - The source data is temporary (e.g., a local variable going out of scope)
    /// - You're transferring from a borrowed slice that won't remain valid
    /// - You want simple, predictable behavior and don't need async optimization
    ///
    /// # Performance Implications
    ///
    /// - **Always copies**: The runtime must copy all data before the call returns
    /// - **Blocking**: The API call blocks until the copy is complete
    /// - **Memory overhead**: Temporary buffer allocation may be required
    /// - **Latency**: Higher latency for large transfers due to synchronous copy
    ///
    /// This is the slowest option but guarantees the simplest memory management.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let mut data = vec![1.0f32, 2.0, 3.0, 4.0];
    /// let host_buffer = HostBuffer::from_data(data.clone(), None, None);
    ///
    /// // With ImmutableOnlyDuringCall, data is copied during this call
    /// let device_buffer = host_buffer.to_sync(&client).copy()?;
    ///
    /// // Immediately safe to modify the original data
    /// data[0] = 999.0;
    /// data.clear();
    /// ```
    ///
    /// # Guarantees
    ///
    /// | Party | Guarantee |
    /// |-------|-----------|
    /// | **Caller** | Data is valid and immutable during the API call |
    /// | **Runtime** | Will not access data after the API call returns |
    ImmutableOnlyDuringCall =
        PJRT_HostBufferSemantics_PJRT_HostBufferSemantics_kImmutableOnlyDuringCall as i32,

    /// The runtime may hold references to `data` until the transfer completes.
    ///
    /// # Memory Safety
    ///
    /// The caller must keep the source data valid and unmodified until the
    /// `done_with_host_buffer` event fires. The high-level async APIs
    /// ([`TypedHostBuffer::to`], [`HostBuffer::to`]) await this event automatically.
    ///
    /// **Warning**: Modifying or freeing the data before the event fires causes
    /// undefined behavior!
    ///
    /// # When to Use
    ///
    /// - **Default choice** for most use cases
    /// - You can keep the source data alive until transfer completes
    /// - You want async transfers for better throughput
    /// - You're using the high-level `to()` or `to_sync()` APIs (which handle this)
    ///
    /// # Performance Implications
    ///
    /// - **Async copy**: Data may be copied asynchronously after the call returns
    /// - **Non-blocking**: API call returns immediately, transfer happens in background
    /// - **DMA friendly**: On GPU/TPU, enables efficient DMA transfers
    /// - **Overlap**: Can overlap data transfer with computation
    ///
    /// This provides the best performance/safety tradeoff for most use cases.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use std::sync::Arc;
    ///
    /// // Data must remain valid until transfer completes
    /// let data = Arc::new(vec![1.0f32, 2.0, 3.0, 4.0]);
    /// let host_buffer = HostBuffer::from_data((*data).clone(), None, None);
    ///
    /// // Using the async API - awaits done_with_host_buffer automatically
    /// let device_buffer = host_buffer.to(&client).copy().await?;
    ///
    /// // Now safe to drop the Arc - await ensured transfer is complete
    /// drop(data);
    /// ```
    ///
    /// # Guarantees
    ///
    /// | Party | Guarantee |
    /// |-------|-----------|
    /// | **Caller** | Data remains valid and immutable until `done_with_host_buffer` fires |
    /// | **Runtime** | Will fire `done_with_host_buffer` when data access is complete |
    ///
    /// # Default Behavior
    ///
    /// This is the default semantics used by all high-level transfer APIs in pjrt-rs.
    ImmutableUntilTransferCompletes =
        PJRT_HostBufferSemantics_PJRT_HostBufferSemantics_kImmutableUntilTransferCompletes as i32,

    /// The buffer may alias `data` directly (zero-copy) and the runtime won't modify it.
    ///
    /// # Memory Safety
    ///
    /// On CPU backends, the PJRT buffer may directly reference your source data
    /// without copying. You must keep the data valid and unmodified for the
    /// **entire lifetime of the device buffer**.
    ///
    /// The `done_with_host_buffer` event fires when the buffer is destroyed,
    /// not when an individual operation completes.
    ///
    /// **Warning**: Freeing or modifying the data while the buffer exists causes
    /// undefined behavior!
    ///
    /// # When to Use
    ///
    /// - You're using a CPU backend and want to avoid copying large data
    /// - The source data is long-lived (e.g., static data, cached weights)
    /// - You can guarantee the data outlives all device buffers referencing it
    /// - The data is read-only after creation
    ///
    /// # Performance Implications
    ///
    /// - **CPU**: True zero-copy - no data movement, immediate availability
    /// - **GPU/TPU**: Falls back to `ImmutableUntilTransferCompletes` behavior
    /// - **Memory efficient**: No duplicate memory allocation on CPU
    /// - **Cache friendly**: Data stays in place, better cache utilization
    ///
    /// Best performance on CPU for large, long-lived, read-only data.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use std::sync::Arc;
    ///
    /// // Long-lived data that will outlive any buffers
    /// let model_weights: Arc<Vec<f32>> = Arc::new(load_weights()?);
    ///
    /// // Create buffer - on CPU, this directly aliases model_weights!
    /// // No copy is made. The runtime reads directly from your memory.
    /// let weight_buffer = /* create with ImmutableZeroCopy semantics */;
    ///
    /// // Use in multiple computations...
    /// for batch in batches {
    ///     executable.execute(&[batch, &weight_buffer], &[])?;
    /// }
    ///
    /// // Only drop model_weights AFTER weight_buffer is destroyed
    /// drop(weight_buffer);
    /// drop(model_weights); // Now safe
    /// ```
    ///
    /// # Platform-Specific Notes
    ///
    /// | Platform | Behavior |
    /// |----------|----------|
    /// | **CPU** | True zero-copy: buffer directly aliases source data |
    /// | **GPU** | Copies to device memory (same as `ImmutableUntilTransferCompletes`) |
    /// | **TPU** | Copies to device memory (same as `ImmutableUntilTransferCompletes`) |
    ///
    /// # Guarantees
    ///
    /// | Party | Guarantee |
    /// |-------|-----------|
    /// | **Caller** | Data remains valid and immutable for the buffer's entire lifetime |
    /// | **Runtime** | Will not write to data; fires `done_with_host_buffer` when buffer is destroyed |
    ImmutableZeroCopy = PJRT_HostBufferSemantics_PJRT_HostBufferSemantics_kImmutableZeroCopy as i32,

    /// The buffer may alias `data` and the runtime may modify it.
    ///
    /// # Memory Safety
    ///
    /// This is the most complex semantic. The runtime may both read and write
    /// to your source data. You must:
    ///
    /// 1. Keep the data valid for the buffer's entire lifetime
    /// 2. Never read from the data while operations are in progress
    /// 3. Synchronize properly before reading results
    ///
    /// **Warning**: Reading from data while the runtime is writing creates a
    /// data race (undefined behavior)!
    ///
    /// # When to Use
    ///
    /// - **Advanced use case**: You understand PJRT's execution model
    /// - You want in-place output buffers on CPU (avoid copy-back)
    /// - You're implementing custom execution pipelines
    /// - You need maximum performance on CPU with output aliasing
    ///
    /// # Performance Implications
    ///
    /// - **CPU**: True zero-copy with mutation - results written directly to host
    /// - **GPU/TPU**: Falls back to `ImmutableUntilTransferCompletes` behavior
    /// - **No copy-back**: On CPU, results appear directly in your memory
    /// - **Synchronization overhead**: You must manage synchronization carefully
    ///
    /// Maximum performance on CPU for output buffers, but requires careful programming.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// // Pre-allocate buffer for output - runtime will write results here
    /// let mut output_data = vec![0.0f32; 10000];
    ///
    /// // Create buffer with MutableZeroCopy
    /// // On CPU: buffer directly aliases output_data
    /// let output_buffer = /* create with MutableZeroCopy semantics */;
    ///
    /// // Execute - runtime writes directly to output_data on CPU!
    /// executable.execute(&[input], &[&output_buffer])?;
    ///
    /// // CRITICAL: Wait for execution to complete before reading!
    /// output_buffer.ready_event()?.wait()?;
    ///
    /// // Now safe to read the results
    /// println!("Sum: {}", output_data.iter().sum::<f32>());
    ///
    /// // Keep output_data alive until buffer is dropped
    /// drop(output_buffer);
    /// ```
    ///
    /// # Platform-Specific Notes
    ///
    /// | Platform | Behavior |
    /// |----------|----------|
    /// | **CPU** | True zero-copy with mutation: runtime may write to source data |
    /// | **GPU** | Same as `ImmutableUntilTransferCompletes` (no mutation of host data) |
    /// | **TPU** | Same as `ImmutableUntilTransferCompletes` (no mutation of host data) |
    ///
    /// # Guarantees
    ///
    /// | Party | Guarantee |
    /// |-------|-----------|
    /// | **Caller** | Data valid for buffer's lifetime; won't read during execution |
    /// | **Runtime** | May modify data; fires `done_with_host_buffer` when buffer destroyed |
    ///
    /// # Safety Checklist
    ///
    /// Before using `MutableZeroCopy`, ensure:
    ///
    /// - [ ] You understand the PJRT execution model for your platform
    /// - [ ] You have proper synchronization (wait on events before reading)
    /// - [ ] The data will outlive all buffers referencing it
    /// - [ ] You won't read from the data while operations are in flight
    /// - [ ] You're prepared to debug subtle race conditions if misused
    MutableZeroCopy = PJRT_HostBufferSemantics_PJRT_HostBufferSemantics_kMutableZeroCopy as i32,
}

pub trait HostBufferCopyToDest {
    fn client(&self) -> &Client;
    fn set_args(&self, args: &mut PJRT_Client_BufferFromHostBuffer_Args) -> Result<()>;
}

impl HostBufferCopyToDest for Client {
    fn client(&self) -> &Client {
        self
    }

    fn set_args(&self, args: &mut PJRT_Client_BufferFromHostBuffer_Args) -> Result<()> {
        args.device = self
            .addressable_devices()
            .first()
            .ok_or(Error::NoAddressableDevice)?
            .ptr;
        Ok(())
    }
}

impl HostBufferCopyToDest for Device {
    fn client(&self) -> &Client {
        Device::client(self)
    }

    fn set_args(&self, args: &mut PJRT_Client_BufferFromHostBuffer_Args) -> Result<()> {
        args.device = self.ptr;
        Ok(())
    }
}

impl HostBufferCopyToDest for Memory {
    fn client(&self) -> &Client {
        Memory::client(self)
    }

    fn set_args(&self, args: &mut PJRT_Client_BufferFromHostBuffer_Args) -> Result<()> {
        args.memory = self.ptr;
        Ok(())
    }
}
