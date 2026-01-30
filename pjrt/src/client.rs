use std::borrow::Cow;
use std::ffi::c_void;
use std::rc::Rc;
use std::slice;

use bon::bon;
use pjrt_sys::{
    PJRT_Client, PJRT_Client_AddressableDevices_Args, PJRT_Client_AddressableMemories_Args,
    PJRT_Client_Compile_Args, PJRT_Client_CreateAliasBuffer_Args,
    PJRT_Client_CreateBuffersForAsyncHostToDevice_Args, PJRT_Client_CreateErrorBuffer_Args,
    PJRT_Client_CreateUninitializedBuffer_Args, PJRT_Client_DefaultDeviceAssignment_Args,
    PJRT_Client_Destroy_Args, PJRT_Client_Devices_Args, PJRT_Client_DmaMap_Args,
    PJRT_Client_DmaUnmap_Args, PJRT_Client_FulfillAliasBuffer_Args,
    PJRT_Client_LookupAddressableDevice_Args, PJRT_Client_LookupDevice_Args,
    PJRT_Client_PlatformName_Args, PJRT_Client_PlatformVersion_Args, PJRT_Client_ProcessIndex_Args,
    PJRT_Client_TopologyDescription_Args, PJRT_Client_UpdateGlobalProcessInfo_Args,
    PJRT_Error_Code, PJRT_Executable_DeserializeAndLoad_Args, PJRT_ProcessInfo, PJRT_Program,
    PJRT_ShapeSpec,
};

use crate::{
    utils, Api, AsyncHostToDeviceTransferManager, Buffer, BufferShape, CompileOptions,
    CompileToLoadedExecutable, Device, DeviceAssignment, ErrorCode, GlobalDeviceId, KeyValueStore,
    LoadedExecutable, LocalHardwareId, Memory, MemoryLayout, NamedValue, PrimitiveType, Program,
    Result, TopologyDescription,
};

struct ClientRaw {
    api: Api,
    ptr: *mut PJRT_Client,
}

impl Drop for ClientRaw {
    fn drop(&mut self) {
        let mut args = PJRT_Client_Destroy_Args::new();
        args.client = self.ptr;
        self.api
            .PJRT_Client_Destroy(args)
            .expect("PJRT_Client_Destroy");
    }
}

#[derive(Clone)]
pub struct Client {
    raw: Rc<ClientRaw>,
}

#[bon]
impl Client {
    pub(crate) fn wrap(api: &Api, ptr: *mut PJRT_Client) -> Self {
        assert!(!ptr.is_null());
        Self {
            raw: Rc::new(ClientRaw {
                api: api.clone(),
                ptr,
            }),
        }
    }

    #[allow(clippy::borrowed_box)]
    #[builder(finish_fn = build)]
    pub fn builder(
        #[builder(start_fn)] api: &Api,
        #[builder(default = bon::vec![], into)] options: Vec<NamedValue>,
        kv_store: Option<&Box<dyn KeyValueStore>>,
    ) -> Result<Self> {
        api.create_client(options, kv_store)
    }

    pub fn api(&self) -> &Api {
        &self.raw.api
    }

    pub(crate) fn ptr(&self) -> *mut PJRT_Client {
        self.raw.ptr
    }

    pub fn platform_name(&self) -> Cow<'_, str> {
        let mut args = PJRT_Client_PlatformName_Args::new();
        args.client = self.ptr();
        args = self
            .api()
            .PJRT_Client_PlatformName(args)
            .expect("PJRT_Client_PlatformName");
        utils::str_from_raw(args.platform_name, args.platform_name_size)
    }

    pub fn platform_version(&self) -> Cow<'_, str> {
        let mut args = PJRT_Client_PlatformVersion_Args::new();
        args.client = self.ptr();
        args = self
            .api()
            .PJRT_Client_PlatformVersion(args)
            .expect("PJRT_Client_PlatformVersion");
        utils::str_from_raw(args.platform_version, args.platform_version_size)
    }

    pub fn process_index(&self) -> i32 {
        let mut args = PJRT_Client_ProcessIndex_Args::new();
        args.client = self.ptr();
        args = self
            .api()
            .PJRT_Client_ProcessIndex(args)
            .expect("PJRT_Client_ProcessIndex");
        args.process_index
    }

    pub fn devices(&self) -> Vec<Device> {
        let mut args = PJRT_Client_Devices_Args::new();
        args.client = self.ptr();
        args = self
            .api()
            .PJRT_Client_Devices(args)
            .expect("PJRT_Client_Devices");
        let raw_devices = unsafe { slice::from_raw_parts(args.devices, args.num_devices) };
        raw_devices
            .iter()
            .cloned()
            .map(|d| Device::wrap(self, d))
            .collect()
    }

    pub fn addressable_devices(&self) -> Vec<Device> {
        let mut args = PJRT_Client_AddressableDevices_Args::new();
        args.client = self.ptr();
        args = self
            .api()
            .PJRT_Client_AddressableDevices(args)
            .expect("PJRT_Client_AddressableDevices");
        let devices = unsafe {
            slice::from_raw_parts(args.addressable_devices, args.num_addressable_devices)
        };
        devices
            .iter()
            .cloned()
            .map(|d| Device::wrap(self, d))
            .collect()
    }

    pub fn addressable_memories(&self) -> Vec<Memory> {
        let mut args = PJRT_Client_AddressableMemories_Args::new();
        args.client = self.ptr();
        args = self
            .api()
            .PJRT_Client_AddressableMemories(args)
            .expect("PJRT_Client_AddressableMemories");
        let memories = unsafe {
            slice::from_raw_parts(args.addressable_memories, args.num_addressable_memories)
        };
        memories
            .iter()
            .cloned()
            .map(|d| Memory::wrap(self, d))
            .collect()
    }

    pub fn lookup_device(&self, global_device_id: GlobalDeviceId) -> Result<Device> {
        let mut args = PJRT_Client_LookupDevice_Args::new();
        args.client = self.ptr();
        args.id = global_device_id;
        args = self.api().PJRT_Client_LookupDevice(args)?;
        Ok(Device::wrap(self, args.device))
    }

    pub fn lookup_addressable_device(&self, local_hardware_id: LocalHardwareId) -> Result<Device> {
        let mut args = PJRT_Client_LookupAddressableDevice_Args::new();
        args.client = self.ptr();
        args.local_hardware_id = local_hardware_id;
        args = self.api().PJRT_Client_LookupAddressableDevice(args)?;
        Ok(Device::wrap(self, args.addressable_device))
    }

    pub fn compile<T>(&self, program: &T, options: CompileOptions) -> Result<LoadedExecutable>
    where
        Self: CompileToLoadedExecutable<T>,
    {
        CompileToLoadedExecutable::<T>::compile(self, program, &options)
    }

    pub fn load_executable(&self, bytes: &[u8]) -> Result<LoadedExecutable> {
        let mut args = PJRT_Executable_DeserializeAndLoad_Args::new();
        args.client = self.ptr();
        args.serialized_executable = bytes.as_ptr() as *const i8;
        args.serialized_executable_size = bytes.len();
        args = self.api().PJRT_Executable_DeserializeAndLoad(args)?;
        Ok(LoadedExecutable::wrap(self, args.loaded_executable))
    }

    pub fn default_device_assignment(
        &self,
        num_replicas: usize,
        num_partitions: usize,
    ) -> Result<DeviceAssignment> {
        let mut default_assignment = vec![0; num_replicas * num_partitions];
        let mut args = PJRT_Client_DefaultDeviceAssignment_Args::new();
        args.client = self.ptr();
        args.num_replicas = num_replicas as i32;
        args.num_partitions = num_partitions as i32;
        args.default_assignment = default_assignment.as_mut_ptr();
        args.default_assignment_size = default_assignment.len();
        _ = self.api().PJRT_Client_DefaultDeviceAssignment(args)?;
        let assignment = DeviceAssignment::new(num_replicas, num_partitions, default_assignment);
        Ok(assignment)
    }

    pub fn topology(&self) -> TopologyDescription {
        let mut args = PJRT_Client_TopologyDescription_Args::new();
        args.client = self.ptr();
        args = self
            .api()
            .PJRT_Client_TopologyDescription(args)
            .expect("PJRT_Client_TopologyDescription");
        TopologyDescription::wrap(self.api(), args.topology, Some(self))
    }

    /// Creates buffers for asynchronous host-to-device transfers.
    ///
    /// Returns a transfer manager that can be used to transfer data asynchronously.
    pub fn create_buffers_for_async_host_to_device(
        &self,
        shapes: &[BufferShape],
        memory: &Memory,
    ) -> Result<AsyncHostToDeviceTransferManager> {
        let mut specs: Vec<PJRT_ShapeSpec> = shapes.iter().map(|s| s.to_spec()).collect();
        let mut layouts: Vec<pjrt_sys::PJRT_Buffer_MemoryLayout> = shapes
            .iter()
            .filter_map(|s| s.layout().map(|l| l.into()))
            .collect();
        let mut layout_ptrs: Vec<*mut pjrt_sys::PJRT_Buffer_MemoryLayout> =
            layouts.iter_mut().map(|l| l as *mut _).collect();

        let mut args = PJRT_Client_CreateBuffersForAsyncHostToDevice_Args::new();
        args.client = self.ptr();
        args.shape_specs = specs.as_mut_ptr();
        args.num_shape_specs = shapes.len();
        if !layout_ptrs.is_empty() {
            args.device_layouts = layout_ptrs.as_mut_ptr();
            args.num_device_layouts = layout_ptrs.len();
        }
        args.memory = memory.ptr;
        let args = self
            .api()
            .PJRT_Client_CreateBuffersForAsyncHostToDevice(args)?;
        Ok(AsyncHostToDeviceTransferManager::wrap(
            self,
            args.transfer_manager,
        ))
    }

    /// Creates an uninitialized buffer on the device.
    ///
    /// The buffer's memory is allocated but not initialized.
    pub fn create_uninitialized_buffer(
        &self,
        dims: &[i64],
        element_type: PrimitiveType,
        memory: &Memory,
        layout: Option<&MemoryLayout>,
    ) -> Result<Buffer> {
        let mut args = PJRT_Client_CreateUninitializedBuffer_Args::new();
        args.client = self.ptr();
        args.shape_dims = dims.as_ptr();
        args.shape_num_dims = dims.len();
        args.shape_element_type = element_type as pjrt_sys::PJRT_Buffer_Type;
        if let Some(layout) = layout {
            let mut layout_c = pjrt_sys::PJRT_Buffer_MemoryLayout::from(layout);
            args.shape_layout = &mut layout_c as *mut _;
        }
        args.memory = memory.ptr;
        let args = self.api().PJRT_Client_CreateUninitializedBuffer(args)?;
        Ok(Buffer::wrap(self, args.buffer))
    }

    /// Creates a buffer that carries an error future without allocating memory.
    ///
    /// If this buffer is passed to an Execute call, the execution will fail
    /// with the given error code and message.
    pub fn create_error_buffer(
        &self,
        error_code: ErrorCode,
        error_message: &str,
        dims: &[i64],
        element_type: PrimitiveType,
        memory: &Memory,
        layout: Option<&MemoryLayout>,
    ) -> Result<Buffer> {
        let mut args = PJRT_Client_CreateErrorBuffer_Args::new();
        args.client = self.ptr();
        args.error_code = error_code as PJRT_Error_Code;
        args.error_message = error_message.as_ptr() as *const i8;
        args.error_message_size = error_message.len();
        args.shape_dims = dims.as_ptr();
        args.shape_num_dims = dims.len();
        args.shape_element_type = element_type as pjrt_sys::PJRT_Buffer_Type;
        if let Some(layout) = layout {
            let mut layout_c = pjrt_sys::PJRT_Buffer_MemoryLayout::from(layout);
            args.shape_layout = &mut layout_c as *mut _;
        }
        args.memory = memory.ptr;
        let args = self.api().PJRT_Client_CreateErrorBuffer(args)?;
        Ok(Buffer::wrap(self, args.buffer))
    }

    /// Creates an alias buffer that can be fulfilled later.
    ///
    /// This is useful for creating output buffers before the computation
    /// that produces them has completed.
    #[builder(finish_fn = build)]
    pub fn create_alias_buffer(
        &self,
        #[builder(start_fn)] memory: &Memory,
        dims: &[i64],
        element_type: PrimitiveType,
        layout: Option<&MemoryLayout>,
    ) -> Result<(Buffer, FulfillAliasBufferCallback)> {
        let mut args = PJRT_Client_CreateAliasBuffer_Args::new();
        args.client = self.ptr();
        args.memory = memory.ptr;
        args.shape_dims = dims.as_ptr();
        args.shape_num_dims = dims.len();
        args.shape_element_type = element_type as pjrt_sys::PJRT_Buffer_Type;
        if let Some(layout) = layout {
            let mut layout_c = pjrt_sys::PJRT_Buffer_MemoryLayout::from(layout);
            args.shape_layout = &mut layout_c as *mut _;
        }
        let args = self.api().PJRT_Client_CreateAliasBuffer(args)?;
        let buffer = Buffer::wrap(self, args.alias_buffer);
        let callback = FulfillAliasBufferCallback {
            client: self.clone(),
            ptr: args.fulfill_alias_buffer_cb,
        };
        Ok((buffer, callback))
    }

    /// Updates global process information for distributed execution.
    pub fn update_global_process_info(&self, process_infos: &[ProcessInfo]) -> Result<()> {
        let mut raw_infos: Vec<PJRT_ProcessInfo> = process_infos
            .iter()
            .map(|info| {
                let mut raw = PJRT_ProcessInfo::new();
                raw.task_id = info.task_id;
                raw.incarnation_id = info.incarnation_id;
                raw.state = info.state as pjrt_sys::PJRT_ProcessState;
                raw.error_code = info.error_code.unwrap_or(0);
                if let Some(ref msg) = info.error_message {
                    raw.error_message = msg.as_ptr() as *const i8;
                    raw.error_message_size = msg.len();
                }
                raw
            })
            .collect();

        let mut args = PJRT_Client_UpdateGlobalProcessInfo_Args::new();
        args.client = self.ptr();
        args.process_infos = raw_infos.as_mut_ptr();
        args.num_process_infos = raw_infos.len();
        self.api()
            .PJRT_Client_UpdateGlobalProcessInfo(args)
            .map(|_| ())
    }

    /// Maps a host memory region for DMA transfers to the device.
    ///
    /// This allows the device to directly access host memory via DMA.
    pub fn dma_map(&self, data: *mut c_void, size: usize) -> Result<()> {
        let mut args = PJRT_Client_DmaMap_Args::new();
        args.client = self.ptr();
        args.data = data;
        args.size = size;
        self.api().PJRT_Client_DmaMap(args).map(|_| ())
    }

    /// Unmaps a previously DMA-mapped host memory region.
    pub fn dma_unmap(&self, data: *mut c_void) -> Result<()> {
        let mut args = PJRT_Client_DmaUnmap_Args::new();
        args.client = self.ptr();
        args.data = data;
        self.api().PJRT_Client_DmaUnmap(args).map(|_| ())
    }

    /// Creates a buffer that is a non-owned view of existing device memory.
    ///
    /// This creates a PJRT buffer that wraps existing device memory allocated
    /// by another library (e.g., via dlpack). The buffer may be mutated, for
    /// example if donated to an Execute operation.
    ///
    /// # Safety
    ///
    /// - `device_buffer_ptr` must point to valid device memory
    /// - The memory must outlive the returned Buffer
    /// - If `on_delete` is provided, it will be called when the buffer is destroyed
    #[builder(finish_fn = build)]
    pub unsafe fn create_view_of_device_buffer(
        &self,
        #[builder(start_fn)] device_buffer_ptr: *mut c_void,
        #[builder(start_fn)] element_type: PrimitiveType,
        #[builder(into)] dims: Vec<i64>,
        layout: Option<&MemoryLayout>,
        memory: Option<&Memory>,
        device: Option<&Device>,
        stream: Option<isize>,
    ) -> Result<Buffer> {
        use pjrt_sys::PJRT_Client_CreateViewOfDeviceBuffer_Args;

        let mut args = PJRT_Client_CreateViewOfDeviceBuffer_Args::new();
        args.client = self.ptr();
        args.device_buffer_ptr = device_buffer_ptr;
        args.dims = dims.as_ptr();
        args.num_dims = dims.len();
        args.element_type = element_type as pjrt_sys::PJRT_Buffer_Type;

        let mut layout_c = layout.map(pjrt_sys::PJRT_Buffer_MemoryLayout::from);
        if let Some(ref mut l) = layout_c {
            args.layout = l as *mut _;
        }

        if let Some(memory) = memory {
            args.memory = memory.ptr;
        } else if let Some(device) = device {
            args.device = device.ptr;
        }

        if let Some(stream) = stream {
            args.stream = stream;
        }

        let args = self.api().PJRT_Client_CreateViewOfDeviceBuffer(args)?;
        Ok(Buffer::wrap(self, args.buffer))
    }
}

impl CompileToLoadedExecutable<Program> for Client {
    fn compile(&self, program: &Program, options: &CompileOptions) -> Result<LoadedExecutable> {
        let options_encoded = options.encode();
        let mut args = PJRT_Client_Compile_Args::new();
        args.client = self.ptr();
        args.program = &program.prog as *const PJRT_Program;
        args.compile_options = options_encoded.as_ptr() as *const i8;
        args.compile_options_size = options_encoded.len();
        args = self.api().PJRT_Client_Compile(args)?;
        Ok(LoadedExecutable::wrap(self, args.executable))
    }
}

/// Represents the state of a process in distributed execution.
#[repr(i32)]
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum ProcessState {
    Unspecified = 0,
    Uninitialized = 1,
    Disconnected = 2,
    Connected = 3,
    Error = 4,
}

/// Information about a process in distributed execution.
#[derive(Debug, Clone)]
pub struct ProcessInfo {
    pub task_id: i32,
    pub incarnation_id: u64,
    pub state: ProcessState,
    pub error_code: Option<i32>,
    pub error_message: Option<String>,
}

impl ProcessInfo {
    pub fn new(task_id: i32, state: ProcessState) -> Self {
        Self {
            task_id,
            incarnation_id: 0,
            state,
            error_code: None,
            error_message: None,
        }
    }

    pub fn with_incarnation(mut self, incarnation_id: u64) -> Self {
        self.incarnation_id = incarnation_id;
        self
    }

    pub fn with_error(mut self, code: i32, message: impl Into<String>) -> Self {
        self.error_code = Some(code);
        self.error_message = Some(message.into());
        self
    }
}

/// A callback that can be used to fulfill an alias buffer.
pub struct FulfillAliasBufferCallback {
    client: Client,
    ptr: *mut pjrt_sys::PJRT_FulfillAliasBufferCallback,
}

impl FulfillAliasBufferCallback {
    /// Fulfills the alias buffer with the given data.
    pub fn fulfill(
        &self,
        buffer: &Buffer,
        status_code: Option<ErrorCode>,
        error_message: Option<&str>,
    ) -> Result<()> {
        let mut args = PJRT_Client_FulfillAliasBuffer_Args::new();
        args.client = self.client.ptr();
        args.buffer = buffer.ptr;
        if let Some(code) = status_code {
            args.status_code = code as PJRT_Error_Code;
        }
        if let Some(msg) = error_message {
            args.error_message = msg.as_ptr() as *const i8;
            args.error_message_size = msg.len();
        }
        args.fulfill_alias_buffer_cb = self.ptr;
        self.client
            .api()
            .PJRT_Client_FulfillAliasBuffer(args)
            .map(|_| ())
    }
}
