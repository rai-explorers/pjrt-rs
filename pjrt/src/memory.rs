use std::borrow::Cow;
use std::slice;

use pjrt_sys::{
    PJRT_Memory, PJRT_Memory_AddressableByDevices_Args, PJRT_Memory_DebugString_Args,
    PJRT_Memory_Id_Args, PJRT_Memory_Kind_Args, PJRT_Memory_Kind_Id_Args,
    PJRT_Memory_ToString_Args,
};

use crate::{utils, Client, Device};

pub struct Memory {
    client: Client,
    pub(crate) ptr: *mut PJRT_Memory,
}

impl Memory {
    pub fn new(client: &Client, ptr: *mut PJRT_Memory) -> Memory {
        assert!(!ptr.is_null());
        Self {
            client: client.clone(),
            ptr,
        }
    }

    pub fn client(&self) -> &Client {
        &self.client
    }

    pub fn id(&self) -> i32 {
        let mut args = PJRT_Memory_Id_Args::new();
        args.memory = self.ptr;
        args = self
            .client
            .api()
            .PJRT_Memory_Id(args)
            .expect("PJRT_Memory_Id");
        args.id
    }

    pub fn kind(&self) -> Cow<'_, str> {
        let mut args = PJRT_Memory_Kind_Args::new();
        args.memory = self.ptr;
        args = self
            .client
            .api()
            .PJRT_Memory_Kind(args)
            .expect("PJRT_Memory_Kind");
        utils::str_from_raw(args.kind, args.kind_size)
    }

    pub fn kind_id(&self) -> i32 {
        let mut args = PJRT_Memory_Kind_Id_Args::new();
        args.memory = self.ptr;
        args = self
            .client
            .api()
            .PJRT_Memory_Kind_Id(args)
            .expect("PJRT_Memory_Kind_Id");
        args.kind_id
    }

    pub fn debug_string(&self) -> Cow<'_, str> {
        let mut args = PJRT_Memory_DebugString_Args::new();
        args.memory = self.ptr;
        args = self
            .client
            .api()
            .PJRT_Memory_DebugString(args)
            .expect("PJRT_Memory_DebugString");
        utils::str_from_raw(args.debug_string, args.debug_string_size)
    }

    pub fn to_string(&self) -> Cow<'_, str> {
        let mut args = PJRT_Memory_ToString_Args::new();
        args.memory = self.ptr;
        args = self
            .client
            .api()
            .PJRT_Memory_ToString(args)
            .expect("PJRT_Memory_ToString");
        utils::str_from_raw(args.to_string, args.to_string_size)
    }

    pub fn addressable_by_devices(&self) -> Vec<Device> {
        let mut args = PJRT_Memory_AddressableByDevices_Args::new();
        args.memory = self.ptr;
        args = self
            .client
            .api()
            .PJRT_Memory_AddressableByDevices(args)
            .expect("PJRT_Memory_AddressableByDevices");
        let devices = unsafe { slice::from_raw_parts(args.devices, args.num_devices) };
        devices
            .iter()
            .map(|device| Device::new(&self.client, *device))
            .collect()
    }
}
