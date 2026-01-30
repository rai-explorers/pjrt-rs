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

#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[allow(dead_code)]
pub enum HostBufferSemantics {
    /// The runtime may not hold references to `data` after the call to
    /// `PJRT_Client_BufferFromHostBuffer` completes. The caller promises that
    /// `data` is immutable and will not be freed only for the duration of the
    /// PJRT_Client_BufferFromHostBuffer call.
    ImmutableOnlyDuringCall =
        PJRT_HostBufferSemantics_PJRT_HostBufferSemantics_kImmutableOnlyDuringCall as i32,

    /// The runtime may hold onto `data` after the call to
    /// `PJRT_Client_BufferFromHostBuffer`
    /// returns while the runtime completes a transfer to the device. The caller
    /// promises not to mutate or free `data` until the transfer completes, at
    /// which point `done_with_host_buffer` will be triggered.
    ImmutableUntilTransferCompletes =
        PJRT_HostBufferSemantics_PJRT_HostBufferSemantics_kImmutableUntilTransferCompletes as i32,

    /// The PjRtBuffer may alias `data` internally and the runtime may use the
    /// `data` contents as long as the buffer is alive. The runtime promises not
    /// to mutate contents of the buffer (i.e. it will not use it for aliased
    /// output buffers). The caller promises to keep `data` alive and not to mutate
    /// its contents as long as the buffer is alive; to notify the caller that the
    /// buffer may be freed, the runtime will call `done_with_host_buffer` when the
    /// PjRtBuffer is freed.
    ImmutableZeroCopy = PJRT_HostBufferSemantics_PJRT_HostBufferSemantics_kImmutableZeroCopy as i32,

    /// The PjRtBuffer may alias `data` internally and the runtime may use the
    /// `data` contents as long as the buffer is alive. The runtime is allowed
    /// to mutate contents of the buffer (i.e. use it for aliased output
    /// buffers). The caller promises to keep `data` alive and not to mutate its
    /// contents as long as the buffer is alive (otherwise it could be a data
    /// race with the runtime); to notify the caller that the buffer may be
    /// freed, the runtime will call `on_done_with_host_buffer` when the
    /// PjRtBuffer is freed. On non-CPU platforms this acts identically to
    /// kImmutableUntilTransferCompletes.
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
