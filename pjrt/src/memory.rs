//! PJRT Memory Management
//!
//! This module provides the `Memory` struct for working with PJRT memory spaces.
//! Memory spaces represent different physical memory regions (e.g., GPU memory,
//! system RAM, unified memory) and provide functionality to:
//!
//! - Query memory properties and capabilities
//! - Determine which devices can access each memory space
//! - Optimize buffer placement based on memory topology
//!
//! Understanding memory topology is important for:
//! - Optimizing data placement for performance
//! - Managing memory usage on device-limited systems
//! - Implementing efficient multi-GPU algorithms
//!
//! # Examples
//!
//! ## Querying Memory Properties
//!
//! ```rust,ignore
//! use pjrt::Client;
//!
//! // Get a device's default memory
//! let device = client.addressable_devices().first().unwrap();
//! let memory = device.default_memory();
//!
//! // Query memory properties
//! println!("Memory ID: {}", memory.id());
//! println!("Memory Kind: {}", memory.kind());
//! println!("Kind ID: {}", memory.kind_id());
//! println!("Debug: {}", memory.debug_string());
//! ```
//!
//! ## Memory-Device Relationships
//!
//! ```rust,ignore
//! // Find which devices can access this memory
//! let devices = memory.addressable_by_devices();
//! println!("Accessible by {} devices", devices.len());
//! for device in devices {
//!     let desc = device.description();
//!     println!("  - Device {} ({})", desc.id(), desc.kind());
//! }
//! ```
//!
//! ## Addressable Memories
//!
//! ```rust,ignore
//! // Get all addressable memories from the client
//! let memories = client.addressable_memories();
//! for memory in memories {
//!     println!("{}", memory); // Uses Display implementation
//! }
//! ```

use std::borrow::Cow;
use std::fmt::{self, Debug, Display};
use std::slice;

use pjrt_sys::{
    PJRT_Memory, PJRT_Memory_AddressableByDevices_Args, PJRT_Memory_DebugString_Args,
    PJRT_Memory_Id_Args, PJRT_Memory_Kind_Args, PJRT_Memory_Kind_Id_Args,
    PJRT_Memory_ToString_Args,
};

use crate::{utils, Client, Device};

/// A memory space attached to a PJRT device.
///
/// # Thread Safety
///
/// `Memory` is `!Send + !Sync` because it holds a [`Client`] reference
/// (which uses `Rc` internally). All memory operations must occur on the
/// thread that created the parent [`Client`].
pub struct Memory {
    client: Client,
    pub(crate) ptr: *mut PJRT_Memory,
}

impl Memory {
    pub(crate) fn wrap(client: &Client, ptr: *mut PJRT_Memory) -> Memory {
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
            .map(|device| Device::wrap(&self.client, *device))
            .collect()
    }
}

impl Display for Memory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Memory({})", self.to_string())
    }
}

impl Debug for Memory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Memory({})", self.debug_string())
    }
}
