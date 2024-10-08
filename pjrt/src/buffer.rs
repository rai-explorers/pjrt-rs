use bon::bon;
use pjrt_sys::{
    PJRT_Buffer, PJRT_Buffer_CopyToDevice_Args, PJRT_Buffer_CopyToMemory_Args,
    PJRT_Buffer_Delete_Args, PJRT_Buffer_Destroy_Args, PJRT_Buffer_Device_Args,
    PJRT_Buffer_Dimensions_Args, PJRT_Buffer_DynamicDimensionIndices_Args,
    PJRT_Buffer_ElementType_Args, PJRT_Buffer_GetMemoryLayout_Args, PJRT_Buffer_IsDeleted_Args,
    PJRT_Buffer_IsOnCpu_Args, PJRT_Buffer_MemoryLayout, PJRT_Buffer_Memory_Args,
    PJRT_Buffer_OnDeviceSizeInBytes_Args, PJRT_Buffer_ReadyEvent_Args,
    PJRT_Buffer_ToHostBuffer_Args, PJRT_Buffer_UnpaddedDimensions_Args,
};

use crate::event::Event;
use crate::{Client, Device, HostBuffer, Memory, MemoryLayout, PrimitiveType, Result};

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

    #[builder(finish_fn = copy)]
    pub async fn to_host(&self, host_layout: Option<MemoryLayout>) -> Result<HostBuffer> {
        let (args, data) = self.call_copy_to_host(host_layout.as_ref())?;
        let event = Event::wrap(self.client.api(), args.event);
        event.await?;
        let ty = self.primitive_type();
        let dims = self.dims();
        let layout = host_layout.unwrap_or_else(|| self.layout());
        HostBuffer::from_bytes(data, ty)
            .dims(dims)
            .layout(layout)
            .build()
    }

    #[builder(finish_fn = copy)]
    pub fn to_host_sync(&self, host_layout: Option<MemoryLayout>) -> Result<HostBuffer> {
        let (args, data) = self.call_copy_to_host(host_layout.as_ref())?;
        let event = Event::wrap(self.client.api(), args.event);
        event.wait()?;
        let ty = self.primitive_type();
        let dims = self.dims();
        let layout = host_layout.unwrap_or_else(|| self.layout());
        HostBuffer::from_bytes(data, ty)
            .dims(dims)
            .layout(layout)
            .build()
    }

    // TODO:
    // PJRT_Buffer_UnsafePointer
    // PJRT_Buffer_IncreaseExternalReferenceCount
    // PJRT_Buffer_DecreaseExternalReferenceCount
    // PJRT_Buffer_OpaqueDeviceMemoryDataPointer
}
