use std::borrow::Cow;
use std::slice;

use bon::bon;
use pjrt_sys::{
    PJRT_SerializedTopology, PJRT_TopologyDescription, PJRT_TopologyDescription_Attributes_Args,
    PJRT_TopologyDescription_Deserialize_Args, PJRT_TopologyDescription_Destroy_Args,
    PJRT_TopologyDescription_GetDeviceDescriptions_Args,
    PJRT_TopologyDescription_PlatformName_Args, PJRT_TopologyDescription_PlatformVersion_Args,
    PJRT_TopologyDescription_Serialize_Args,
};

use crate::{utils, Api, Client, DeviceDescription, NamedValue, NamedValueMap, Result};

/// A description of the topology of devices available in a PJRT runtime.
///
/// # Thread Safety
///
/// `TopologyDescription` is `!Send + !Sync` because it may hold a
/// [`Client`] reference (which uses `Rc`) and contains a raw pointer.
pub struct TopologyDescription {
    pub(crate) api: Api,
    pub(crate) client: Option<Client>,
    pub(crate) ptr: *mut PJRT_TopologyDescription,
}

impl std::fmt::Debug for TopologyDescription {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TopologyDescription")
            .field("platform_name", &self.platform_name().unwrap_or_default())
            .field(
                "platform_version",
                &self.platform_version().unwrap_or_default(),
            )
            .field(
                "num_devices",
                &self.device_descriptions().map(|d| d.len()).unwrap_or(0),
            )
            .finish()
    }
}

impl Drop for TopologyDescription {
    fn drop(&mut self) {
        let mut args = PJRT_TopologyDescription_Destroy_Args::new();
        args.topology = self.ptr;
        if self.client.is_none() {
            self.api
                .PJRT_TopologyDescription_Destroy(args)
                .expect("PJRT_TopologyDescription_Destroy");
        }
    }
}

#[bon]
impl TopologyDescription {
    pub(crate) fn wrap(
        api: &Api,
        ptr: *mut PJRT_TopologyDescription,
        client: Option<&Client>,
    ) -> TopologyDescription {
        assert!(!ptr.is_null());
        Self {
            api: api.clone(),
            ptr,
            client: client.cloned(),
        }
    }

    #[builder(finish_fn = build)]
    pub fn builder(
        #[builder(start_fn)] api: &Api,
        #[builder(start_fn)] name: impl AsRef<str>,
        #[builder(default = bon::vec![], into)] options: Vec<NamedValue>,
    ) -> Result<Self> {
        api.create_topology(name, options)
    }

    pub fn platform_name(&self) -> Result<Cow<'_, str>> {
        let mut args = PJRT_TopologyDescription_PlatformName_Args::new();
        args.topology = self.ptr;
        args = self.api.PJRT_TopologyDescription_PlatformName(args)?;
        Ok(utils::str_from_raw(
            args.platform_name,
            args.platform_name_size,
        ))
    }

    pub fn platform_version(&self) -> Result<Cow<'_, str>> {
        let mut args = PJRT_TopologyDescription_PlatformVersion_Args::new();
        args.topology = self.ptr;
        args = self.api.PJRT_TopologyDescription_PlatformVersion(args)?;
        Ok(utils::str_from_raw(
            args.platform_version,
            args.platform_version_size,
        ))
    }

    pub fn device_descriptions(&self) -> Result<Vec<DeviceDescription>> {
        let mut args = PJRT_TopologyDescription_GetDeviceDescriptions_Args::new();
        args.topology = self.ptr;
        args = self
            .api
            .PJRT_TopologyDescription_GetDeviceDescriptions(args)?;
        let descriptions =
            unsafe { slice::from_raw_parts(args.descriptions, args.num_descriptions) };
        Ok(descriptions
            .iter()
            .map(|ptr| DeviceDescription::wrap(&self.api, *ptr))
            .collect())
    }

    pub fn attributes(&self) -> Result<NamedValueMap> {
        let mut args = PJRT_TopologyDescription_Attributes_Args::new();
        args.topology = self.ptr;
        args = self.api.PJRT_TopologyDescription_Attributes(args)?;
        utils::to_named_value_map(args.attributes, args.num_attributes)
    }

    pub fn serialize(&self) -> Result<SerializedTopology> {
        let mut args = PJRT_TopologyDescription_Serialize_Args::new();
        args.topology = self.ptr;
        args = self.api.PJRT_TopologyDescription_Serialize(args)?;
        let deleter = args
            .serialized_topology_deleter
            .ok_or_else(|| crate::Error::InvalidArgument("null topology deleter".into()))?;
        Ok(SerializedTopology {
            ptr: args.serialized_topology,
            deleter,
            data_ptr: args.serialized_bytes as *const u8,
            data_len: args.serialized_bytes_size,
        })
    }

    /// Deserializes a topology from previously serialized bytes.
    ///
    /// This is useful for recreating a topology on a different process
    /// or after serialization for caching purposes.
    pub fn deserialize(api: &Api, bytes: &[u8]) -> Result<Self> {
        let mut args = PJRT_TopologyDescription_Deserialize_Args::new();
        args.serialized_topology = bytes.as_ptr() as *const i8;
        args.serialized_topology_size = bytes.len();
        let args = api.PJRT_TopologyDescription_Deserialize(args)?;
        Ok(Self::wrap(api, args.topology, None))
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
