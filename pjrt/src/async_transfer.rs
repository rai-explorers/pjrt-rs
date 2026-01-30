use std::ffi::c_void;

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
    Buffer, Client, Device, ErrorCode, Event, MemoryLayout, NamedValue, PrimitiveType, Result,
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
    pub fn device(&self) -> Device {
        let mut args = PJRT_AsyncHostToDeviceTransferManager_Device_Args::new();
        args.transfer_manager = self.ptr;
        args = self
            .client
            .api()
            .PJRT_AsyncHostToDeviceTransferManager_Device(args)
            .expect("PJRT_AsyncHostToDeviceTransferManager_Device");
        Device::wrap(&self.client, args.device_out)
    }

    /// Returns the number of buffers managed by this transfer manager.
    pub fn buffer_count(&self) -> usize {
        let mut args = PJRT_AsyncHostToDeviceTransferManager_BufferCount_Args::new();
        args.transfer_manager = self.ptr;
        args = self
            .client
            .api()
            .PJRT_AsyncHostToDeviceTransferManager_BufferCount(args)
            .expect("PJRT_AsyncHostToDeviceTransferManager_BufferCount");
        args.buffer_count
    }

    /// Returns the size (in bytes) of the buffer at the given index.
    pub fn buffer_size(&self, buffer_index: i32) -> usize {
        let mut args = PJRT_AsyncHostToDeviceTransferManager_BufferSize_Args::new();
        args.transfer_manager = self.ptr;
        args.buffer_index = buffer_index;
        args = self
            .client
            .api()
            .PJRT_AsyncHostToDeviceTransferManager_BufferSize(args)
            .expect("PJRT_AsyncHostToDeviceTransferManager_BufferSize");
        args.buffer_size
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
        args.transfer_metadata = metadata.as_ptr() as *const pjrt_sys::PJRT_NamedValue;
        args.num_metadata = metadata.len();
        self.client
            .api()
            .PJRT_AsyncHostToDeviceTransferManager_AddMetadata(args)
            .map(|_| ())
    }
}

/// Specifies the shape of a buffer to be created.
pub struct BufferShape {
    dims: Vec<i64>,
    element_type: PrimitiveType,
    layout: Option<MemoryLayout>,
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
