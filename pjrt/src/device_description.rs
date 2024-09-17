use std::borrow::Cow;
use std::fmt::{self, Debug, Display};

use pjrt_sys::{
    PJRT_DeviceDescription, PJRT_DeviceDescription_Attributes_Args,
    PJRT_DeviceDescription_DebugString_Args, PJRT_DeviceDescription_Id_Args,
    PJRT_DeviceDescription_Kind_Args, PJRT_DeviceDescription_ProcessIndex_Args,
    PJRT_DeviceDescription_ToString_Args,
};

use crate::named_value::NamedValueMap;
use crate::{utils, Api};

pub struct DeviceDescription {
    api: Api,
    pub(crate) ptr: *mut PJRT_DeviceDescription,
}

impl DeviceDescription {
    pub fn wrap(api: &Api, ptr: *mut PJRT_DeviceDescription) -> DeviceDescription {
        assert!(!ptr.is_null());
        Self {
            api: api.clone(),
            ptr,
        }
    }

    pub fn api(&self) -> &Api {
        &self.api
    }

    pub fn id(&self) -> i32 {
        let mut args = PJRT_DeviceDescription_Id_Args::new();
        args.device_description = self.ptr;
        args = self
            .api
            .PJRT_DeviceDescription_Id(args)
            .expect("PJRT_DeviceDescription_Id");
        args.id
    }

    pub fn process_index(&self) -> i32 {
        let mut args = PJRT_DeviceDescription_ProcessIndex_Args::new();
        args.device_description = self.ptr;
        args = self
            .api
            .PJRT_DeviceDescription_ProcessIndex(args)
            .expect("PJRT_DeviceDescription_ProcessIndex");
        args.process_index
    }

    pub fn attributes(&self) -> NamedValueMap {
        let mut args = PJRT_DeviceDescription_Attributes_Args::new();
        args.device_description = self.ptr;
        args = self
            .api
            .PJRT_DeviceDescription_Attributes(args)
            .expect("PJRT_DeviceDescription_Attributes");
        utils::to_named_value_map(args.attributes, args.num_attributes)
    }

    pub fn kind(&self) -> Cow<'_, str> {
        let mut args = PJRT_DeviceDescription_Kind_Args::new();
        args.device_description = self.ptr;
        args = self
            .api
            .PJRT_DeviceDescription_Kind(args)
            .expect("PJRT_DeviceDescription_Kind");
        utils::str_from_raw(args.device_kind, args.device_kind_size)
    }

    pub fn debug_string(&self) -> Cow<'_, str> {
        let mut args = PJRT_DeviceDescription_DebugString_Args::new();
        args.device_description = self.ptr;
        args = self
            .api
            .PJRT_DeviceDescription_DebugString(args)
            .expect("PJRT_DeviceDescription_DebugString");
        utils::str_from_raw(args.debug_string, args.debug_string_size)
    }

    pub fn to_string(&self) -> Cow<'_, str> {
        let mut args = PJRT_DeviceDescription_ToString_Args::new();
        args.device_description = self.ptr;
        args = self
            .api
            .PJRT_DeviceDescription_ToString(args)
            .expect("PJRT_DeviceDescription_ToString");
        utils::str_from_raw(args.to_string, args.to_string_size)
    }
}

impl Display for DeviceDescription {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "DeviceDescription({})", self.to_string())
    }
}

impl Debug for DeviceDescription {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "DeviceDescription({})", self.debug_string())
    }
}
