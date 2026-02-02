//! PJRT TPU Topology Extension
//!
//! This module provides safe Rust bindings for the PJRT TPU Topology extension.
//! The TPU Topology extension provides access to TPU-specific topology information.
//!
//! ## Overview
//!
//! This extension is primarily used with TPU devices and provides capabilities for:
//!
//! - Querying TPU chip topology
//! - Understanding TPU core arrangement
//! - Accessing TPU-specific device layout information
//!
//! ## Usage
//!
//! ```rust,ignore
//! use pjrt::TpuTopologyExtension;
//!
//! // Get the TPU topology extension if available
//! if let Some(ext) = api.get_extension::<TpuTopologyExtension>() {
//!     println!("TPU topology extension is available");
//! }
//! ```
//!
//! ## Note
//!
//! This extension is only available in TPU PJRT plugins.

use crate::extension::{Extension, ExtensionType};
use crate::Api;

/// Safe wrapper for PJRT TPU Topology extension.
///
/// This extension provides access to TPU-specific topology information,
/// including chip arrangement and core layout.
///
/// ## Availability
///
/// This extension is only available in TPU PJRT plugins.
pub struct TpuTopologyExtension {
    raw: *mut pjrt_sys::PJRT_Extension_Base,
    _api: Api,
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

        Some(Self {
            raw: ptr,
            _api: api.clone(),
        })
    }
}

impl TpuTopologyExtension {
    /// Returns the raw extension pointer.
    ///
    /// # Safety
    ///
    /// The returned pointer is valid only for the lifetime of this extension.
    pub fn raw_ptr(&self) -> *mut pjrt_sys::PJRT_Extension_Base {
        self.raw
    }
}
