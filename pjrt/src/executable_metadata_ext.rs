//! PJRT Executable Metadata Extension
//!
//! This module provides safe Rust bindings for the PJRT Executable Metadata extension.
//! The Executable Metadata extension allows access to additional metadata about
//! compiled executables.
//!
//! ## Overview
//!
//! This extension provides capabilities for:
//!
//! - Querying metadata about compiled executables
//! - Accessing compilation statistics
//! - Retrieving additional executable properties
//!
//! ## Usage
//!
//! ```rust,ignore
//! use pjrt::ExecutableMetadataExtension;
//!
//! // Get the executable metadata extension if available
//! if let Some(ext) = api.get_extension::<ExecutableMetadataExtension>() {
//!     println!("Executable metadata extension is available");
//! }
//! ```
//!
//! ## Note
//!
//! The specific metadata available depends on the PJRT plugin implementation.

use crate::extension::{Extension, ExtensionType};
use crate::Api;

/// Safe wrapper for PJRT Executable Metadata extension.
///
/// This extension provides access to additional metadata about compiled executables,
/// such as compilation statistics and executable properties.
///
/// ## Availability
///
/// This extension may not be available in all PJRT plugins. Check for availability
/// before use.
pub struct ExecutableMetadataExtension {
    raw: *mut pjrt_sys::PJRT_Extension_Base,
    _api: Api,
}

impl std::fmt::Debug for ExecutableMetadataExtension {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ExecutableMetadataExtension")
            .field("type", &"ExecutableMetadata")
            .finish()
    }
}

unsafe impl Extension for ExecutableMetadataExtension {
    fn extension_type() -> ExtensionType {
        ExtensionType::ExecutableMetadata
    }

    unsafe fn from_raw(ptr: *mut pjrt_sys::PJRT_Extension_Base, api: &Api) -> Option<Self>
    where
        Self: Sized,
    {
        if ptr.is_null() {
            return None;
        }

        if (*ptr).type_ != ExtensionType::ExecutableMetadata.to_raw() {
            return None;
        }

        Some(Self {
            raw: ptr,
            _api: api.clone(),
        })
    }
}

impl ExecutableMetadataExtension {
    /// Returns the raw extension pointer.
    ///
    /// # Safety
    ///
    /// The returned pointer is valid only for the lifetime of this extension.
    pub fn raw_ptr(&self) -> *mut pjrt_sys::PJRT_Extension_Base {
        self.raw
    }
}
