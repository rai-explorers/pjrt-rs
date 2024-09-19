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
    Type, F32, F64, I16, I32, I64, I8, U16, U32, U64, U8,
};

#[derive(Debug)]
pub struct TypedHostBuffer<T: Type> {
    data: Rc<Vec<T::ElemType>>,
    dims: Vec<i64>,
    layout: MemoryLayout,
}

impl<T: Type> TypedHostBuffer<T> {
    pub fn builder() -> TypedHostBufferBuilder {
        TypedHostBufferBuilder
    }

    pub fn scalar(data: T::ElemType) -> Self {
        let data = vec![data];
        let dims = vec![];
        let layout = MemoryLayout::strides(vec![]);
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
        args.type_ = T::PRIMITIVE_TYPE as PJRT_Buffer_Type;
        args.dims = self.dims.as_ptr();
        args.num_dims = self.dims.len();
        args.host_buffer_semantics =
            HostBufferSemantics::ImmutableUntilTransferCompletes as PJRT_HostBufferSemantics;
        if let Some(byte_strides) = &config.byte_strides {
            args.byte_strides = byte_strides.as_ptr() as *const _;
            args.num_byte_strides = byte_strides.len();
        }
        if let Some(device_layout) = &config.device_layout {
            let mut device_layout = PJRT_Buffer_MemoryLayout::from(device_layout);
            args.device_layout = &mut device_layout as *mut _;
        }
        config.dest.set_args(&mut args)?;
        client.api().PJRT_Client_BufferFromHostBuffer(args)
    }

    pub fn copy_to_sync<D, C>(&self, config: C) -> Result<Buffer>
    where
        D: HostBufferCopyToDest,
        C: IntoHostBufferCopyToConfig<D>,
    {
        let config = config.into_copy_to_config();
        let client = config.dest.client();
        let args = self.call_copy_to(&config)?;
        let done_with_host_event = Event::wrap(client.api(), args.done_with_host_buffer);
        done_with_host_event.wait()?;
        let buf = Buffer::wrap(client, args.buffer);
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
        let done_with_host_event = Event::wrap(client.api(), args.done_with_host_buffer);
        done_with_host_event.await?;
        let buf = Buffer::wrap(client, args.buffer);
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
impl_from_typed_buffer![I8];
impl_from_typed_buffer![I16];
impl_from_typed_buffer![I32];
impl_from_typed_buffer![I64];
impl_from_typed_buffer![U8];
impl_from_typed_buffer![U16];
impl_from_typed_buffer![U32];
impl_from_typed_buffer![U64];

#[derive(Debug)]
pub enum HostBuffer {
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
}

impl HostBuffer {
    pub fn builder() -> HostBufferBuilder {
        HostBufferBuilder
    }

    pub fn scalar<E>(data: E) -> HostBuffer
    where
        E: ElemType,
        Self: From<TypedHostBuffer<E::Type>>,
    {
        let buf = TypedHostBuffer::<E::Type>::scalar(data);
        Self::from(buf)
    }

    pub fn dims(&self) -> &[i64] {
        match self {
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
        }
    }

    pub fn layout(&self) -> &MemoryLayout {
        match self {
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
            Self::I8(buf) => buf.copy_to_sync(config),
            Self::I16(buf) => buf.copy_to_sync(config),
            Self::I32(buf) => buf.copy_to_sync(config),
            Self::I64(buf) => buf.copy_to_sync(config),
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
            Self::I8(buf) => buf.copy_to(config).await,
            Self::I16(buf) => buf.copy_to(config).await,
            Self::I32(buf) => buf.copy_to(config).await,
            Self::I64(buf) => buf.copy_to(config).await,
            Self::U8(buf) => buf.copy_to(config).await,
            Self::U16(buf) => buf.copy_to(config).await,
            Self::U32(buf) => buf.copy_to(config).await,
            Self::U64(buf) => buf.copy_to(config).await,
        }
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

impl<'a> HostBufferCopyToDest for &'a Client {
    fn client(&self) -> &Client {
        *self
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

impl<'a> HostBufferCopyToDest for &'a Device {
    fn client(&self) -> &Client {
        Device::client(*self)
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

impl<'a> HostBufferCopyToDest for &'a Memory {
    fn client(&self) -> &Client {
        Memory::client(*self)
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

#[derive(Debug)]
pub struct TypedHostBufferBuilder;

#[bon]
impl TypedHostBufferBuilder {
    #[builder(finish_fn = build)]
    pub fn data<E>(
        &self,
        #[builder(start_fn, into)] data: Vec<E>,
        #[builder(into)] dims: Option<Vec<i64>>,
        #[builder] layout: Option<MemoryLayout>,
    ) -> TypedHostBuffer<E::Type>
    where
        E: ElemType,
    {
        let data: Vec<E> = data.into();
        let dims = dims.unwrap_or_else(|| vec![data.len() as i64]);
        let layout = layout
            .unwrap_or_else(|| MemoryLayout::strides(utils::byte_strides(&dims, E::Type::SIZE)));
        TypedHostBuffer {
            data: Rc::new(data),
            dims,
            layout,
        }
    }

    #[builder(finish_fn = build)]
    pub fn bytes<T>(
        &self,
        #[builder(start_fn, into)] bytes: Vec<u8>,
        #[builder(into)] dims: Option<Vec<i64>>,
        #[builder] layout: Option<MemoryLayout>,
    ) -> TypedHostBuffer<T>
    where
        T: Type,
    {
        let length = bytes.len() / T::SIZE;
        let capacity = bytes.capacity() / T::SIZE;
        let ptr = bytes.as_ptr() as *mut T::ElemType;
        let data = unsafe { Vec::from_raw_parts(ptr, length, capacity) };
        mem::forget(bytes);
        let dims = dims.unwrap_or_else(|| vec![length as i64]);
        assert!(dims.iter().product::<i64>() == length as i64);
        let layout =
            layout.unwrap_or_else(|| MemoryLayout::strides(utils::byte_strides(&dims, T::SIZE)));
        TypedHostBuffer {
            data: Rc::new(data),
            dims,
            layout,
        }
    }
}

#[derive(Debug)]
pub struct HostBufferBuilder;

#[bon]
impl HostBufferBuilder {
    #[builder(finish_fn = build)]
    pub fn data<E>(
        &self,
        #[builder(start_fn, into)] data: Vec<E>,
        #[builder(into)] dims: Option<Vec<i64>>,
        #[builder] layout: Option<MemoryLayout>,
    ) -> HostBuffer
    where
        E: ElemType,
        HostBuffer: From<TypedHostBuffer<E::Type>>,
    {
        let buf = TypedHostBufferBuilder
            .data::<E>(data)
            .maybe_dims(dims)
            .maybe_layout(layout)
            .build();
        HostBuffer::from(buf)
    }

    #[builder(finish_fn = build)]
    pub fn bytes(
        &self,
        #[builder(start_fn)] bytes: Vec<u8>,
        #[builder(start_fn)] ty: PrimitiveType,
        #[builder(into)] dims: Option<Vec<i64>>,
        #[builder] layout: Option<MemoryLayout>,
    ) -> Result<HostBuffer> {
        match ty {
            PrimitiveType::F32 => Ok(HostBuffer::F32(
                TypedHostBufferBuilder
                    .bytes::<F32>(bytes)
                    .maybe_dims(dims)
                    .maybe_layout(layout)
                    .build(),
            )),
            PrimitiveType::F64 => Ok(HostBuffer::F64(
                TypedHostBufferBuilder
                    .bytes::<F64>(bytes)
                    .maybe_dims(dims)
                    .maybe_layout(layout)
                    .build(),
            )),
            PrimitiveType::S8 => Ok(HostBuffer::I8(
                TypedHostBufferBuilder
                    .bytes::<I8>(bytes)
                    .maybe_dims(dims)
                    .maybe_layout(layout)
                    .build(),
            )),
            PrimitiveType::S16 => Ok(HostBuffer::I16(
                TypedHostBufferBuilder
                    .bytes::<I16>(bytes)
                    .maybe_dims(dims)
                    .maybe_layout(layout)
                    .build(),
            )),
            PrimitiveType::S32 => Ok(HostBuffer::I32(
                TypedHostBufferBuilder
                    .bytes::<I32>(bytes)
                    .maybe_dims(dims)
                    .maybe_layout(layout)
                    .build(),
            )),
            PrimitiveType::S64 => Ok(HostBuffer::I64(
                TypedHostBufferBuilder
                    .bytes::<I64>(bytes)
                    .maybe_dims(dims)
                    .maybe_layout(layout)
                    .build(),
            )),
            PrimitiveType::U8 => Ok(HostBuffer::U8(
                TypedHostBufferBuilder
                    .bytes::<U8>(bytes)
                    .maybe_dims(dims)
                    .maybe_layout(layout)
                    .build(),
            )),
            PrimitiveType::U16 => Ok(HostBuffer::U16(
                TypedHostBufferBuilder
                    .bytes::<U16>(bytes)
                    .maybe_dims(dims)
                    .maybe_layout(layout)
                    .build(),
            )),
            PrimitiveType::U32 => Ok(HostBuffer::U32(
                TypedHostBufferBuilder
                    .bytes::<U32>(bytes)
                    .maybe_dims(dims)
                    .maybe_layout(layout)
                    .build(),
            )),
            PrimitiveType::U64 => Ok(HostBuffer::U64(
                TypedHostBufferBuilder
                    .bytes::<U64>(bytes)
                    .maybe_dims(dims)
                    .maybe_layout(layout)
                    .build(),
            )),
            _ => Err(Error::NotSupportedType(ty)),
        }
    }
}
