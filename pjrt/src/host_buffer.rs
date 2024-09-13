use std::ffi::c_void;
use std::iter::Product;
use std::mem;
use std::rc::Rc;

use pjrt_sys::{
    PJRT_Buffer_MemoryLayout, PJRT_Client_BufferFromHostBuffer_Args,
    PJRT_HostBufferSemantics_PJRT_HostBufferSemantics_kImmutableOnlyDuringCall,
    PJRT_HostBufferSemantics_PJRT_HostBufferSemantics_kImmutableUntilTransferCompletes,
    PJRT_HostBufferSemantics_PJRT_HostBufferSemantics_kImmutableZeroCopy,
    PJRT_HostBufferSemantics_PJRT_HostBufferSemantics_kMutableZeroCopy,
};

use crate::event::Event;
use crate::{
    utils, Buffer, Client, Device, ElemType, Error, Memory, MemoryLayout, PrimitiveType, Result,
    Shape, Type, F32, F64, S16, S32, S64, S8, U16, U32, U64, U8,
};

#[derive(Debug)]
pub struct TypedHostBuffer<T: Type> {
    data: Rc<Vec<T::ElemType>>,
    shape: Vec<i64>,
    layout: MemoryLayout,
}

impl<T: Type> TypedHostBuffer<T> {
    pub fn new(data: impl Into<Vec<T::ElemType>>, shape: impl Shape) -> Self {
        let data: Vec<T::ElemType> = data.into();
        let shape = shape.shape().to_vec();
        let layout = MemoryLayout::from_strides(utils::byte_strides(&shape, T::SIZE));
        Self {
            data: Rc::new(data),
            shape,
            layout,
        }
    }

    pub fn scalar(data: T::ElemType) -> Self {
        let data = vec![data];
        let shape = vec![];
        let layout = MemoryLayout::from_strides(vec![]);
        Self {
            data: Rc::new(data),
            shape,
            layout,
        }
    }

    pub fn from_bytes(bytes: Vec<u8>, shape: impl Shape) -> Self {
        let length = bytes.len() / T::SIZE;
        let capacity = bytes.capacity() / T::SIZE;
        let ptr = bytes.as_ptr() as *mut T::ElemType;
        let data = unsafe { Vec::from_raw_parts(ptr, length, capacity) };
        mem::forget(bytes);
        let shape = shape.shape().to_vec();
        assert!(shape.iter().product::<i64>() == length as i64);
        let layout = MemoryLayout::from_strides(utils::byte_strides(&shape, T::SIZE));
        Self {
            data: Rc::new(data),
            shape,
            layout,
        }
    }

    pub fn from_bytes_with_layout(bytes: Vec<u8>, shape: impl Shape, layout: MemoryLayout) -> Self {
        let length = bytes.len() / T::SIZE;
        let capacity = bytes.capacity() / T::SIZE;
        let ptr = bytes.as_ptr() as *mut T::ElemType;
        let data = unsafe { Vec::from_raw_parts(ptr, length, capacity) };
        mem::forget(bytes);
        let shape = shape.shape().to_vec();
        assert!(shape.iter().product::<i64>() == length as i64);
        Self {
            data: Rc::new(data),
            shape,
            layout,
        }
    }

    pub fn call_copy_to<D>(
        &self,
        config: &HostBufferCopyToConfig<D>,
    ) -> Result<PJRT_Client_BufferFromHostBuffer_Args>
    where
        D: HostBufferCopyToDest,
    {
        let client = config.dest.client();
        let mut args = PJRT_Client_BufferFromHostBuffer_Args::new();
        args.client = client.ptr();
        args.data = self.data.as_ptr() as *const c_void;
        args.type_ = T::PRIMITIVE_TYPE as u32;
        args.dims = self.shape.as_ptr() as *const i64;
        args.num_dims = self.shape.len();
        args.host_buffer_semantics = HostBufferSemantics::ImmutableUntilTransferCompletes as u32;
        if let Some(byte_strides) = &config.byte_strides {
            args.byte_strides = byte_strides.as_ptr() as *const _;
            args.num_byte_strides = byte_strides.len();
        }
        if let Some(device_layout) = &config.device_layout {
            let mut device_layout = PJRT_Buffer_MemoryLayout::from(device_layout);
            args.device_layout = &mut device_layout as *mut _;
        }
        config.dest.set_args(&mut args)?;
        unsafe { client.api().PJRT_Client_BufferFromHostBuffer(args) }
    }

    pub fn copy_to_sync<D, C>(&self, config: C) -> Result<Buffer>
    where
        D: HostBufferCopyToDest,
        C: IntoHostBufferCopyToConfig<D>,
    {
        let config = config.into_copy_to_config();
        let client = config.dest.client();
        let args = self.call_copy_to(&config)?;
        let done_with_host_event = Event::new(client.api(), args.done_with_host_buffer);
        done_with_host_event.wait()?;
        let buf = Buffer::new(client, args.buffer);
        let buf_ready_event = buf.ready_event()?;
        buf_ready_event.wait()?;
        Ok(buf)
    }

    pub async fn copy_to<D, C>(&self, config: C) -> Result<Buffer>
    where
        D: HostBufferCopyToDest,
        C: IntoHostBufferCopyToConfig<D>,
    {
        let config = config.into_copy_to_config();
        let client = config.dest.client();
        let args = self.call_copy_to(&config)?;
        let done_with_host_event = Event::new(client.api(), args.done_with_host_buffer);
        done_with_host_event.await?;
        let buf = Buffer::new(client, args.buffer);
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

impl_from_typed_buffer!(F32);
impl_from_typed_buffer![F64];
impl_from_typed_buffer![S8];
impl_from_typed_buffer![S16];
impl_from_typed_buffer![S32];
impl_from_typed_buffer![S64];
impl_from_typed_buffer![U8];
impl_from_typed_buffer![U16];
impl_from_typed_buffer![U32];
impl_from_typed_buffer![U64];

#[derive(Debug)]
pub enum HostBuffer {
    F32(TypedHostBuffer<F32>),
    F64(TypedHostBuffer<F64>),
    S8(TypedHostBuffer<S8>),
    S16(TypedHostBuffer<S16>),
    S32(TypedHostBuffer<S32>),
    S64(TypedHostBuffer<S64>),
    U8(TypedHostBuffer<U8>),
    U16(TypedHostBuffer<U16>),
    U32(TypedHostBuffer<U32>),
    U64(TypedHostBuffer<U64>),
}

impl HostBuffer {
    pub fn new<T>(data: impl Into<Vec<T>>, shape: impl Shape) -> Self
    where
        T: ElemType,
        Self: From<TypedHostBuffer<T::Type>>,
    {
        let buf = TypedHostBuffer::<T::Type>::new(data, shape);
        Self::from(buf)
    }

    pub fn scalar<T>(data: T) -> Self
    where
        T: ElemType,
        Self: From<TypedHostBuffer<T::Type>>,
    {
        let buf = TypedHostBuffer::<T::Type>::scalar(data);
        Self::from(buf)
    }

    pub fn from_bytes(data: Vec<u8>, ty: PrimitiveType, shape: impl Shape) -> Result<Self> {
        match ty {
            PrimitiveType::F32 => Ok(Self::F32(TypedHostBuffer::from_bytes(data, shape))),
            PrimitiveType::F64 => Ok(Self::F64(TypedHostBuffer::from_bytes(data, shape))),
            PrimitiveType::S8 => Ok(Self::S8(TypedHostBuffer::from_bytes(data, shape))),
            _ => Err(Error::NotSupportedType(ty)),
        }
    }

    pub fn from_bytes_with_layout(
        data: Vec<u8>,
        ty: PrimitiveType,
        shape: impl Shape,
        layout: MemoryLayout,
    ) -> Result<Self> {
        match ty {
            PrimitiveType::F32 => Ok(Self::F32(TypedHostBuffer::from_bytes_with_layout(
                data, shape, layout,
            ))),
            PrimitiveType::F64 => Ok(Self::F64(TypedHostBuffer::from_bytes_with_layout(
                data, shape, layout,
            ))),
            PrimitiveType::S8 => Ok(Self::S8(TypedHostBuffer::from_bytes_with_layout(
                data, shape, layout,
            ))),
            _ => Err(Error::NotSupportedType(ty)),
        }
    }

    pub fn copy_to_sync<D, C>(&self, config: C) -> Result<Buffer>
    where
        D: HostBufferCopyToDest,
        C: IntoHostBufferCopyToConfig<D>,
    {
        match self {
            Self::F32(buf) => buf.copy_to_sync(config),
            Self::F64(buf) => buf.copy_to_sync(config),
            Self::S8(buf) => buf.copy_to_sync(config),
            Self::S16(buf) => buf.copy_to_sync(config),
            Self::S32(buf) => buf.copy_to_sync(config),
            Self::S64(buf) => buf.copy_to_sync(config),
            Self::U8(buf) => buf.copy_to_sync(config),
            Self::U16(buf) => buf.copy_to_sync(config),
            Self::U32(buf) => buf.copy_to_sync(config),
            Self::U64(buf) => buf.copy_to_sync(config),
        }
    }

    pub async fn copy_to<D, C>(&self, config: C) -> Result<Buffer>
    where
        D: HostBufferCopyToDest,
        C: IntoHostBufferCopyToConfig<D>,
    {
        match self {
            Self::F32(buf) => buf.copy_to(config).await,
            Self::F64(buf) => buf.copy_to(config).await,
            Self::S8(buf) => buf.copy_to(config).await,
            Self::S16(buf) => buf.copy_to(config).await,
            Self::S32(buf) => buf.copy_to(config).await,
            Self::S64(buf) => buf.copy_to(config).await,
            Self::U8(buf) => buf.copy_to(config).await,
            Self::U16(buf) => buf.copy_to(config).await,
            Self::U32(buf) => buf.copy_to(config).await,
            Self::U64(buf) => buf.copy_to(config).await,
        }
    }
}

#[repr(u32)]
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum HostBufferSemantics {
    /// The runtime may not hold references to `data` after the call to
    /// `PJRT_Client_BufferFromHostBuffer` completes. The caller promises that
    /// `data` is immutable and will not be freed only for the duration of the
    /// PJRT_Client_BufferFromHostBuffer call.
    ImmutableOnlyDuringCall =
        PJRT_HostBufferSemantics_PJRT_HostBufferSemantics_kImmutableOnlyDuringCall,

    /// The runtime may hold onto `data` after the call to
    /// `PJRT_Client_BufferFromHostBuffer`
    /// returns while the runtime completes a transfer to the device. The caller
    /// promises not to mutate or free `data` until the transfer completes, at
    /// which point `done_with_host_buffer` will be triggered.
    ImmutableUntilTransferCompletes =
        PJRT_HostBufferSemantics_PJRT_HostBufferSemantics_kImmutableUntilTransferCompletes,

    /// The PjRtBuffer may alias `data` internally and the runtime may use the
    /// `data` contents as long as the buffer is alive. The runtime promises not
    /// to mutate contents of the buffer (i.e. it will not use it for aliased
    /// output buffers). The caller promises to keep `data` alive and not to mutate
    /// its contents as long as the buffer is alive; to notify the caller that the
    /// buffer may be freed, the runtime will call `done_with_host_buffer` when the
    /// PjRtBuffer is freed.
    ImmutableZeroCopy = PJRT_HostBufferSemantics_PJRT_HostBufferSemantics_kImmutableZeroCopy,

    /// The PjRtBuffer may alias `data` internally and the runtime may use the
    /// `data` contents as long as the buffer is alive. The runtime is allowed
    /// to mutate contents of the buffer (i.e. use it for aliased output
    /// buffers). The caller promises to keep `data` alive and not to mutate its
    /// contents as long as the buffer is alive (otherwise it could be a data
    /// race with the runtime); to notify the caller that the buffer may be
    /// freed, the runtime will call `on_done_with_host_buffer` when the
    /// PjRtBuffer is freed. On non-CPU platforms this acts identically to
    /// kImmutableUntilTransferCompletes.
    MutableZeroCopy = PJRT_HostBufferSemantics_PJRT_HostBufferSemantics_kMutableZeroCopy,
}

pub trait HostBufferCopyToDest {
    fn client(&self) -> &Client;
    fn set_args(&self, args: &mut PJRT_Client_BufferFromHostBuffer_Args) -> Result<()>;
}

impl HostBufferCopyToDest for Client {
    fn client(&self) -> &Client {
        &self
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

impl<'a> HostBufferCopyToDest for &'a Client {
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
        &self.client
    }

    fn set_args(&self, args: &mut PJRT_Client_BufferFromHostBuffer_Args) -> Result<()> {
        args.device = self.ptr;
        Ok(())
    }
}

impl<'a> HostBufferCopyToDest for &'a Device {
    fn client(&self) -> &Client {
        &self.client
    }

    fn set_args(&self, args: &mut PJRT_Client_BufferFromHostBuffer_Args) -> Result<()> {
        args.device = self.ptr;
        Ok(())
    }
}

impl HostBufferCopyToDest for Memory {
    fn client(&self) -> &Client {
        &self.client
    }

    fn set_args(&self, args: &mut PJRT_Client_BufferFromHostBuffer_Args) -> Result<()> {
        args.memory = self.ptr;
        Ok(())
    }
}

impl<'a> HostBufferCopyToDest for &'a Memory {
    fn client(&self) -> &Client {
        &self.client
    }

    fn set_args(&self, args: &mut PJRT_Client_BufferFromHostBuffer_Args) -> Result<()> {
        args.memory = self.ptr;
        Ok(())
    }
}

pub struct HostBufferCopyToConfig<D>
where
    D: HostBufferCopyToDest,
{
    dest: D,
    byte_strides: Option<Vec<i64>>,
    device_layout: Option<MemoryLayout>,
}

impl<D> HostBufferCopyToConfig<D>
where
    D: HostBufferCopyToDest,
{
    pub fn new(dest: D) -> Self {
        Self {
            dest,
            byte_strides: None,
            device_layout: None,
        }
    }

    pub fn byte_strides(mut self, byte_strides: Vec<i64>) -> Self {
        self.byte_strides = Some(byte_strides);
        self
    }

    pub fn device_layout(mut self, device_layout: MemoryLayout) -> Self {
        self.device_layout = Some(device_layout);
        self
    }
}

mod private {
    use crate::host_buffer::{HostBufferCopyToConfig, HostBufferCopyToDest};
    use crate::MemoryLayout;

    pub trait Argument {
        type Repr;
    }

    pub trait ToConfig<A, D>
    where
        D: HostBufferCopyToDest,
    {
        fn into_config(self) -> HostBufferCopyToConfig<D>;
    }

    impl<D> Argument for D
    where
        D: HostBufferCopyToDest,
    {
        type Repr = (D,);
    }

    impl<D> ToConfig<(D,), D> for D
    where
        D: HostBufferCopyToDest,
    {
        fn into_config(self) -> HostBufferCopyToConfig<D> {
            HostBufferCopyToConfig::new(self)
        }
    }

    impl<D, B> Argument for (D, B)
    where
        D: HostBufferCopyToDest,
        B: Into<Vec<i64>>,
    {
        type Repr = (D, B);
    }

    impl<D, B> ToConfig<(D, B), D> for (D, B)
    where
        D: HostBufferCopyToDest,
        B: Into<Vec<i64>>,
    {
        fn into_config(self) -> HostBufferCopyToConfig<D> {
            HostBufferCopyToConfig::new(self.0).byte_strides(self.1.into())
        }
    }

    impl<D> Argument for (D, MemoryLayout)
    where
        D: HostBufferCopyToDest,
    {
        type Repr = (D, MemoryLayout);
    }

    impl<D> ToConfig<(D, MemoryLayout), D> for (D, MemoryLayout)
    where
        D: HostBufferCopyToDest,
    {
        fn into_config(self) -> HostBufferCopyToConfig<D> {
            HostBufferCopyToConfig::new(self.0).device_layout(self.1)
        }
    }

    impl<'a, D> Argument for (D, &'a MemoryLayout)
    where
        D: HostBufferCopyToDest,
    {
        type Repr = (D, &'a MemoryLayout);
    }

    impl<'a, D> ToConfig<(D, &'a MemoryLayout), D> for (D, &'a MemoryLayout)
    where
        D: HostBufferCopyToDest,
    {
        fn into_config(self) -> HostBufferCopyToConfig<D> {
            HostBufferCopyToConfig::new(self.0).device_layout(self.1.clone())
        }
    }

    impl<D, B, M> Argument for (D, B, M)
    where
        D: HostBufferCopyToDest,
        B: Into<Vec<i64>>,
        M: Into<MemoryLayout>,
    {
        type Repr = (D, B, M);
    }

    impl<D, B, M> ToConfig<(D, B, M), D> for (D, B, M)
    where
        D: HostBufferCopyToDest,
        B: Into<Vec<i64>>,
        M: Into<MemoryLayout>,
    {
        fn into_config(self) -> HostBufferCopyToConfig<D> {
            HostBufferCopyToConfig::new(self.0)
                .byte_strides(self.1.into())
                .device_layout(self.2.into())
        }
    }
}

pub trait IntoHostBufferCopyToConfig<D>
where
    D: HostBufferCopyToDest,
{
    fn into_copy_to_config(self) -> HostBufferCopyToConfig<D>;
}

impl<T, D> IntoHostBufferCopyToConfig<D> for T
where
    T: private::Argument + private::ToConfig<T::Repr, D>,
    D: HostBufferCopyToDest,
{
    fn into_copy_to_config(self) -> HostBufferCopyToConfig<D> {
        self.into_config()
    }
}
