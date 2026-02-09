//! PJRT Device Management
//!
//! This module provides the `Device` struct and related types for managing PJRT devices.
//! Devices represent hardware accelerators (CPU, GPU, TPU, etc.) available for computation.
//! This module includes functionality to:
//!
//! - Query device properties and capabilities
//! - Access device memory spaces
//! - Monitor memory usage
//! - Create async tracking events
//!
//! Each device has various IDs:
//! - `GlobalDeviceId`: Unique across all devices of the same type
//! - `LocalDeviceId`: Unique within a client (-1 if undefined)
//! - `LocalHardwareId`: Physical device ID, shared by logical devices on the same hardware
//!
//! # Examples
//!
//! ## Querying Device Information
//!
//! ```rust,ignore
//! use pjrt::Client;
//!
//! let devices = client.devices();
//! for device in &devices {
//!     let desc = device.description();
//!     println!("Device ID: {}", desc.id());
//!     println!("Device Kind: {}", desc.kind());
//!     println!("Is Addressable: {}", device.is_addressable());
//!     println!("Local Hardware ID: {}", device.local_hardware_id());
//! }
//! ```
//!
//! ## Working with Device Memory
//!
//! ```rust,ignore
//! // Get default memory space for a device
//! let device = client.addressable_devices().first().unwrap();
//! let memory = device.default_memory();
//! println!("Memory kind: {}", memory.kind());
//!
//! // Get all addressable memories
//! let memories = device.addressable_memories();
//! for mem in memories {
//!     println!("Memory: {} (id: {})", mem.kind(), mem.id());
//! }
//! ```
//!
//! ## Memory Statistics
//!
//! ```rust,ignore
//! if let Ok(stats) = device.memory_stats() {
//!     println!("Bytes in use: {:?}", stats.bytes_in_use);
//!     println!("Peak bytes in use: {:?}", stats.peak_bytes_in_use);
//! }
//! ```

use std::slice;

use pjrt_sys::{
    PJRT_AsyncTrackingEvent_Destroy_Args, PJRT_Device, PJRT_Device_AddressableMemories_Args,
    PJRT_Device_CreateAsyncTrackingEvent_Args, PJRT_Device_DefaultMemory_Args,
    PJRT_Device_GetDescription_Args, PJRT_Device_IsAddressable_Args,
    PJRT_Device_LocalHardwareId_Args, PJRT_Device_MemoryStats_Args,
    PJRT_Device_PoisonExecution_Args,
};

use crate::{Client, DeviceDescription, ErrorCode, Memory, Result};

/// The logical global device ID.
/// This is unique among devices of this type (e.g. CPUs, GPUs).
/// On multi-host platforms, this will be unique across all hosts' devices.
pub type GlobalDeviceId = i32;

/// The logical local device ID.
/// This will be used to look up an addressable device local to a given client.
/// It is -1 if undefined.
pub type LocalDeviceId = i32;

/// The physical local device ID.
/// Multiple PJRT devices can have the same LocalHardwareId if
/// these PJRT devices share the same physical device.
/// In general, not guaranteed to be dense, and -1 if undefined.
pub type LocalHardwareId = i32;

/// A hardware device (CPU, GPU, TPU) managed by a PJRT client.
///
/// # Thread Safety
///
/// `Device` is `!Send + !Sync` because it holds a [`Client`] reference
/// (which uses `Rc` internally). All device operations must occur on the
/// thread that created the parent [`Client`].
pub struct Device {
    client: Client,
    pub(crate) ptr: *mut PJRT_Device,
}

impl std::fmt::Debug for Device {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let description = self.description();
        f.debug_struct("Device")
            .field("id", &description.id())
            .field("kind", &description.kind())
            .field("is_addressable", &self.is_addressable())
            .field("local_hardware_id", &self.local_hardware_id())
            .finish()
    }
}

impl Device {
    pub(crate) fn wrap(client: &Client, ptr: *mut PJRT_Device) -> Device {
        assert!(!ptr.is_null());
        Self {
            client: client.clone(),
            ptr,
        }
    }

    pub fn client(&self) -> &Client {
        &self.client
    }

    pub fn description(&self) -> DeviceDescription {
        let mut args = PJRT_Device_GetDescription_Args::new();
        args.device = self.ptr;
        args = self
            .client
            .api()
            .PJRT_Device_GetDescription(args)
            .expect("PJRT_Device_GetDescription");
        DeviceDescription::wrap(self.client.api(), args.device_description)
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

    pub fn local_hardware_id(&self) -> LocalHardwareId {
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
            .map(|d| Memory::wrap(&self.client, d))
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
        Memory::wrap(&self.client, args.memory)
    }

    pub fn memory_stats(&self) -> Result<MemoryStats> {
        let mut args = PJRT_Device_MemoryStats_Args::new();
        args.device = self.ptr;
        args = self.client.api().PJRT_Device_MemoryStats(args)?;
        Ok(MemoryStats::from(args))
    }

    /// Creates an async tracking event for this device.
    ///
    /// This can be used to track the completion of asynchronous operations.
    pub fn create_async_tracking_event(&self) -> Result<AsyncTrackingEvent> {
        let mut args = PJRT_Device_CreateAsyncTrackingEvent_Args::new();
        args.device = self.ptr;
        let args = self
            .client
            .api()
            .PJRT_Device_CreateAsyncTrackingEvent(args)?;
        Ok(AsyncTrackingEvent {
            api: self.client.api().clone(),
            ptr: args.event,
        })
    }

    /// Marks this device's execution as poisoned.
    ///
    /// This is a mechanism to signal that something has gone wrong with the device
    /// and subsequent operations should fail with the given error code and message.
    pub fn poison_execution(&self, error_code: ErrorCode, error_message: &str) -> Result<()> {
        let mut args = PJRT_Device_PoisonExecution_Args::new();
        args.device = self.ptr;
        args.error_code = error_code as pjrt_sys::PJRT_Error_Code;
        args.error_message = error_message.as_ptr() as *const i8;
        args.error_message_size = error_message.len();
        self.client
            .api()
            .PJRT_Device_PoisonExecution(args)
            .map(|_| ())
    }
}

/// An async tracking event for monitoring device operations.
pub struct AsyncTrackingEvent {
    api: crate::Api,
    ptr: *mut pjrt_sys::PJRT_AsyncTrackingEvent,
}

impl Drop for AsyncTrackingEvent {
    fn drop(&mut self) {
        let mut args = PJRT_AsyncTrackingEvent_Destroy_Args::new();
        args.event = self.ptr;
        self.api
            .PJRT_AsyncTrackingEvent_Destroy(args)
            .expect("PJRT_AsyncTrackingEvent_Destroy");
    }
}

impl AsyncTrackingEvent {
    pub fn api(&self) -> &crate::Api {
        &self.api
    }

    #[allow(dead_code)]
    pub(crate) fn ptr(&self) -> *mut pjrt_sys::PJRT_AsyncTrackingEvent {
        self.ptr
    }
}

impl std::fmt::Debug for AsyncTrackingEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AsyncTrackingEvent").finish_non_exhaustive()
    }
}

/// Memory usage statistics for a device.
///
/// `bytes_in_use` is always available. All other fields are optional
/// because not every PJRT plugin reports them.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct MemoryStats {
    /// Current bytes in use.
    pub bytes_in_use: i64,
    /// Peak bytes in use over the lifetime of the allocator.
    pub peak_bytes_in_use: Option<i64>,
    /// Number of allocations made.
    pub num_allocs: Option<i64>,
    /// Size of the largest single allocation.
    pub largest_alloc_size: Option<i64>,
    /// Memory limit (if set by the runtime).
    pub bytes_limit: Option<i64>,
    /// Current bytes reserved by the allocator.
    pub bytes_reserved: Option<i64>,
    /// Peak bytes reserved by the allocator.
    pub peak_bytes_reserved: Option<i64>,
    /// Limit on reservable bytes.
    pub bytes_reservable_limit: Option<i64>,
    /// Largest free block available in the pool.
    pub largest_free_block_bytes: Option<i64>,
    /// Current bytes in the memory pool.
    pub pool_bytes: Option<i64>,
    /// Peak bytes in the memory pool.
    pub peak_pool_bytes: Option<i64>,
}

impl From<PJRT_Device_MemoryStats_Args> for MemoryStats {
    fn from(args: PJRT_Device_MemoryStats_Args) -> Self {
        Self {
            bytes_in_use: args.bytes_in_use,
            peak_bytes_in_use: args
                .peak_bytes_in_use_is_set
                .then_some(args.peak_bytes_in_use),
            num_allocs: args.num_allocs_is_set.then_some(args.num_allocs),
            largest_alloc_size: args
                .largest_alloc_size_is_set
                .then_some(args.largest_alloc_size),
            bytes_limit: args.bytes_limit_is_set.then_some(args.bytes_limit),
            bytes_reserved: args.bytes_reserved_is_set.then_some(args.bytes_reserved),
            peak_bytes_reserved: args
                .peak_bytes_reserved_is_set
                .then_some(args.peak_bytes_reserved),
            bytes_reservable_limit: args
                .bytes_reservable_limit_is_set
                .then_some(args.bytes_reservable_limit),
            largest_free_block_bytes: args
                .largest_free_block_bytes_is_set
                .then_some(args.largest_free_block_bytes),
            pool_bytes: args.pool_bytes_is_set.then_some(args.pool_bytes),
            peak_pool_bytes: args.peak_pool_bytes_is_set.then_some(args.peak_pool_bytes),
        }
    }
}
