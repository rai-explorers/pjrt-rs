use std::borrow::Cow;
use std::rc::Rc;
use std::slice;

use bon::bon;
use pjrt_sys::{
    PJRT_Client, PJRT_Client_AddressableDevices_Args, PJRT_Client_AddressableMemories_Args,
    PJRT_Client_Compile_Args, PJRT_Client_DefaultDeviceAssignment_Args, PJRT_Client_Destroy_Args,
    PJRT_Client_Devices_Args, PJRT_Client_LookupAddressableDevice_Args,
    PJRT_Client_LookupDevice_Args, PJRT_Client_PlatformName_Args, PJRT_Client_PlatformVersion_Args,
    PJRT_Client_ProcessIndex_Args, PJRT_Client_TopologyDescription_Args,
    PJRT_Executable_DeserializeAndLoad_Args, PJRT_Program,
};

use crate::{
    utils, Api, CompileOptions, CompileToLoadedExecutable, Device, DeviceAssignment,
    GlobalDeviceId, KeyValueStore, LoadedExecutable, LocalHardwareId, Memory, NamedValue, Program,
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
        #[builder] kv_store: Option<&Box<dyn KeyValueStore>>,
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

    // TODO:
    // PJRT_Client_CreateViewOfDeviceBuffer
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
