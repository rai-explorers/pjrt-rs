//! PJRT TPU Executable Extension
//!
//! This module provides safe Rust bindings for the PJRT TPU Executable extension.
//! The TPU Executable extension provides access to TPU-specific executable features.
//!
//! ## Overview
//!
//! This extension is primarily used with TPU devices and provides capabilities for:
//!
//! - Accessing TPU-specific executable properties
//! - TPU-optimized execution features
//! - TPU compilation metadata
//!
//! ## Usage
//!
//! ```rust,ignore
//! use pjrt::TpuExecutableExtension;
//!
//! // Get the TPU executable extension if available
//! if let Some(ext) = api.get_extension::<TpuExecutableExtension>() {
//!     println!("TPU executable extension is available");
//! }
//! ```
//!
//! ## Note
//!
//! This extension is only available in TPU PJRT plugins.

use crate::extension::{Extension, ExtensionType};
use crate::Api;

/// Safe wrapper for PJRT TPU Executable extension.
///
/// This extension provides access to TPU-specific executable features,
/// including TPU-optimized execution and compilation metadata.
///
/// ## Availability
///
/// This extension is only available in TPU PJRT plugins.
pub struct TpuExecutableExtension {
    raw: *mut pjrt_sys::PJRT_Extension_Base,
    _api: Api,
}

impl std::fmt::Debug for TpuExecutableExtension {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TpuExecutableExtension")
            .field("type", &"TpuExecutable")
            .finish()
    }
}

unsafe impl Extension for TpuExecutableExtension {
    fn extension_type() -> ExtensionType {
        ExtensionType::TpuExecutable
    }

    unsafe fn from_raw(ptr: *mut pjrt_sys::PJRT_Extension_Base, api: &Api) -> Option<Self>
    where
        Self: Sized,
    {
        if ptr.is_null() {
            return None;
        }

        if (*ptr).type_ != ExtensionType::TpuExecutable.to_raw() {
            return None;
        }

        Some(Self {
            raw: ptr,
            _api: api.clone(),
        })
    }
}

impl TpuExecutableExtension {
    /// Returns the raw extension pointer.
    ///
    /// # Safety
    ///
    /// The returned pointer is valid only for the lifetime of this extension.
    pub fn raw_ptr(&self) -> *mut pjrt_sys::PJRT_Extension_Base {
        self.raw
    }
}
