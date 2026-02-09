use std::borrow::Cow;
use std::fmt::{self, Debug, Display};

use pjrt_sys::{
    PJRT_DeviceDescription, PJRT_DeviceDescription_Attributes_Args,
    PJRT_DeviceDescription_DebugString_Args, PJRT_DeviceDescription_Id_Args,
    PJRT_DeviceDescription_Kind_Args, PJRT_DeviceDescription_ProcessIndex_Args,
    PJRT_DeviceDescription_ToString_Args,
};

use crate::named_value::NamedValueMap;
use crate::{utils, Api, GlobalDeviceId, Result};

/// Metadata describing a PJRT device (ID, kind, process index, attributes).
///
/// # Thread Safety
///
/// `DeviceDescription` is `!Send + !Sync` due to the raw
/// `*mut PJRT_DeviceDescription` pointer.
pub struct DeviceDescription {
    api: Api,
    pub(crate) ptr: *mut PJRT_DeviceDescription,
}

impl DeviceDescription {
    pub(crate) fn wrap(api: &Api, ptr: *mut PJRT_DeviceDescription) -> DeviceDescription {
        assert!(!ptr.is_null());
        Self {
            api: api.clone(),
            ptr,
        }
    }

    pub fn api(&self) -> &Api {
        &self.api
    }

    pub fn id(&self) -> Result<GlobalDeviceId> {
        let mut args = PJRT_DeviceDescription_Id_Args::new();
        args.device_description = self.ptr;
        args = self.api.PJRT_DeviceDescription_Id(args)?;
        Ok(args.id)
    }

    pub fn process_index(&self) -> Result<i32> {
        let mut args = PJRT_DeviceDescription_ProcessIndex_Args::new();
        args.device_description = self.ptr;
        args = self.api.PJRT_DeviceDescription_ProcessIndex(args)?;
        Ok(args.process_index)
    }

    pub fn attributes(&self) -> Result<NamedValueMap> {
        let mut args = PJRT_DeviceDescription_Attributes_Args::new();
        args.device_description = self.ptr;
        args = self.api.PJRT_DeviceDescription_Attributes(args)?;
        utils::to_named_value_map(args.attributes, args.num_attributes)
    }

    pub fn kind(&self) -> Result<Cow<'_, str>> {
        let mut args = PJRT_DeviceDescription_Kind_Args::new();
        args.device_description = self.ptr;
        args = self.api.PJRT_DeviceDescription_Kind(args)?;
        Ok(utils::str_from_raw(args.device_kind, args.device_kind_size))
    }

    pub fn debug_string(&self) -> Result<Cow<'_, str>> {
        let mut args = PJRT_DeviceDescription_DebugString_Args::new();
        args.device_description = self.ptr;
        args = self.api.PJRT_DeviceDescription_DebugString(args)?;
        Ok(utils::str_from_raw(
            args.debug_string,
            args.debug_string_size,
        ))
    }

    pub fn display_string(&self) -> Result<Cow<'_, str>> {
        let mut args = PJRT_DeviceDescription_ToString_Args::new();
        args.device_description = self.ptr;
        args = self.api.PJRT_DeviceDescription_ToString(args)?;
        Ok(utils::str_from_raw(args.to_string, args.to_string_size))
    }
}

impl Display for DeviceDescription {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.display_string() {
            Ok(s) => write!(f, "DeviceDescription({})", s),
            Err(e) => write!(f, "DeviceDescription(<error: {}>)", e),
        }
    }
}

impl Debug for DeviceDescription {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.debug_string() {
            Ok(s) => write!(f, "DeviceDescription({})", s),
            Err(e) => write!(f, "DeviceDescription(<error: {}>)", e),
        }
    }
}
