use std::borrow::Cow;
use std::slice;

use pjrt_sys::{
    PJRT_SerializedTopology, PJRT_TopologyDescription, PJRT_TopologyDescription_Attributes_Args,
    PJRT_TopologyDescription_Destroy_Args, PJRT_TopologyDescription_GetDeviceDescriptions_Args,
    PJRT_TopologyDescription_PlatformName_Args, PJRT_TopologyDescription_PlatformVersion_Args,
    PJRT_TopologyDescription_Serialize_Args,
};

use crate::{utils, Api, DeviceDescription, NamedValueMap};

pub struct TopologyDescription {
    api: Api,
    ptr: *mut PJRT_TopologyDescription,
}

impl Drop for TopologyDescription {
    fn drop(&mut self) {
        let mut args = PJRT_TopologyDescription_Destroy_Args::new();
        args.topology = self.ptr;
        self.api
            .PJRT_TopologyDescription_Destroy(args)
            .expect("PJRT_TopologyDescription_Destroy");
    }
}

impl TopologyDescription {
    pub fn new(api: &Api, ptr: *mut PJRT_TopologyDescription) -> TopologyDescription {
        assert!(!ptr.is_null());
        Self {
            api: api.clone(),
            ptr,
        }
    }

    pub fn platform_name(&self) -> Cow<'_, str> {
        let mut args = PJRT_TopologyDescription_PlatformName_Args::new();
        args.topology = self.ptr;
        args = self
            .api
            .PJRT_TopologyDescription_PlatformName(args)
            .expect("PJRT_TopologyDescription_PlatformName");
        utils::str_from_raw(args.platform_name, args.platform_name_size)
    }

    pub fn platform_version(&self) -> Cow<'_, str> {
        let mut args = PJRT_TopologyDescription_PlatformVersion_Args::new();
        args.topology = self.ptr;
        args = self
            .api
            .PJRT_TopologyDescription_PlatformVersion(args)
            .expect("PJRT_TopologyDescription_PlatformVersion");
        utils::str_from_raw(args.platform_version, args.platform_version_size)
    }

    pub fn device_descriptions(&self) -> Vec<DeviceDescription> {
        let mut args = PJRT_TopologyDescription_GetDeviceDescriptions_Args::new();
        args.topology = self.ptr;
        args = self
            .api
            .PJRT_TopologyDescription_GetDeviceDescriptions(args)
            .expect("PJRT_TopologyDescription_GetDeviceDescriptions");
        let descriptions =
            unsafe { slice::from_raw_parts(args.descriptions, args.num_descriptions) };
        descriptions
            .iter()
            .map(|ptr| DeviceDescription::new(&self.api, *ptr))
            .collect()
    }

    pub fn attributes(&self) -> NamedValueMap {
        let mut args = PJRT_TopologyDescription_Attributes_Args::new();
        args.topology = self.ptr;
        args = self
            .api
            .PJRT_TopologyDescription_Attributes(args)
            .expect("PJRT_TopologyDescription_Attributes");
        utils::to_named_value_map(args.attributes, args.num_attributes)
    }

    pub fn serialize(&self) -> SerializedTopology {
        let mut args = PJRT_TopologyDescription_Serialize_Args::new();
        args.topology = self.ptr;
        args = self
            .api
            .PJRT_TopologyDescription_Serialize(args)
            .expect("PJRT_TopologyDescription_Serialize");
        SerializedTopology {
            ptr: args.serialized_topology,
            deleter: args.serialized_topology_deleter.expect("topology_deleter"),
            data_ptr: args.serialized_bytes as *const u8,
            data_len: args.serialized_bytes_size,
        }
    }
}

pub struct SerializedTopology {
    ptr: *mut PJRT_SerializedTopology,
    deleter: unsafe extern "C" fn(topology: *mut PJRT_SerializedTopology),
    data_ptr: *const u8,
    data_len: usize,
}

impl Drop for SerializedTopology {
    fn drop(&mut self) {
        unsafe { (self.deleter)(self.ptr) };
    }
}

impl SerializedTopology {
    pub fn bytes(&self) -> &[u8] {
        unsafe { std::slice::from_raw_parts(self.data_ptr, self.data_len) }
    }
}
