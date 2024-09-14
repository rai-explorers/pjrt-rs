use pjrt_sys::{
    PJRT_Buffer, PJRT_Buffer_CopyToDevice_Args, PJRT_Buffer_CopyToMemory_Args,
    PJRT_Buffer_Delete_Args, PJRT_Buffer_Destroy_Args, PJRT_Buffer_Device_Args,
    PJRT_Buffer_Dimensions_Args, PJRT_Buffer_DynamicDimensionIndices_Args,
    PJRT_Buffer_ElementType_Args, PJRT_Buffer_GetMemoryLayout_Args, PJRT_Buffer_IsDeleted_Args,
    PJRT_Buffer_IsOnCpu_Args, PJRT_Buffer_Memory_Args, PJRT_Buffer_OnDeviceSizeInBytes_Args,
    PJRT_Buffer_ReadyEvent_Args, PJRT_Buffer_ToHostBuffer_Args,
    PJRT_Buffer_UnpaddedDimensions_Args,
};

use crate::event::Event;
use crate::{Client, Device, HostBuffer, Memory, MemoryLayout, PrimitiveType, Result};

pub struct Buffer {
    pub(crate) client: Client,
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

impl Buffer {
    pub fn new(client: &Client, ptr: *mut PJRT_Buffer) -> Self {
        assert!(!ptr.is_null());
        Self {
            client: client.clone(),
            ptr,
        }
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

    pub fn dimensions(&self) -> Vec<i64> {
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
        s.iter().map(|s| *s).collect()
    }

    pub fn unpadded_dimensions(&self) -> Vec<i64> {
        let mut args = PJRT_Buffer_UnpaddedDimensions_Args::new();
        args.buffer = self.ptr;
        args = self
            .client
            .api()
            .PJRT_Buffer_UnpaddedDimensions(args)
            .expect("PJRT_Buffer_UnpaddedDimensions");
        let s = unsafe { std::slice::from_raw_parts(args.unpadded_dims, args.num_dims) };
        s.iter().map(|s| *s).collect()
    }

    pub fn dynamic_dimension_indices(&self) -> Vec<usize> {
        let mut args = PJRT_Buffer_DynamicDimensionIndices_Args::new();
        args.buffer = self.ptr;
        args = self
            .client
            .api()
            .PJRT_Buffer_DynamicDimensionIndices(args)
            .expect("PJRT_Buffer_DynamicDimensionIndices");
        let s =
            unsafe { std::slice::from_raw_parts(args.dynamic_dim_indices, args.num_dynamic_dims) };
        s.iter().map(|s| *s).collect()
    }

    // deprecated, use layout extension
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
        Device::new(&self.client, args.device)
    }

    pub fn memory(&self) -> Memory {
        let mut args = PJRT_Buffer_Memory_Args::new();
        args.buffer = self.ptr;
        args = self
            .client
            .api()
            .PJRT_Buffer_Memory(args)
            .expect("PJRT_Buffer_Memory");
        Memory::new(&self.client, args.memory)
    }

    pub fn delete(&self) {
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
        Ok(Event::new(self.client.api(), args.event))
    }

    fn call_copy_to_device(&self, device: &Device) -> Result<PJRT_Buffer_CopyToDevice_Args> {
        let mut args = PJRT_Buffer_CopyToDevice_Args::new();
        args.buffer = self.ptr;
        args.dst_device = device.ptr;
        self.client.api().PJRT_Buffer_CopyToDevice(args)
    }

    pub async fn copy_to_device(&self, device: &Device) -> Result<Buffer> {
        let args = self.call_copy_to_device(device)?;
        let buf = Buffer::new(&device.client, args.dst_buffer);
        let event = buf.ready_event()?;
        event.await?;
        Ok(buf)
    }

    pub fn copy_to_device_sync(&self, device: &Device) -> Result<Buffer> {
        let args = self.call_copy_to_device(device)?;
        let buf = Buffer::new(&device.client, args.dst_buffer);
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

    pub async fn copy_to_memory(&self, memory: &Memory) -> Result<Buffer> {
        let args = self.call_copy_to_memory(memory)?;
        let buf = Buffer::new(&memory.client, args.dst_buffer);
        let event = buf.ready_event()?;
        event.await?;
        Ok(buf)
    }

    pub fn copy_to_memory_sync(&self, memory: &Memory) -> Result<Buffer> {
        let args = self.call_copy_to_memory(memory)?;
        let buf = Buffer::new(&memory.client, args.dst_buffer);
        let event = buf.ready_event()?;
        event.wait()?;
        Ok(buf)
    }

    pub fn call_copy_to_host(&self) -> Result<(PJRT_Buffer_ToHostBuffer_Args, Vec<u8>)> {
        let mut args = PJRT_Buffer_ToHostBuffer_Args::new();
        args.src = self.ptr;
        // first call to get the size of the buffer
        args = self.client.api().PJRT_Buffer_ToHostBuffer(args)?;
        let buf_size = args.dst_size;
        // second call to fill the buffer
        let mut buf: Vec<u8> = vec![0; buf_size];
        args.dst = buf.as_mut_ptr() as *mut _;
        args = self.client.api().PJRT_Buffer_ToHostBuffer(args)?;
        Ok((args, buf))
    }

    pub async fn copy_to_host(&self) -> Result<HostBuffer> {
        let (args, data) = self.call_copy_to_host()?;
        let event = Event::new(self.client.api(), args.event);
        event.await?;
        let ty = self.primitive_type();
        let shape = self.dimensions();
        let layout = self.layout();
        HostBuffer::from_bytes_with_layout(data, ty, shape, layout)
    }

    pub fn copy_to_host_sync(&self) -> Result<HostBuffer> {
        let (args, data) = self.call_copy_to_host()?;
        let event = Event::new(self.client.api(), args.event);
        event.wait()?;
        let ty = self.primitive_type();
        let shape = self.dimensions();
        let layout = self.layout();
        HostBuffer::from_bytes_with_layout(data, ty, shape, layout)
    }
}
