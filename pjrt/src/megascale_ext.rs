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
//! - Coordinating training across many devices/hosts
//! - Optimized collective operations at scale
//! - Large-scale synchronization primitives
//!
//! ## Usage
//!
//! ```rust,ignore
//! use pjrt::MegascaleExtension;
//!
//! // Get the megascale extension if available
//! if let Some(ext) = api.get_extension::<MegascaleExtension>() {
//!     println!("Megascale extension is available");
//! }
//! ```
//!
//! ## Note
//!
//! This extension is primarily available in PJRT plugins designed for
//! large-scale distributed training, such as TPU pods.

use crate::extension::{Extension, ExtensionType};
use crate::Api;

/// Safe wrapper for PJRT Megascale extension.
///
/// This extension provides capabilities for large-scale distributed training,
/// including optimized collective operations and synchronization primitives.
///
/// ## Availability
///
/// This extension is typically only available in PJRT plugins designed for
/// large-scale distributed execution, such as TPU pod plugins.
pub struct MegascaleExtension {
    raw: *mut pjrt_sys::PJRT_Extension_Base,
    _api: Api,
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

        Some(Self {
            raw: ptr,
            _api: api.clone(),
        })
    }
}

impl MegascaleExtension {
    /// Returns the raw extension pointer.
    ///
    /// # Safety
    ///
    /// The returned pointer is valid only for the lifetime of this extension.
    pub fn raw_ptr(&self) -> *mut pjrt_sys::PJRT_Extension_Base {
        self.raw
    }
}
