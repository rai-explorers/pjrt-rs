//! PJRT Megascale Extension
//!
//! This module provides safe Rust bindings for the PJRT Megascale extension.
//! The Megascale extension provides capabilities for large-scale distributed training.
//!
//! ## Overview
//!
//! This extension is designed for very large scale distributed training scenarios
//! and provides capabilities for:
//!
//! - Creating and managing client contexts for multi-slice training
//! - Configuring multi-slice topologies with AoT or runtime configurations
//! - Querying number of slices, slice IDs, and device counts per slice
//! - Serializing multi-slice configurations
//!
//! ## Usage
//!
//! ```rust,ignore
//! use pjrt::MegascaleExtension;
//!
//! // Get the megascale extension if available
//! if let Some(ext) = api.get_extension::<MegascaleExtension>() {
//!     // Create a client context from an existing PJRT client
//!     let ctx = ext.create_client_context_from_pjrt_client(&client)?;
//!     ctx.initialize()?;
//!     let port = ctx.megascale_port()?;
//!     println!("Megascale port: {}", port);
//! }
//! ```
//!
//! ## Note
//!
//! This extension is primarily available in PJRT plugins designed for
//! large-scale distributed training, such as TPU pods.

use std::rc::Rc;

use pjrt_sys::{
    PJRT_Megascale_ClientContext_Initialize_Args, PJRT_Megascale_ClientContext_MegascalePort_Args,
    PJRT_Megascale_ClientContext_UnblockPendingWork_Args, PJRT_Megascale_CreateAoTConfig_Args,
    PJRT_Megascale_CreateClientContextFromPjRtClient_Args,
    PJRT_Megascale_CreateDefaultClientContext_Args, PJRT_Megascale_CreateMultiSliceConfig_Args,
    PJRT_Megascale_DeleteClientContext_Args, PJRT_Megascale_DeleteMultiSliceConfig_Args,
    PJRT_Megascale_Extension, PJRT_Megascale_MultiSliceConfig_GetNumDevicesPerSlice_Args,
    PJRT_Megascale_MultiSliceConfig_NumSlices_Args, PJRT_Megascale_MultiSliceConfig_Serialize_Args,
    PJRT_Megascale_MultiSliceConfig_SliceId_Args,
};

use crate::extension::{Extension, ExtensionType};
use crate::{Api, Client, Error, Result, TopologyDescription};

/// Opaque handle to a Megascale client context.
///
/// Created by [`MegascaleExtension::create_client_context_from_pjrt_client`] or
/// [`MegascaleExtension::create_default_client_context`]. Must be destroyed by
/// [`MegascaleExtension::delete_client_context`] or dropped via the `Drop` impl.
pub struct MegascaleClientContext {
    ptr: *mut pjrt_sys::PJRT_Megascale_ClientContext,
    ext: Rc<PJRT_Megascale_Extension>,
    api: Api,
}

impl std::fmt::Debug for MegascaleClientContext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MegascaleClientContext")
            .field("ptr", &self.ptr)
            .finish()
    }
}

impl MegascaleClientContext {
    /// Initialize this client context.
    ///
    /// Must be called before using the context for multi-slice operations.
    pub fn initialize(&self) -> Result<()> {
        let mut args: PJRT_Megascale_ClientContext_Initialize_Args = unsafe { std::mem::zeroed() };
        args.struct_size = std::mem::size_of::<PJRT_Megascale_ClientContext_Initialize_Args>();
        args.client_context = self.ptr;

        let ext_fn = self
            .ext
            .client_context_initialize
            .ok_or(Error::NullFunctionPointer(
                "PJRT_Megascale_ClientContext_Initialize",
            ))?;

        let err = unsafe { ext_fn(&mut args) };
        self.api.err_or(err, ())?;
        Ok(())
    }

    /// Unblock pending work for a given launch ID.
    ///
    /// # Arguments
    ///
    /// * `launch_id` - The launch ID to unblock
    /// * `expire_after_ms` - Expiration time in milliseconds
    pub fn unblock_pending_work(&self, launch_id: i32, expire_after_ms: i64) -> Result<()> {
        let mut args: PJRT_Megascale_ClientContext_UnblockPendingWork_Args =
            unsafe { std::mem::zeroed() };
        args.struct_size =
            std::mem::size_of::<PJRT_Megascale_ClientContext_UnblockPendingWork_Args>();
        args.client_context = self.ptr;
        args.launch_id = launch_id;
        args.expire_after_ms = expire_after_ms;

        let ext_fn =
            self.ext
                .client_context_unblock_pending_work
                .ok_or(Error::NullFunctionPointer(
                    "PJRT_Megascale_ClientContext_UnblockPendingWork",
                ))?;

        let err = unsafe { ext_fn(&mut args) };
        self.api.err_or(err, ())?;
        Ok(())
    }

    /// Get the Megascale port for this client context.
    pub fn megascale_port(&self) -> Result<i32> {
        let mut args: PJRT_Megascale_ClientContext_MegascalePort_Args =
            unsafe { std::mem::zeroed() };
        args.struct_size = std::mem::size_of::<PJRT_Megascale_ClientContext_MegascalePort_Args>();
        args.client_context = self.ptr;

        let ext_fn = self
            .ext
            .client_context_megascale_port
            .ok_or(Error::NullFunctionPointer(
                "PJRT_Megascale_ClientContext_MegascalePort",
            ))?;

        let err = unsafe { ext_fn(&mut args) };
        self.api.err_or(err, ())?;
        Ok(args.port)
    }
}

impl Drop for MegascaleClientContext {
    fn drop(&mut self) {
        if let Some(ext_fn) = self.ext.delete_client_context {
            let mut args: PJRT_Megascale_DeleteClientContext_Args = unsafe { std::mem::zeroed() };
            args.struct_size = std::mem::size_of::<PJRT_Megascale_DeleteClientContext_Args>();
            args.client_context = self.ptr;
            unsafe { ext_fn(&mut args) };
        }
    }
}

/// Opaque handle to a Megascale multi-slice configuration.
///
/// Created by [`MegascaleExtension::create_aot_config`] or
/// [`MegascaleExtension::create_multi_slice_config`]. Automatically
/// destroyed on drop.
pub struct MegascaleMultiSliceConfig {
    ptr: *mut pjrt_sys::PJRT_Megascale_MultiSliceConfig,
    ext: Rc<PJRT_Megascale_Extension>,
    api: Api,
}

impl std::fmt::Debug for MegascaleMultiSliceConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MegascaleMultiSliceConfig")
            .field("ptr", &self.ptr)
            .finish()
    }
}

impl MegascaleMultiSliceConfig {
    /// Get the number of slices in this multi-slice config.
    pub fn num_slices(&self) -> Result<i32> {
        let mut args: PJRT_Megascale_MultiSliceConfig_NumSlices_Args =
            unsafe { std::mem::zeroed() };
        args.struct_size = std::mem::size_of::<PJRT_Megascale_MultiSliceConfig_NumSlices_Args>();
        args.config = self.ptr;

        let ext_fn = self
            .ext
            .multi_slice_config_num_slices
            .ok_or(Error::NullFunctionPointer(
                "PJRT_Megascale_MultiSliceConfig_NumSlices",
            ))?;

        let err = unsafe { ext_fn(&mut args) };
        self.api.err_or(err, ())?;
        Ok(args.num_slices)
    }

    /// Get the slice ID of this configuration.
    pub fn slice_id(&self) -> Result<i32> {
        let mut args: PJRT_Megascale_MultiSliceConfig_SliceId_Args = unsafe { std::mem::zeroed() };
        args.struct_size = std::mem::size_of::<PJRT_Megascale_MultiSliceConfig_SliceId_Args>();
        args.config = self.ptr;

        let ext_fn = self
            .ext
            .multi_slice_config_slice_id
            .ok_or(Error::NullFunctionPointer(
                "PJRT_Megascale_MultiSliceConfig_SliceId",
            ))?;

        let err = unsafe { ext_fn(&mut args) };
        self.api.err_or(err, ())?;
        Ok(args.slice_id)
    }

    /// Get the number of devices per slice.
    ///
    /// Returns a vector of (slice_id, num_devices) pairs.
    pub fn get_num_devices_per_slice(&self) -> Result<Vec<(i32, i32)>> {
        let mut args: PJRT_Megascale_MultiSliceConfig_GetNumDevicesPerSlice_Args =
            unsafe { std::mem::zeroed() };
        args.struct_size =
            std::mem::size_of::<PJRT_Megascale_MultiSliceConfig_GetNumDevicesPerSlice_Args>();
        args.config = self.ptr;

        let ext_fn = self
            .ext
            .multi_slice_config_get_num_devices_per_slice
            .ok_or(Error::NullFunctionPointer(
                "PJRT_Megascale_MultiSliceConfig_GetNumDevicesPerSlice",
            ))?;

        let err = unsafe { ext_fn(&mut args) };
        self.api.err_or(err, ())?;

        let count = args.num_devices_per_slice_map;
        let result = if count > 0 && !args.slice_ids.is_null() && !args.num_devices.is_null() {
            let slice_ids = unsafe { std::slice::from_raw_parts(args.slice_ids, count) };
            let num_devices = unsafe { std::slice::from_raw_parts(args.num_devices, count) };
            slice_ids
                .iter()
                .zip(num_devices.iter())
                .map(|(&sid, &nd)| (sid, nd))
                .collect()
        } else {
            Vec::new()
        };

        // Clean up
        if let Some(deleter) = args.devices_per_slice_map_deleter {
            if !args.devices_per_slice_map_ptr.is_null() {
                unsafe { deleter(args.devices_per_slice_map_ptr) };
            }
        }

        Ok(result)
    }

    /// Serialize this multi-slice config.
    ///
    /// Returns the serialized bytes.
    pub fn serialize(&self) -> Result<Vec<u8>> {
        let mut args: PJRT_Megascale_MultiSliceConfig_Serialize_Args =
            unsafe { std::mem::zeroed() };
        args.struct_size = std::mem::size_of::<PJRT_Megascale_MultiSliceConfig_Serialize_Args>();
        args.config = self.ptr;

        let ext_fn = self
            .ext
            .multi_slice_config_serialize
            .ok_or(Error::NullFunctionPointer(
                "PJRT_Megascale_MultiSliceConfig_Serialize",
            ))?;

        let err = unsafe { ext_fn(&mut args) };
        self.api.err_or(err, ())?;

        let data = if args.serialized.is_null() || args.size == 0 {
            Vec::new()
        } else {
            unsafe { std::slice::from_raw_parts(args.serialized as *const u8, args.size) }.to_vec()
        };

        // Clean up
        if let Some(deleter) = args.serialized_config_deleter {
            if !args.serialized_config_ptr.is_null() {
                unsafe { deleter(args.serialized_config_ptr) };
            }
        }

        Ok(data)
    }
}

impl Drop for MegascaleMultiSliceConfig {
    fn drop(&mut self) {
        if let Some(ext_fn) = self.ext.delete_multi_slice_config {
            let mut args: PJRT_Megascale_DeleteMultiSliceConfig_Args =
                unsafe { std::mem::zeroed() };
            args.struct_size = std::mem::size_of::<PJRT_Megascale_DeleteMultiSliceConfig_Args>();
            args.multi_slice_config = self.ptr;
            unsafe { ext_fn(&mut args) };
        }
    }
}

/// Safe wrapper for PJRT Megascale extension.
///
/// This extension provides capabilities for large-scale distributed training,
/// including:
///
/// - **Client context management**: Create contexts from PJRT clients or defaults
/// - **Multi-slice configuration**: Configure AoT or runtime multi-slice topologies
/// - **Query operations**: Get slice counts, IDs, device counts per slice
/// - **Serialization**: Serialize multi-slice configs for transfer
///
/// ## Availability
///
/// This extension is typically only available in PJRT plugins designed for
/// large-scale distributed execution, such as TPU pod plugins.
pub struct MegascaleExtension {
    raw: Rc<PJRT_Megascale_Extension>,
    api: Api,
}

impl std::fmt::Debug for MegascaleExtension {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MegascaleExtension")
            .field("type", &"Megascale")
            .finish()
    }
}

unsafe impl Extension for MegascaleExtension {
    fn extension_type() -> ExtensionType {
        ExtensionType::Megascale
    }

    unsafe fn from_raw(ptr: *mut pjrt_sys::PJRT_Extension_Base, api: &Api) -> Option<Self>
    where
        Self: Sized,
    {
        if ptr.is_null() {
            return None;
        }

        if (*ptr).type_ != ExtensionType::Megascale.to_raw() {
            return None;
        }

        let ext = ptr as *mut PJRT_Megascale_Extension;
        Some(Self {
            raw: Rc::new(*ext),
            api: api.clone(),
        })
    }
}

impl MegascaleExtension {
    /// Returns the raw extension pointer.
    pub fn raw_ptr(&self) -> *mut pjrt_sys::PJRT_Extension_Base {
        &self.raw.base as *const pjrt_sys::PJRT_Extension_Base as *mut pjrt_sys::PJRT_Extension_Base
    }

    /// Create a Megascale client context from an existing PJRT client.
    ///
    /// The returned context is automatically destroyed on drop.
    pub fn create_client_context_from_pjrt_client(
        &self,
        client: &Client,
    ) -> Result<MegascaleClientContext> {
        let mut args: PJRT_Megascale_CreateClientContextFromPjRtClient_Args =
            unsafe { std::mem::zeroed() };
        args.struct_size =
            std::mem::size_of::<PJRT_Megascale_CreateClientContextFromPjRtClient_Args>();
        args.client = client.ptr();

        let ext_fn =
            self.raw
                .create_client_context_from_pjrt_client
                .ok_or(Error::NullFunctionPointer(
                    "PJRT_Megascale_CreateClientContextFromPjRtClient",
                ))?;

        let err = unsafe { ext_fn(&mut args) };
        self.api.err_or(err, ())?;

        Ok(MegascaleClientContext {
            ptr: args.client_context,
            ext: Rc::clone(&self.raw),
            api: self.api.clone(),
        })
    }

    /// Create a Megascale client context with default options.
    ///
    /// The returned context is automatically destroyed on drop.
    pub fn create_default_client_context(&self) -> Result<MegascaleClientContext> {
        let mut args: PJRT_Megascale_CreateDefaultClientContext_Args =
            unsafe { std::mem::zeroed() };
        args.struct_size = std::mem::size_of::<PJRT_Megascale_CreateDefaultClientContext_Args>();

        let ext_fn = self
            .raw
            .create_default_client_context
            .ok_or(Error::NullFunctionPointer(
                "PJRT_Megascale_CreateDefaultClientContext",
            ))?;

        let err = unsafe { ext_fn(&mut args) };
        self.api.err_or(err, ())?;

        Ok(MegascaleClientContext {
            ptr: args.client_context,
            ext: Rc::clone(&self.raw),
            api: self.api.clone(),
        })
    }

    /// Delete a Megascale client context explicitly.
    ///
    /// This consumes the context. Prefer letting the context drop naturally.
    pub fn delete_client_context(&self, ctx: MegascaleClientContext) -> Result<()> {
        let ctx = std::mem::ManuallyDrop::new(ctx);
        let mut args: PJRT_Megascale_DeleteClientContext_Args = unsafe { std::mem::zeroed() };
        args.struct_size = std::mem::size_of::<PJRT_Megascale_DeleteClientContext_Args>();
        args.client_context = ctx.ptr;

        let ext_fn = self
            .raw
            .delete_client_context
            .ok_or(Error::NullFunctionPointer(
                "PJRT_Megascale_DeleteClientContext",
            ))?;

        let err = unsafe { ext_fn(&mut args) };
        self.api.err_or(err, ())
    }

    /// Create an ahead-of-time (AoT) multi-slice configuration.
    ///
    /// # Arguments
    ///
    /// * `topology` - The topology description
    /// * `num_slices` - Number of slices
    ///
    /// The returned config is automatically destroyed on drop.
    pub fn create_aot_config(
        &self,
        topology: &TopologyDescription,
        num_slices: i32,
    ) -> Result<MegascaleMultiSliceConfig> {
        let mut args: PJRT_Megascale_CreateAoTConfig_Args = unsafe { std::mem::zeroed() };
        args.struct_size = std::mem::size_of::<PJRT_Megascale_CreateAoTConfig_Args>();
        args.topology = topology.ptr;
        args.num_slices = num_slices;

        let ext_fn = self
            .raw
            .create_aot_config
            .ok_or(Error::NullFunctionPointer("PJRT_Megascale_CreateAoTConfig"))?;

        let err = unsafe { ext_fn(&mut args) };
        self.api.err_or(err, ())?;

        Ok(MegascaleMultiSliceConfig {
            ptr: args.multi_slice_config,
            ext: Rc::clone(&self.raw),
            api: self.api.clone(),
        })
    }

    /// Create a runtime multi-slice configuration.
    ///
    /// # Arguments
    ///
    /// * `topology` - The topology description
    /// * `num_slices` - Number of slices
    /// * `local_slice_id` - Local slice ID
    /// * `local_host_id` - Local host ID
    /// * `endpoint_addresses` - Serialized endpoint addresses proto
    /// * `dcn_topology` - Serialized DCN topology proto
    /// * `client_context` - Megascale client context
    ///
    /// The returned config is automatically destroyed on drop.
    #[allow(clippy::too_many_arguments)]
    pub fn create_multi_slice_config(
        &self,
        topology: &TopologyDescription,
        num_slices: i32,
        local_slice_id: i32,
        local_host_id: i32,
        endpoint_addresses: &[u8],
        dcn_topology: &[u8],
        client_context: &MegascaleClientContext,
    ) -> Result<MegascaleMultiSliceConfig> {
        let mut args: PJRT_Megascale_CreateMultiSliceConfig_Args = unsafe { std::mem::zeroed() };
        args.struct_size = std::mem::size_of::<PJRT_Megascale_CreateMultiSliceConfig_Args>();
        args.topology = topology.ptr;
        args.num_slices = num_slices;
        args.local_slice_id = local_slice_id;
        args.local_host_id = local_host_id;
        args.endpoint_addresses = endpoint_addresses.as_ptr() as *const i8;
        args.endpoint_addresses_size = endpoint_addresses.len() as i32;
        args.dcn_topology = dcn_topology.as_ptr() as *const i8;
        args.dcn_topology_size = dcn_topology.len() as i32;
        args.client_context = client_context.ptr;

        let ext_fn = self
            .raw
            .create_multi_slice_config
            .ok_or(Error::NullFunctionPointer(
                "PJRT_Megascale_CreateMultiSliceConfig",
            ))?;

        let err = unsafe { ext_fn(&mut args) };
        self.api.err_or(err, ())?;

        Ok(MegascaleMultiSliceConfig {
            ptr: args.multi_slice_config,
            ext: Rc::clone(&self.raw),
            api: self.api.clone(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extension_type() {
        assert_eq!(
            MegascaleExtension::extension_type(),
            ExtensionType::Megascale
        );
    }

    #[test]
    fn test_from_raw_null_returns_none() {
        let api = unsafe { Api::empty_for_testing() };
        let result = unsafe { MegascaleExtension::from_raw(std::ptr::null_mut(), &api) };
        assert!(result.is_none());
    }

    #[test]
    fn test_from_raw_wrong_type_returns_none() {
        let api = unsafe { Api::empty_for_testing() };
        let mut ext: PJRT_Megascale_Extension = unsafe { std::mem::zeroed() };
        ext.base.struct_size = std::mem::size_of::<PJRT_Megascale_Extension>();
        ext.base.type_ = ExtensionType::Example.to_raw();
        let result = unsafe {
            MegascaleExtension::from_raw(&mut ext.base as *mut pjrt_sys::PJRT_Extension_Base, &api)
        };
        assert!(result.is_none());
    }

    #[test]
    fn test_from_raw_correct_type() {
        let api = unsafe { Api::empty_for_testing() };
        let mut ext: PJRT_Megascale_Extension = unsafe { std::mem::zeroed() };
        ext.base.struct_size = std::mem::size_of::<PJRT_Megascale_Extension>();
        ext.base.type_ = ExtensionType::Megascale.to_raw();
        let result = unsafe {
            MegascaleExtension::from_raw(&mut ext.base as *mut pjrt_sys::PJRT_Extension_Base, &api)
        };
        assert!(result.is_some());
    }

    #[test]
    fn test_null_fn_pointer_create_client_context_from_pjrt_client() {
        let api = unsafe { Api::empty_for_testing() };
        let mut ext: PJRT_Megascale_Extension = unsafe { std::mem::zeroed() };
        ext.base.struct_size = std::mem::size_of::<PJRT_Megascale_Extension>();
        ext.base.type_ = ExtensionType::Megascale.to_raw();
        let wrapper = unsafe {
            MegascaleExtension::from_raw(&mut ext.base as *mut pjrt_sys::PJRT_Extension_Base, &api)
        }
        .unwrap();
        // Can't call the method without a real client, but we test the fn pointer check
        let result = wrapper.create_default_client_context();
        assert!(result.is_err());
    }

    #[test]
    fn test_debug_format() {
        let api = unsafe { Api::empty_for_testing() };
        let mut ext: PJRT_Megascale_Extension = unsafe { std::mem::zeroed() };
        ext.base.struct_size = std::mem::size_of::<PJRT_Megascale_Extension>();
        ext.base.type_ = ExtensionType::Megascale.to_raw();
        let wrapper = unsafe {
            MegascaleExtension::from_raw(&mut ext.base as *mut pjrt_sys::PJRT_Extension_Base, &api)
        }
        .unwrap();
        let debug = format!("{:?}", wrapper);
        assert!(debug.contains("MegascaleExtension"));
        assert!(debug.contains("Megascale"));
    }
}
