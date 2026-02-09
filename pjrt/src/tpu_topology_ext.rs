//! PJRT TPU Topology Extension
//!
//! This module provides safe Rust bindings for the PJRT TPU Topology extension.
//! The TPU Topology extension provides detailed TPU-specific topology queries.
//!
//! ## Overview
//!
//! This extension is primarily used with TPU devices and provides capabilities for:
//!
//! - Querying process, chip, core, and logical device counts
//! - Subslice topology extraction and device ID mapping
//! - Coordinate-based chip and device lookups
//! - Topology bounds queries (chips per process, chip bounds, process bounds)
//! - ICI connectivity and barrier information
//! - Slice configuration queries
//!
//! ## Usage
//!
//! ```rust,ignore
//! use pjrt::TpuTopologyExtension;
//!
//! if let Some(ext) = api.get_extension::<TpuTopologyExtension>() {
//!     let count = ext.process_count(&topology)?;
//!     println!("Process count: {}", count);
//!     let chips = ext.chip_count(&topology)?;
//!     println!("Chip count: {}", chips);
//! }
//! ```
//!
//! ## Note
//!
//! This extension is only available in TPU PJRT plugins.

use std::borrow::Cow;
use std::rc::Rc;

use pjrt_sys::{
    PJRT_TpuTopology_ChipBounds_Args, PJRT_TpuTopology_ChipCoordAndIdxForLogiDevice_Args,
    PJRT_TpuTopology_ChipCount_Args, PJRT_TpuTopology_ChipIdFromCoord_Args,
    PJRT_TpuTopology_ChipsPerProcessBounds_Args, PJRT_TpuTopology_ChipsPerProcess_Args,
    PJRT_TpuTopology_CoreCountPerChip_Args, PJRT_TpuTopology_CoreCountPerProcess_Args,
    PJRT_TpuTopology_CoreCount_Args, PJRT_TpuTopology_Extension,
    PJRT_TpuTopology_GetDefaultPlatformConfig_Args, PJRT_TpuTopology_GetRoutingStrategy_Args,
    PJRT_TpuTopology_GetSliceConfig_Args, PJRT_TpuTopology_GetSliceConfigs_Args,
    PJRT_TpuTopology_HasLimitedIciConnectivity_Args,
    PJRT_TpuTopology_IsEnhancedBarrierEnabled_Args,
    PJRT_TpuTopology_IsReachableOverLimitedIci_Args, PJRT_TpuTopology_IsSubsliceTopology_Args,
    PJRT_TpuTopology_LogiDeviceCountPerChip_Args, PJRT_TpuTopology_LogiDeviceCountPerProcess_Args,
    PJRT_TpuTopology_LogiDeviceCount_Args, PJRT_TpuTopology_LogiDeviceIdFromChipCoordAndIdx_Args,
    PJRT_TpuTopology_LogiDeviceIdsOnProcess_Args, PJRT_TpuTopology_ProcIdAndIdxOnProcForChip_Args,
    PJRT_TpuTopology_ProcIdAndIdxOnProcForLogiDevice_Args, PJRT_TpuTopology_ProcessBounds_Args,
    PJRT_TpuTopology_ProcessCoordFromId_Args, PJRT_TpuTopology_ProcessCount_Args,
    PJRT_TpuTopology_ProcessIds_Args, PJRT_TpuTopology_ReplaceHostBounds_Args,
    PJRT_TpuTopology_SliceConfig, PJRT_TpuTopology_SubsliceDeviceIdFromFullDeviceId_Args,
    PJRT_TpuTopology_Subslice_Args,
};

use crate::extension::{Extension, ExtensionType};
use crate::{Api, Error, Result, TopologyDescription};

/// TPU slice configuration.
///
/// Describes the dimensional layout and wrapping behavior of a TPU slice.
#[derive(Debug, Clone)]
pub struct SliceConfig {
    /// Number of dimensions in this config.
    pub dim_size: usize,
    /// Dimension sizes (up to 4 elements, only `dim_size` are valid).
    pub dimensions: [i32; 4],
    /// Whether each dimension wraps around (up to 4 elements).
    pub wrap: [bool; 4],
    /// Whether the topology has a twist.
    pub twist: bool,
}

impl From<&PJRT_TpuTopology_SliceConfig> for SliceConfig {
    fn from(c: &PJRT_TpuTopology_SliceConfig) -> Self {
        Self {
            dim_size: c.dim_size,
            dimensions: c.dimensions,
            wrap: c.wrap,
            twist: c.twist,
        }
    }
}

/// Default platform configuration result.
#[derive(Debug, Clone)]
pub struct DefaultPlatformConfig {
    /// Number of chips per tray.
    pub num_chips_per_tray: i64,
    /// Number of trays.
    pub num_trays: i64,
}

/// Safe wrapper for PJRT TPU Topology extension.
///
/// This extension provides detailed TPU-specific topology queries covering
/// process counts, chip/core/device counts, coordinate-based lookups,
/// bounds queries, ICI connectivity, and slice configurations.
///
/// ## Availability
///
/// This extension is only available in TPU PJRT plugins.
pub struct TpuTopologyExtension {
    raw: Rc<PJRT_TpuTopology_Extension>,
    api: Api,
}

impl std::fmt::Debug for TpuTopologyExtension {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TpuTopologyExtension")
            .field("type", &"TpuTopology")
            .finish()
    }
}

unsafe impl Extension for TpuTopologyExtension {
    fn extension_type() -> ExtensionType {
        ExtensionType::TpuTopology
    }

    unsafe fn from_raw(ptr: *mut pjrt_sys::PJRT_Extension_Base, api: &Api) -> Option<Self>
    where
        Self: Sized,
    {
        if ptr.is_null() {
            return None;
        }

        if (*ptr).type_ != ExtensionType::TpuTopology.to_raw() {
            return None;
        }

        let ext = ptr as *mut PJRT_TpuTopology_Extension;
        Some(Self {
            raw: Rc::new(*ext),
            api: api.clone(),
        })
    }
}

impl TpuTopologyExtension {
    /// Returns the raw extension pointer.
    pub fn raw_ptr(&self) -> *mut pjrt_sys::PJRT_Extension_Base {
        &self.raw.base as *const pjrt_sys::PJRT_Extension_Base as *mut pjrt_sys::PJRT_Extension_Base
    }

    // ─── Subslice operations ────────────────────────────────────────────

    /// Create a subslice topology from the given topology.
    ///
    /// The caller owns the returned topology and is responsible for destruction.
    ///
    /// # Arguments
    ///
    /// * `topology` - The parent topology
    /// * `chips_per_host_bounds` - Bounds array for chips per host
    /// * `host_bounds` - Host bounds array
    pub fn subslice(
        &self,
        topology: &TopologyDescription,
        chips_per_host_bounds: &[i32],
        host_bounds: &[i32],
    ) -> Result<TopologyDescription> {
        let mut args: PJRT_TpuTopology_Subslice_Args = unsafe { std::mem::zeroed() };
        args.struct_size = std::mem::size_of::<PJRT_TpuTopology_Subslice_Args>();
        args.topology = topology.ptr;
        args.chips_per_host_bounds = chips_per_host_bounds.as_ptr();
        args.chips_per_host_bounds_num_dims = chips_per_host_bounds.len();
        args.host_bounds = host_bounds.as_ptr();
        args.host_bounds_num_dims = host_bounds.len();

        let ext_fn = self
            .raw
            .subslice
            .ok_or(Error::NullFunctionPointer("PJRT_TpuTopology_Subslice"))?;

        let err = unsafe { ext_fn(&mut args) };
        self.api.err_or(err, ())?;

        Ok(TopologyDescription {
            api: self.api.clone(),
            client: None,
            ptr: args.subslice_topology,
        })
    }

    /// Check if the topology is a subslice topology.
    pub fn is_subslice_topology(&self, topology: &TopologyDescription) -> Result<bool> {
        let mut args: PJRT_TpuTopology_IsSubsliceTopology_Args = unsafe { std::mem::zeroed() };
        args.struct_size = std::mem::size_of::<PJRT_TpuTopology_IsSubsliceTopology_Args>();
        args.topology = topology.ptr;

        let ext_fn = self
            .raw
            .is_subslice_topology
            .ok_or(Error::NullFunctionPointer(
                "PJRT_TpuTopology_IsSubsliceTopology",
            ))?;

        let err = unsafe { ext_fn(&mut args) };
        self.api.err_or(err, ())?;
        Ok(args.is_subslice_topology)
    }

    /// Get the subslice device ID corresponding to a full device ID.
    pub fn subslice_device_id_from_full_device_id(
        &self,
        client_topology: &TopologyDescription,
        subslice_topology: &TopologyDescription,
        subslice_origin: &[i32],
        full_device_id: i32,
    ) -> Result<i32> {
        let mut args: PJRT_TpuTopology_SubsliceDeviceIdFromFullDeviceId_Args =
            unsafe { std::mem::zeroed() };
        args.struct_size =
            std::mem::size_of::<PJRT_TpuTopology_SubsliceDeviceIdFromFullDeviceId_Args>();
        args.client_topology = client_topology.ptr;
        args.subslice_topology = subslice_topology.ptr;
        args.subslice_origin = subslice_origin.as_ptr();
        args.subslice_origin_dim_num = subslice_origin.len();
        args.full_device_id = full_device_id;

        let ext_fn =
            self.raw
                .subslice_device_id_from_full_device_id
                .ok_or(Error::NullFunctionPointer(
                    "PJRT_TpuTopology_SubsliceDeviceIdFromFullDeviceId",
                ))?;

        let err = unsafe { ext_fn(&mut args) };
        self.api.err_or(err, ())?;
        Ok(args.subslice_device_id)
    }

    /// Replace the host bounds of a topology and return a new topology.
    ///
    /// The caller owns the returned topology.
    pub fn replace_host_bounds(
        &self,
        topology: &TopologyDescription,
        host_bounds: &[i32],
    ) -> Result<TopologyDescription> {
        let mut args: PJRT_TpuTopology_ReplaceHostBounds_Args = unsafe { std::mem::zeroed() };
        args.struct_size = std::mem::size_of::<PJRT_TpuTopology_ReplaceHostBounds_Args>();
        args.topology = topology.ptr;
        args.host_bounds = host_bounds.as_ptr();
        args.host_bounds_dim_num = host_bounds.len();

        let ext_fn = self
            .raw
            .replace_host_bounds
            .ok_or(Error::NullFunctionPointer(
                "PJRT_TpuTopology_ReplaceHostBounds",
            ))?;

        let err = unsafe { ext_fn(&mut args) };
        self.api.err_or(err, ())?;

        Ok(TopologyDescription {
            api: self.api.clone(),
            client: None,
            ptr: args.new_topology,
        })
    }

    // ─── Boolean queries ────────────────────────────────────────────────

    /// Check if enhanced barrier is enabled for this topology.
    pub fn is_enhanced_barrier_enabled(&self, topology: &TopologyDescription) -> Result<bool> {
        let mut args: PJRT_TpuTopology_IsEnhancedBarrierEnabled_Args =
            unsafe { std::mem::zeroed() };
        args.struct_size = std::mem::size_of::<PJRT_TpuTopology_IsEnhancedBarrierEnabled_Args>();
        args.topology = topology.ptr;

        let ext_fn = self
            .raw
            .is_enhanced_barrier_enabled
            .ok_or(Error::NullFunctionPointer(
                "PJRT_TpuTopology_IsEnhancedBarrierEnabled",
            ))?;

        let err = unsafe { ext_fn(&mut args) };
        self.api.err_or(err, ())?;
        Ok(args.is_enhanced_barrier_enabled)
    }

    /// Check if this topology has limited ICI connectivity.
    pub fn has_limited_ici_connectivity(&self, topology: &TopologyDescription) -> Result<bool> {
        let mut args: PJRT_TpuTopology_HasLimitedIciConnectivity_Args =
            unsafe { std::mem::zeroed() };
        args.struct_size = std::mem::size_of::<PJRT_TpuTopology_HasLimitedIciConnectivity_Args>();
        args.topology = topology.ptr;

        let ext_fn = self
            .raw
            .has_limited_ici_connectivity
            .ok_or(Error::NullFunctionPointer(
                "PJRT_TpuTopology_HasLimitedIciConnectivity",
            ))?;

        let err = unsafe { ext_fn(&mut args) };
        self.api.err_or(err, ())?;
        Ok(args.has_limited_ici_connectivity)
    }

    /// Check if a chip can directly reach another chip over limited ICI.
    pub fn is_reachable_over_limited_ici(
        &self,
        topology: &TopologyDescription,
        source_chip_id: i32,
        dest_chip_id: i32,
    ) -> Result<bool> {
        let mut args: PJRT_TpuTopology_IsReachableOverLimitedIci_Args =
            unsafe { std::mem::zeroed() };
        args.struct_size = std::mem::size_of::<PJRT_TpuTopology_IsReachableOverLimitedIci_Args>();
        args.topology = topology.ptr;
        args.source_chip_id = source_chip_id;
        args.dest_chip_id = dest_chip_id;

        let ext_fn = self
            .raw
            .is_reachable_over_limited_ici
            .ok_or(Error::NullFunctionPointer(
                "PJRT_TpuTopology_IsReachableOverLimitedIci",
            ))?;

        let err = unsafe { ext_fn(&mut args) };
        self.api.err_or(err, ())?;
        Ok(args.is_reachable_over_limited_ici)
    }

    // ─── Count queries ──────────────────────────────────────────────────

    /// Get the number of processes in this topology.
    pub fn process_count(&self, topology: &TopologyDescription) -> Result<i32> {
        let mut args: PJRT_TpuTopology_ProcessCount_Args = unsafe { std::mem::zeroed() };
        args.struct_size = std::mem::size_of::<PJRT_TpuTopology_ProcessCount_Args>();
        args.topology = topology.ptr;

        let ext_fn = self
            .raw
            .process_count
            .ok_or(Error::NullFunctionPointer("PJRT_TpuTopology_ProcessCount"))?;

        let err = unsafe { ext_fn(&mut args) };
        self.api.err_or(err, ())?;
        Ok(args.process_count)
    }

    /// Get the number of chips per process.
    pub fn chips_per_process(&self, topology: &TopologyDescription) -> Result<i32> {
        let mut args: PJRT_TpuTopology_ChipsPerProcess_Args = unsafe { std::mem::zeroed() };
        args.struct_size = std::mem::size_of::<PJRT_TpuTopology_ChipsPerProcess_Args>();
        args.topology = topology.ptr;

        let ext_fn = self
            .raw
            .chips_per_process
            .ok_or(Error::NullFunctionPointer(
                "PJRT_TpuTopology_ChipsPerProcess",
            ))?;

        let err = unsafe { ext_fn(&mut args) };
        self.api.err_or(err, ())?;
        Ok(args.chips_per_process)
    }

    /// Get the number of cores of default type per chip.
    pub fn core_count_per_chip(&self, topology: &TopologyDescription) -> Result<i32> {
        let mut args: PJRT_TpuTopology_CoreCountPerChip_Args = unsafe { std::mem::zeroed() };
        args.struct_size = std::mem::size_of::<PJRT_TpuTopology_CoreCountPerChip_Args>();
        args.topology = topology.ptr;

        let ext_fn = self
            .raw
            .core_count_per_chip
            .ok_or(Error::NullFunctionPointer(
                "PJRT_TpuTopology_CoreCountPerChip",
            ))?;

        let err = unsafe { ext_fn(&mut args) };
        self.api.err_or(err, ())?;
        Ok(args.core_count_of_default_type_per_chip)
    }

    /// Get the total number of chips in this topology.
    pub fn chip_count(&self, topology: &TopologyDescription) -> Result<i32> {
        let mut args: PJRT_TpuTopology_ChipCount_Args = unsafe { std::mem::zeroed() };
        args.struct_size = std::mem::size_of::<PJRT_TpuTopology_ChipCount_Args>();
        args.topology = topology.ptr;

        let ext_fn = self
            .raw
            .chip_count
            .ok_or(Error::NullFunctionPointer("PJRT_TpuTopology_ChipCount"))?;

        let err = unsafe { ext_fn(&mut args) };
        self.api.err_or(err, ())?;
        Ok(args.chip_count)
    }

    /// Get the total number of cores of default type.
    pub fn core_count(&self, topology: &TopologyDescription) -> Result<i32> {
        let mut args: PJRT_TpuTopology_CoreCount_Args = unsafe { std::mem::zeroed() };
        args.struct_size = std::mem::size_of::<PJRT_TpuTopology_CoreCount_Args>();
        args.topology = topology.ptr;

        let ext_fn = self
            .raw
            .core_count
            .ok_or(Error::NullFunctionPointer("PJRT_TpuTopology_CoreCount"))?;

        let err = unsafe { ext_fn(&mut args) };
        self.api.err_or(err, ())?;
        Ok(args.core_count_of_default_type)
    }

    /// Get the total number of logical devices of default type.
    pub fn logical_device_count(&self, topology: &TopologyDescription) -> Result<i32> {
        let mut args: PJRT_TpuTopology_LogiDeviceCount_Args = unsafe { std::mem::zeroed() };
        args.struct_size = std::mem::size_of::<PJRT_TpuTopology_LogiDeviceCount_Args>();
        args.topology = topology.ptr;

        let ext_fn = self
            .raw
            .logical_device_count
            .ok_or(Error::NullFunctionPointer(
                "PJRT_TpuTopology_LogiDeviceCount",
            ))?;

        let err = unsafe { ext_fn(&mut args) };
        self.api.err_or(err, ())?;
        Ok(args.logical_device_count_of_default_type)
    }

    /// Get the number of logical devices of default type per process.
    pub fn logical_device_count_per_process(&self, topology: &TopologyDescription) -> Result<i32> {
        let mut args: PJRT_TpuTopology_LogiDeviceCountPerProcess_Args =
            unsafe { std::mem::zeroed() };
        args.struct_size = std::mem::size_of::<PJRT_TpuTopology_LogiDeviceCountPerProcess_Args>();
        args.topology = topology.ptr;

        let ext_fn =
            self.raw
                .logical_device_count_per_process
                .ok_or(Error::NullFunctionPointer(
                    "PJRT_TpuTopology_LogiDeviceCountPerProcess",
                ))?;

        let err = unsafe { ext_fn(&mut args) };
        self.api.err_or(err, ())?;
        Ok(args.logical_device_count_of_default_type_per_process)
    }

    /// Get the number of logical devices of default type per chip.
    pub fn logical_device_count_per_chip(&self, topology: &TopologyDescription) -> Result<i32> {
        let mut args: PJRT_TpuTopology_LogiDeviceCountPerChip_Args = unsafe { std::mem::zeroed() };
        args.struct_size = std::mem::size_of::<PJRT_TpuTopology_LogiDeviceCountPerChip_Args>();
        args.topology = topology.ptr;

        let ext_fn = self
            .raw
            .logical_device_count_per_chip
            .ok_or(Error::NullFunctionPointer(
                "PJRT_TpuTopology_LogiDeviceCountPerChip",
            ))?;

        let err = unsafe { ext_fn(&mut args) };
        self.api.err_or(err, ())?;
        Ok(args.logical_device_count_of_default_type_per_chip)
    }

    /// Get the number of cores per process.
    pub fn core_count_per_process(&self, topology: &TopologyDescription) -> Result<i32> {
        let mut args: PJRT_TpuTopology_CoreCountPerProcess_Args = unsafe { std::mem::zeroed() };
        args.struct_size = std::mem::size_of::<PJRT_TpuTopology_CoreCountPerProcess_Args>();
        args.topology = topology.ptr;

        let ext_fn = self
            .raw
            .core_count_per_process
            .ok_or(Error::NullFunctionPointer(
                "PJRT_TpuTopology_CoreCountPerProcess",
            ))?;

        let err = unsafe { ext_fn(&mut args) };
        self.api.err_or(err, ())?;
        Ok(args.core_count_of_default_type_per_process)
    }

    // ─── ID-based queries ───────────────────────────────────────────────

    /// Get the process IDs in this topology.
    ///
    /// # Arguments
    ///
    /// * `topology` - The topology to query
    /// * `max_process_ids` - Maximum number of process IDs to return
    pub fn process_ids(
        &self,
        topology: &TopologyDescription,
        max_process_ids: i32,
    ) -> Result<Vec<i32>> {
        let mut buf = vec![0i32; max_process_ids as usize];
        let mut args: PJRT_TpuTopology_ProcessIds_Args = unsafe { std::mem::zeroed() };
        args.struct_size = std::mem::size_of::<PJRT_TpuTopology_ProcessIds_Args>();
        args.topology = topology.ptr;
        args.max_process_ids = max_process_ids;
        args.process_ids = buf.as_mut_ptr();

        let ext_fn = self
            .raw
            .process_ids
            .ok_or(Error::NullFunctionPointer("PJRT_TpuTopology_ProcessIds"))?;

        let err = unsafe { ext_fn(&mut args) };
        self.api.err_or(err, ())?;
        buf.truncate(args.num_process_ids);
        Ok(buf)
    }

    /// Get logical device IDs on a given process.
    ///
    /// # Arguments
    ///
    /// * `topology` - The topology to query
    /// * `process_id` - The process ID
    /// * `max_ids` - Maximum number of device IDs to return
    pub fn logical_device_ids_on_process(
        &self,
        topology: &TopologyDescription,
        process_id: i32,
        max_ids: i32,
    ) -> Result<Vec<i32>> {
        let mut buf = vec![0i32; max_ids as usize];
        let mut args: PJRT_TpuTopology_LogiDeviceIdsOnProcess_Args = unsafe { std::mem::zeroed() };
        args.struct_size = std::mem::size_of::<PJRT_TpuTopology_LogiDeviceIdsOnProcess_Args>();
        args.topology = topology.ptr;
        args.process_id = process_id;
        args.max_logical_device_ids = max_ids;
        args.logical_device_of_default_type_ids = buf.as_mut_ptr();

        let ext_fn = self
            .raw
            .logical_device_ids_on_process
            .ok_or(Error::NullFunctionPointer(
                "PJRT_TpuTopology_LogiDeviceIdsOnProcess",
            ))?;

        let err = unsafe { ext_fn(&mut args) };
        self.api.err_or(err, ())?;
        buf.truncate(args.num_logical_device_ids);
        Ok(buf)
    }

    /// Get the process ID and index on process for a given chip.
    ///
    /// Returns `(process_id, index_on_process)`.
    pub fn proc_id_and_idx_on_proc_for_chip(
        &self,
        topology: &TopologyDescription,
        chip_id: i32,
    ) -> Result<(i32, i32)> {
        let mut args: PJRT_TpuTopology_ProcIdAndIdxOnProcForChip_Args =
            unsafe { std::mem::zeroed() };
        args.struct_size = std::mem::size_of::<PJRT_TpuTopology_ProcIdAndIdxOnProcForChip_Args>();
        args.topology = topology.ptr;
        args.chip_id = chip_id;

        let ext_fn =
            self.raw
                .proc_id_and_idx_on_proc_for_chip
                .ok_or(Error::NullFunctionPointer(
                    "PJRT_TpuTopology_ProcIdAndIdxOnProcForChip",
                ))?;

        let err = unsafe { ext_fn(&mut args) };
        self.api.err_or(err, ())?;
        Ok((args.process_id, args.index_on_process))
    }

    /// Get the process ID and index on process for a given logical device.
    ///
    /// Returns `(process_id, index_on_process)`.
    pub fn proc_id_and_idx_on_proc_for_logi_device(
        &self,
        topology: &TopologyDescription,
        device_id: i32,
    ) -> Result<(i32, i32)> {
        let mut args: PJRT_TpuTopology_ProcIdAndIdxOnProcForLogiDevice_Args =
            unsafe { std::mem::zeroed() };
        args.struct_size =
            std::mem::size_of::<PJRT_TpuTopology_ProcIdAndIdxOnProcForLogiDevice_Args>();
        args.topology = topology.ptr;
        args.device_id = device_id;

        let ext_fn =
            self.raw
                .proc_id_and_idx_on_proc_for_logi_device
                .ok_or(Error::NullFunctionPointer(
                    "PJRT_TpuTopology_ProcIdAndIdxOnProcForLogiDevice",
                ))?;

        let err = unsafe { ext_fn(&mut args) };
        self.api.err_or(err, ())?;
        Ok((args.process_id, args.index_on_process))
    }

    // ─── Coordinate queries ─────────────────────────────────────────────

    /// Get the coordinates of a process from its ID.
    ///
    /// # Arguments
    ///
    /// * `topology` - The topology to query
    /// * `process_id` - The process ID
    /// * `max_dims` - Maximum number of coordinate dimensions
    pub fn process_coord_from_id(
        &self,
        topology: &TopologyDescription,
        process_id: i32,
        max_dims: usize,
    ) -> Result<Vec<i32>> {
        let mut buf = vec![0i32; max_dims];
        let mut args: PJRT_TpuTopology_ProcessCoordFromId_Args = unsafe { std::mem::zeroed() };
        args.struct_size = std::mem::size_of::<PJRT_TpuTopology_ProcessCoordFromId_Args>();
        args.topology = topology.ptr;
        args.process_id = process_id;
        args.coords_max_dims = max_dims;
        args.coords = buf.as_mut_ptr();

        let ext_fn = self
            .raw
            .process_coord_from_id
            .ok_or(Error::NullFunctionPointer(
                "PJRT_TpuTopology_ProcessCoordFromId",
            ))?;

        let err = unsafe { ext_fn(&mut args) };
        self.api.err_or(err, ())?;
        buf.truncate(args.coords_num_dims);
        Ok(buf)
    }

    /// Get the chip ID from coordinates.
    pub fn chip_id_from_coord(
        &self,
        topology: &TopologyDescription,
        coords: &[i32],
    ) -> Result<i32> {
        let mut args: PJRT_TpuTopology_ChipIdFromCoord_Args = unsafe { std::mem::zeroed() };
        args.struct_size = std::mem::size_of::<PJRT_TpuTopology_ChipIdFromCoord_Args>();
        args.topology = topology.ptr;
        args.coords = coords.as_ptr();
        args.coords_num_dims = coords.len();

        let ext_fn = self
            .raw
            .chip_id_from_coord
            .ok_or(Error::NullFunctionPointer(
                "PJRT_TpuTopology_ChipIdFromCoord",
            ))?;

        let err = unsafe { ext_fn(&mut args) };
        self.api.err_or(err, ())?;
        Ok(args.chip_id)
    }

    /// Get the logical device ID from chip coordinates and index on chip.
    pub fn logical_device_id_from_chip_coord_and_idx(
        &self,
        topology: &TopologyDescription,
        chip_coords: &[i32],
        logical_device_index_on_chip: i32,
    ) -> Result<i32> {
        let mut args: PJRT_TpuTopology_LogiDeviceIdFromChipCoordAndIdx_Args =
            unsafe { std::mem::zeroed() };
        args.struct_size =
            std::mem::size_of::<PJRT_TpuTopology_LogiDeviceIdFromChipCoordAndIdx_Args>();
        args.topology = topology.ptr;
        args.chip_coords = chip_coords.as_ptr();
        args.chip_coords_num_dims = chip_coords.len();
        args.logical_device_index_on_chip = logical_device_index_on_chip;

        let ext_fn = self.raw.logical_device_id_from_chip_coord_and_idx.ok_or(
            Error::NullFunctionPointer("PJRT_TpuTopology_LogiDeviceIdFromChipCoordAndIdx"),
        )?;

        let err = unsafe { ext_fn(&mut args) };
        self.api.err_or(err, ())?;
        Ok(args.logical_device_of_default_type_id)
    }

    /// Get the chip coordinates and device index for a logical device.
    ///
    /// Returns `(chip_coords, device_index_on_chip)`.
    pub fn chip_coord_and_idx_for_logi_device(
        &self,
        topology: &TopologyDescription,
        device_id: i32,
        max_dims: usize,
    ) -> Result<(Vec<i32>, i32)> {
        let mut buf = vec![0i32; max_dims];
        let mut args: PJRT_TpuTopology_ChipCoordAndIdxForLogiDevice_Args =
            unsafe { std::mem::zeroed() };
        args.struct_size =
            std::mem::size_of::<PJRT_TpuTopology_ChipCoordAndIdxForLogiDevice_Args>();
        args.topology = topology.ptr;
        args.device_id = device_id;
        args.chip_coords_max_dims = max_dims;
        args.chip_coords = buf.as_mut_ptr();

        let ext_fn =
            self.raw
                .chip_coord_and_idx_for_logi_device
                .ok_or(Error::NullFunctionPointer(
                    "PJRT_TpuTopology_ChipCoordAndIdxForLogiDevice",
                ))?;

        let err = unsafe { ext_fn(&mut args) };
        self.api.err_or(err, ())?;
        buf.truncate(args.chip_coords_num_dims);
        Ok((buf, args.device_index_on_chip))
    }

    // ─── Bounds queries ─────────────────────────────────────────────────

    /// Get the chips-per-process bounds.
    pub fn chips_per_process_bounds(
        &self,
        topology: &TopologyDescription,
        max_dims: usize,
    ) -> Result<Vec<i32>> {
        let mut buf = vec![0i32; max_dims];
        let mut args: PJRT_TpuTopology_ChipsPerProcessBounds_Args = unsafe { std::mem::zeroed() };
        args.struct_size = std::mem::size_of::<PJRT_TpuTopology_ChipsPerProcessBounds_Args>();
        args.topology = topology.ptr;
        args.chip_per_process_bounds_max_dims = max_dims;
        args.chip_per_process_bounds = buf.as_mut_ptr();

        let ext_fn = self
            .raw
            .chips_per_process_bounds
            .ok_or(Error::NullFunctionPointer(
                "PJRT_TpuTopology_ChipsPerProcessBounds",
            ))?;

        let err = unsafe { ext_fn(&mut args) };
        self.api.err_or(err, ())?;
        buf.truncate(args.chip_per_process_bounds_num_dims);
        Ok(buf)
    }

    /// Get the chip bounds of this topology.
    pub fn chip_bounds(&self, topology: &TopologyDescription, max_dims: usize) -> Result<Vec<i32>> {
        let mut buf = vec![0i32; max_dims];
        let mut args: PJRT_TpuTopology_ChipBounds_Args = unsafe { std::mem::zeroed() };
        args.struct_size = std::mem::size_of::<PJRT_TpuTopology_ChipBounds_Args>();
        args.topology = topology.ptr;
        args.chip_bounds_max_dims = max_dims;
        args.chip_bounds = buf.as_mut_ptr();

        let ext_fn = self
            .raw
            .chip_bounds
            .ok_or(Error::NullFunctionPointer("PJRT_TpuTopology_ChipBounds"))?;

        let err = unsafe { ext_fn(&mut args) };
        self.api.err_or(err, ())?;
        buf.truncate(args.chip_bounds_num_dims);
        Ok(buf)
    }

    /// Get the process bounds of this topology.
    pub fn process_bounds(
        &self,
        topology: &TopologyDescription,
        max_dims: usize,
    ) -> Result<Vec<i32>> {
        let mut buf = vec![0i32; max_dims];
        let mut args: PJRT_TpuTopology_ProcessBounds_Args = unsafe { std::mem::zeroed() };
        args.struct_size = std::mem::size_of::<PJRT_TpuTopology_ProcessBounds_Args>();
        args.topology = topology.ptr;
        args.process_bounds_max_dims = max_dims;
        args.process_bounds = buf.as_mut_ptr();

        let ext_fn = self
            .raw
            .process_bounds
            .ok_or(Error::NullFunctionPointer("PJRT_TpuTopology_ProcessBounds"))?;

        let err = unsafe { ext_fn(&mut args) };
        self.api.err_or(err, ())?;
        buf.truncate(args.process_bounds_num_dims);
        Ok(buf)
    }

    // ─── Routing and slice config ───────────────────────────────────────

    /// Get the routing strategy as a string.
    ///
    /// The caller provides a buffer for the result. Returns the strategy string.
    pub fn get_routing_strategy(
        &self,
        topology: &TopologyDescription,
        max_len: usize,
    ) -> Result<Cow<'static, str>> {
        let mut buf = vec![0u8; max_len];
        let mut args: PJRT_TpuTopology_GetRoutingStrategy_Args = unsafe { std::mem::zeroed() };
        args.struct_size = std::mem::size_of::<PJRT_TpuTopology_GetRoutingStrategy_Args>();
        args.topology = topology.ptr;
        args.routing_strategy = buf.as_mut_ptr() as *mut i8;
        args.routing_strategy_len = max_len;

        let ext_fn = self
            .raw
            .get_routing_strategy
            .ok_or(Error::NullFunctionPointer(
                "PJRT_TpuTopology_GetRoutingStrategy",
            ))?;

        let err = unsafe { ext_fn(&mut args) };
        self.api.err_or(err, ())?;

        let actual_len = args.routing_strategy_len.min(max_len);
        buf.truncate(actual_len);
        Ok(Cow::Owned(String::from_utf8_lossy(&buf).into_owned()))
    }

    /// Get the slice config for a given platform and slice name.
    pub fn get_slice_config(
        &self,
        platform_type_name: &str,
        slice_name: &str,
    ) -> Result<SliceConfig> {
        let mut slice_config: PJRT_TpuTopology_SliceConfig = unsafe { std::mem::zeroed() };
        let mut args: PJRT_TpuTopology_GetSliceConfig_Args = unsafe { std::mem::zeroed() };
        args.struct_size = std::mem::size_of::<PJRT_TpuTopology_GetSliceConfig_Args>();
        args.platform_type_name = platform_type_name.as_ptr() as *const i8;
        args.platform_type_name_len = platform_type_name.len();
        args.slice_name = slice_name.as_ptr() as *const i8;
        args.slice_name_len = slice_name.len();
        args.slice_config = &mut slice_config;

        let ext_fn = self.raw.get_slice_config.ok_or(Error::NullFunctionPointer(
            "PJRT_TpuTopology_GetSliceConfig",
        ))?;

        let err = unsafe { ext_fn(&mut args) };
        self.api.err_or(err, ())?;
        Ok(SliceConfig::from(&slice_config))
    }

    /// Get all slice configs for a given platform.
    ///
    /// # Arguments
    ///
    /// * `platform_type_name` - The platform type name
    /// * `max_configs` - Maximum number of configurations to return
    pub fn get_slice_configs(
        &self,
        platform_type_name: &str,
        max_configs: usize,
    ) -> Result<Vec<SliceConfig>> {
        let mut buf: Vec<PJRT_TpuTopology_SliceConfig> =
            vec![unsafe { std::mem::zeroed() }; max_configs];
        let mut args: PJRT_TpuTopology_GetSliceConfigs_Args = unsafe { std::mem::zeroed() };
        args.struct_size = std::mem::size_of::<PJRT_TpuTopology_GetSliceConfigs_Args>();
        args.platform_type_name = platform_type_name.as_ptr() as *const i8;
        args.platform_type_name_len = platform_type_name.len();
        args.slice_configs = buf.as_mut_ptr();
        args.max_slice_configs = max_configs;

        let ext_fn = self
            .raw
            .get_slice_configs
            .ok_or(Error::NullFunctionPointer(
                "PJRT_TpuTopology_GetSliceConfigs",
            ))?;

        let err = unsafe { ext_fn(&mut args) };
        self.api.err_or(err, ())?;

        buf.truncate(args.num_slice_configs);
        Ok(buf.iter().map(SliceConfig::from).collect())
    }

    /// Get the default platform config for a given platform.
    pub fn get_default_platform_config(
        &self,
        platform_type_name: &str,
    ) -> Result<DefaultPlatformConfig> {
        let mut args: PJRT_TpuTopology_GetDefaultPlatformConfig_Args =
            unsafe { std::mem::zeroed() };
        args.struct_size = std::mem::size_of::<PJRT_TpuTopology_GetDefaultPlatformConfig_Args>();
        args.platform_type_name = platform_type_name.as_ptr() as *const i8;
        args.platform_type_name_len = platform_type_name.len();

        let ext_fn = self
            .raw
            .get_default_platform_config
            .ok_or(Error::NullFunctionPointer(
                "PJRT_TpuTopology_GetDefaultPlatformConfig",
            ))?;

        let err = unsafe { ext_fn(&mut args) };
        self.api.err_or(err, ())?;

        Ok(DefaultPlatformConfig {
            num_chips_per_tray: args.num_chips_per_tray,
            num_trays: args.num_trays,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extension_type() {
        assert_eq!(
            TpuTopologyExtension::extension_type(),
            ExtensionType::TpuTopology
        );
    }

    #[test]
    fn test_from_raw_null_returns_none() {
        let api = unsafe { Api::empty_for_testing() };
        let result = unsafe { TpuTopologyExtension::from_raw(std::ptr::null_mut(), &api) };
        assert!(result.is_none());
    }

    #[test]
    fn test_from_raw_wrong_type_returns_none() {
        let api = unsafe { Api::empty_for_testing() };
        let mut ext: PJRT_TpuTopology_Extension = unsafe { std::mem::zeroed() };
        ext.base.struct_size = std::mem::size_of::<PJRT_TpuTopology_Extension>();
        ext.base.type_ = ExtensionType::Example.to_raw();
        let result = unsafe {
            TpuTopologyExtension::from_raw(
                &mut ext.base as *mut pjrt_sys::PJRT_Extension_Base,
                &api,
            )
        };
        assert!(result.is_none());
    }

    #[test]
    fn test_from_raw_correct_type() {
        let api = unsafe { Api::empty_for_testing() };
        let mut ext: PJRT_TpuTopology_Extension = unsafe { std::mem::zeroed() };
        ext.base.struct_size = std::mem::size_of::<PJRT_TpuTopology_Extension>();
        ext.base.type_ = ExtensionType::TpuTopology.to_raw();
        let result = unsafe {
            TpuTopologyExtension::from_raw(
                &mut ext.base as *mut pjrt_sys::PJRT_Extension_Base,
                &api,
            )
        };
        assert!(result.is_some());
    }

    #[test]
    fn test_null_fn_pointer_process_count() {
        let api = unsafe { Api::empty_for_testing() };
        let mut ext: PJRT_TpuTopology_Extension = unsafe { std::mem::zeroed() };
        ext.base.struct_size = std::mem::size_of::<PJRT_TpuTopology_Extension>();
        ext.base.type_ = ExtensionType::TpuTopology.to_raw();
        let wrapper = unsafe {
            TpuTopologyExtension::from_raw(
                &mut ext.base as *mut pjrt_sys::PJRT_Extension_Base,
                &api,
            )
        }
        .unwrap();
        // Create a fake topology - can't call method, but tests null fn check
        let topo = TopologyDescription {
            api: api.clone(),
            client: None,
            ptr: std::ptr::null_mut(),
        };
        let result = wrapper.process_count(&topo);
        assert!(result.is_err());
        // Prevent drop from calling destroy on null ptr
        std::mem::forget(topo);
    }

    #[test]
    fn test_slice_config_from() {
        let raw = PJRT_TpuTopology_SliceConfig {
            dim_size: 2,
            dimensions: [4, 4, 0, 0],
            wrap: [true, false, false, false],
            twist: true,
        };
        let config = SliceConfig::from(&raw);
        assert_eq!(config.dim_size, 2);
        assert_eq!(config.dimensions, [4, 4, 0, 0]);
        assert_eq!(config.wrap, [true, false, false, false]);
        assert!(config.twist);
    }

    #[test]
    fn test_debug_format() {
        let api = unsafe { Api::empty_for_testing() };
        let mut ext: PJRT_TpuTopology_Extension = unsafe { std::mem::zeroed() };
        ext.base.struct_size = std::mem::size_of::<PJRT_TpuTopology_Extension>();
        ext.base.type_ = ExtensionType::TpuTopology.to_raw();
        let wrapper = unsafe {
            TpuTopologyExtension::from_raw(
                &mut ext.base as *mut pjrt_sys::PJRT_Extension_Base,
                &api,
            )
        }
        .unwrap();
        let debug = format!("{:?}", wrapper);
        assert!(debug.contains("TpuTopologyExtension"));
        assert!(debug.contains("TpuTopology"));
    }
}
