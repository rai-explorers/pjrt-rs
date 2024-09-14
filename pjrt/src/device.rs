use std::slice;

use pjrt_sys::{
    PJRT_Device, PJRT_Device_AddressableMemories_Args, PJRT_Device_DefaultMemory_Args,
    PJRT_Device_GetDescription_Args, PJRT_Device_IsAddressable_Args,
    PJRT_Device_LocalHardwareId_Args, PJRT_Device_MemoryStats_Args,
};

use crate::{Client, DeviceDescription, Memory, Result};

pub struct Device {
    pub(crate) client: Client,
    pub(crate) ptr: *mut PJRT_Device,
}

impl Device {
    pub fn new(client: &Client, ptr: *mut PJRT_Device) -> Device {
        assert!(!ptr.is_null());
        Self {
            client: client.clone(),
            ptr,
        }
    }

    pub fn get_description(&self) -> DeviceDescription {
        let mut args = PJRT_Device_GetDescription_Args::new();
        args.device = self.ptr;
        args = self
            .client
            .api()
            .PJRT_Device_GetDescription(args)
            .expect("PJRT_Device_GetDescription");
        DeviceDescription::new(&self.client.api(), args.device_description)
    }

    pub fn is_addressable(&self) -> bool {
        let mut args = PJRT_Device_IsAddressable_Args::new();
        args.device = self.ptr;
        args = self
            .client
            .api()
            .PJRT_Device_IsAddressable(args)
            .expect("PJRT_Device_IsAddressable");
        args.is_addressable
    }

    pub fn local_hardware_id(&self) -> i32 {
        let mut args = PJRT_Device_LocalHardwareId_Args::new();
        args.device = self.ptr;
        args = self
            .client
            .api()
            .PJRT_Device_LocalHardwareId(args)
            .expect("PJRT_Device_LocalHardwareId");
        args.local_hardware_id
    }

    pub fn addressable_memories(&self) -> Vec<Memory> {
        let mut args = PJRT_Device_AddressableMemories_Args::new();
        args.device = self.ptr;
        args = self
            .client
            .api()
            .PJRT_Device_AddressableMemories(args)
            .expect("PJRT_Device_AddressableMemories");
        let memories = unsafe { slice::from_raw_parts(args.memories, args.num_memories) };
        memories
            .iter()
            .cloned()
            .map(|d| Memory::new(&self.client, d))
            .collect()
    }

    pub fn default_memory(&self) -> Memory {
        let mut args = PJRT_Device_DefaultMemory_Args::new();
        args.device = self.ptr;
        args = self
            .client
            .api()
            .PJRT_Device_DefaultMemory(args)
            .expect("PJRT_Device_DefaultMemory");
        Memory::new(&self.client, args.memory)
    }

    pub fn memory_stats(&self) -> Result<MemoryStats> {
        let mut args = PJRT_Device_MemoryStats_Args::new();
        args.device = self.ptr;
        args = self.client.api().PJRT_Device_MemoryStats(args)?;
        Ok(MemoryStats::from(args))
    }
}

#[derive(Debug, Clone)]
pub struct MemoryStats {
    pub bytes_in_use: i64,
    pub peak_bytes_in_use: i64,
    pub peak_bytes_in_use_is_set: bool,
    pub num_allocs: i64,
    pub num_allocs_is_set: bool,
    pub largest_alloc_size: i64,
    pub largest_alloc_size_is_set: bool,
    pub bytes_limit: i64,
    pub bytes_limit_is_set: bool,
    pub bytes_reserved: i64,
    pub bytes_reserved_is_set: bool,
    pub peak_bytes_reserved: i64,
    pub peak_bytes_reserved_is_set: bool,
    pub bytes_reservable_limit: i64,
    pub bytes_reservable_limit_is_set: bool,
    pub largest_free_block_bytes: i64,
    pub largest_free_block_bytes_is_set: bool,
    pub pool_bytes: i64,
    pub pool_bytes_is_set: bool,
    pub peak_pool_bytes: i64,
    pub peak_pool_bytes_is_set: bool,
}

impl From<PJRT_Device_MemoryStats_Args> for MemoryStats {
    fn from(args: PJRT_Device_MemoryStats_Args) -> Self {
        Self {
            bytes_in_use: args.bytes_in_use,
            peak_bytes_in_use: args.peak_bytes_in_use,
            peak_bytes_in_use_is_set: args.peak_bytes_in_use_is_set,
            num_allocs: args.num_allocs,
            num_allocs_is_set: args.num_allocs_is_set,
            largest_alloc_size: args.largest_alloc_size,
            largest_alloc_size_is_set: args.largest_alloc_size_is_set,
            bytes_limit: args.bytes_limit,
            bytes_limit_is_set: args.bytes_limit_is_set,
            bytes_reserved: args.bytes_reserved,
            bytes_reserved_is_set: args.bytes_reserved_is_set,
            peak_bytes_reserved: args.peak_bytes_reserved,
            peak_bytes_reserved_is_set: args.peak_bytes_reserved_is_set,
            bytes_reservable_limit: args.bytes_reservable_limit,
            bytes_reservable_limit_is_set: args.bytes_reservable_limit_is_set,
            largest_free_block_bytes: args.largest_free_block_bytes,
            largest_free_block_bytes_is_set: args.largest_free_block_bytes_is_set,
            pool_bytes: args.pool_bytes,
            pool_bytes_is_set: args.pool_bytes_is_set,
            peak_pool_bytes: args.peak_pool_bytes,
            peak_pool_bytes_is_set: args.peak_pool_bytes_is_set,
        }
    }
}
