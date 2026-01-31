//! PJRT Profiler Extension
//!
//! This module provides safe Rust bindings for the PJRT Profiler extension.
//! The Profiler extension provides access to profiler capabilities for
//! performance analysis.
//!
//! ## Usage
//!
//! ```rust,ignore
//! use pjrt::profiler::ProfilerExtension;
//!
//! // Get the profiler extension
//! let profiler_ext = api.get_extension::<ProfilerExtension>()?;
//!
//! // Check if profiler API is available
//! if let Some(profiler_api) = profiler_ext.profiler_api() {
//!     // Use the profiler API
//! }
//! ```

use std::rc::Rc;

use pjrt_sys::PJRT_Profiler_Extension;

use crate::extension::{Extension, ExtensionType};
use crate::Api;

/// Safe wrapper for PJRT Profiler extension
///
/// The Profiler extension provides access to profiler capabilities for
/// performance analysis. Note that the profiler_api field can be nullptr
/// when used as an args extension.
pub struct ProfilerExtension {
    raw: Rc<PJRT_Profiler_Extension>,
    _api: Api,
}

impl std::fmt::Debug for ProfilerExtension {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ProfilerExtension")
            .field("api_version", &1i32) // Version 1
            .field(
                "has_profiler_api",
                &(self.raw.profiler_api != std::ptr::null_mut()),
            )
            .field("traceme_context_id", &self.raw.traceme_context_id)
            .finish()
    }
}

unsafe impl Extension for ProfilerExtension {
    fn extension_type() -> ExtensionType {
        ExtensionType::Profiler
    }

    unsafe fn from_raw(ptr: *mut pjrt_sys::PJRT_Extension_Base, api: &Api) -> Option<Self>
    where
        Self: Sized,
    {
        if ptr.is_null() {
            return None;
        }

        let profiler_ext = ptr as *mut PJRT_Profiler_Extension;
        if (*profiler_ext).base.type_ != ExtensionType::Profiler.to_raw() {
            return None;
        }

        Some(Self {
            raw: Rc::new((*profiler_ext).clone()),
            _api: api.clone(),
        })
    }
}

impl ProfilerExtension {
    /// Get the profiler API pointer if available
    ///
    /// Returns Some if the profiler API is available, None otherwise.
    /// Note: This can be None when the extension is used as an args extension.
    pub fn profiler_api(&self) -> Option<*mut pjrt_sys::PLUGIN_Profiler_Api> {
        if self.raw.profiler_api.is_null() {
            None
        } else {
            Some(self.raw.profiler_api)
        }
    }

    /// Get the traceme context ID
    ///
    /// Valid only when used as an args extension.
    pub fn traceme_context_id(&self) -> i64 {
        self.raw.traceme_context_id
    }

    /// Check if profiler API is available
    pub fn has_profiler_api(&self) -> bool {
        !self.raw.profiler_api.is_null()
    }
}
