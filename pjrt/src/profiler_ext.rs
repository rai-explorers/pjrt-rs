//! PJRT Profiler Extension
//!
//! This module provides safe Rust bindings for the PJRT Profiler extension.
//! The Profiler extension provides access to profiler capabilities for
//! performance analysis.
//!
//! The profiler extension exposes a [`ProfilerApi`] function table which can be
//! used to create [`Profiler`] sessions. A profiler session follows the
//! lifecycle: **create → start → stop → collect_data → (drop)**.
//!
//! ## Usage
//!
//! ```rust,ignore
//! use pjrt::ProfilerExtension;
//!
//! // Get the profiler extension from the API
//! let profiler_ext = api.get_extension::<ProfilerExtension>()?;
//!
//! // Get the safe profiler API wrapper
//! if let Some(profiler_api) = profiler_ext.profiler_api() {
//!     // Create a profiler session with options
//!     let mut profiler = profiler_api.create("")?;
//!
//!     // Start profiling
//!     profiler.start()?;
//!
//!     // ... run workload ...
//!
//!     // Stop profiling
//!     profiler.stop()?;
//!
//!     // Collect the profiling data
//!     let data: Vec<u8> = profiler.collect_data()?;
//! }
//! // Profiler is automatically destroyed when dropped
//! ```

use std::rc::Rc;
use std::{ptr, slice};

use pjrt_sys::{
    PJRT_Profiler_Extension, PLUGIN_Profiler_Api, PLUGIN_Profiler_CollectData_Args,
    PLUGIN_Profiler_Create_Args, PLUGIN_Profiler_Destroy_Args, PLUGIN_Profiler_Error,
    PLUGIN_Profiler_Error_Destroy_Args, PLUGIN_Profiler_Error_GetCode_Args,
    PLUGIN_Profiler_Error_Message_Args, PLUGIN_Profiler_Start_Args, PLUGIN_Profiler_Stop_Args,
};

use crate::extension::{Extension, ExtensionType};
use crate::{Api, Error, Result};

/// Safe wrapper for PJRT Profiler extension.
///
/// The Profiler extension provides access to profiler capabilities for
/// performance analysis. Use [`profiler_api`](ProfilerExtension::profiler_api)
/// to obtain a [`ProfilerApi`] which can create profiler sessions.
///
/// Note that the profiler API can be unavailable (returns `None`) when the
/// extension is used as an args-only extension (e.g., for `traceme_context_id`).
pub struct ProfilerExtension {
    raw: Rc<PJRT_Profiler_Extension>,
    _api: Api,
}

impl std::fmt::Debug for ProfilerExtension {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ProfilerExtension")
            .field("api_version", &1i32) // PLUGIN_PROFILER_VERSION
            .field("has_profiler_api", &!self.raw.profiler_api.is_null())
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
            raw: Rc::new(*profiler_ext),
            _api: api.clone(),
        })
    }
}

impl ProfilerExtension {
    /// Get the safe profiler API wrapper if available.
    ///
    /// Returns `Some(ProfilerApi)` if the profiler API function table is present,
    /// `None` otherwise. The API can be absent when the extension is used purely
    /// as an args extension (providing only `traceme_context_id`).
    pub fn profiler_api(&self) -> Option<ProfilerApi> {
        if self.raw.profiler_api.is_null() {
            None
        } else {
            // SAFETY: We checked the pointer is non-null. The profiler_api pointer
            // is valid for the lifetime of the plugin which is held alive by _api.
            Some(ProfilerApi {
                raw: self.raw.profiler_api,
                _ext: Rc::clone(&self.raw),
            })
        }
    }

    /// Get the traceme context ID.
    ///
    /// This is valid only when the extension is used as an args extension
    /// (e.g., attached to compile or execute arguments).
    pub fn traceme_context_id(&self) -> i64 {
        self.raw.traceme_context_id
    }

    /// Check if the profiler API function table is available.
    pub fn has_profiler_api(&self) -> bool {
        !self.raw.profiler_api.is_null()
    }
}

// ---------------------------------------------------------------------------
// ProfilerApi — safe wrapper around PLUGIN_Profiler_Api function table
// ---------------------------------------------------------------------------

/// Safe wrapper around the `PLUGIN_Profiler_Api` function table.
///
/// Provides methods to create [`Profiler`] sessions for performance analysis.
/// Obtained via [`ProfilerExtension::profiler_api`].
#[derive(Debug)]
pub struct ProfilerApi {
    raw: *mut PLUGIN_Profiler_Api,
    /// Prevent the extension struct from being dropped while we hold the raw pointer.
    _ext: Rc<PJRT_Profiler_Extension>,
}

impl ProfilerApi {
    /// Create a new profiler session with the given options string.
    ///
    /// The `options` string is plugin-specific. Pass an empty string for default
    /// profiling behaviour.
    ///
    /// # Errors
    ///
    /// Returns an error if the plugin's `create` function fails or is unavailable.
    pub fn create(&self, options: &str) -> Result<Profiler<'_>> {
        let create_fn = self.raw_create()?;

        let mut args = PLUGIN_Profiler_Create_Args {
            struct_size: std::mem::size_of::<PLUGIN_Profiler_Create_Args>(),
            options: options.as_ptr().cast(),
            options_size: options.len(),
            profiler: ptr::null_mut(),
        };

        // SAFETY: args is stack-allocated with correct struct_size.
        // create_fn is a valid function pointer from the plugin.
        let err = unsafe { create_fn(&mut args) };
        self.check_error(err)?;

        if args.profiler.is_null() {
            return Err(Error::NullPointer);
        }

        Ok(Profiler {
            handle: args.profiler,
            api: self,
        })
    }

    // ---- internal helpers ----

    fn raw_api(&self) -> &PLUGIN_Profiler_Api {
        // SAFETY: self.raw was verified non-null when ProfilerApi was constructed.
        unsafe { &*self.raw }
    }

    fn raw_create(
        &self,
    ) -> Result<unsafe extern "C" fn(*mut PLUGIN_Profiler_Create_Args) -> *mut PLUGIN_Profiler_Error>
    {
        self.raw_api()
            .create
            .ok_or(Error::NullFunctionPointer("PLUGIN_Profiler_Create"))
    }

    fn raw_destroy(
        &self,
    ) -> Result<unsafe extern "C" fn(*mut PLUGIN_Profiler_Destroy_Args) -> *mut PLUGIN_Profiler_Error>
    {
        self.raw_api()
            .destroy
            .ok_or(Error::NullFunctionPointer("PLUGIN_Profiler_Destroy"))
    }

    fn raw_start(
        &self,
    ) -> Result<unsafe extern "C" fn(*mut PLUGIN_Profiler_Start_Args) -> *mut PLUGIN_Profiler_Error>
    {
        self.raw_api()
            .start
            .ok_or(Error::NullFunctionPointer("PLUGIN_Profiler_Start"))
    }

    fn raw_stop(
        &self,
    ) -> Result<unsafe extern "C" fn(*mut PLUGIN_Profiler_Stop_Args) -> *mut PLUGIN_Profiler_Error>
    {
        self.raw_api()
            .stop
            .ok_or(Error::NullFunctionPointer("PLUGIN_Profiler_Stop"))
    }

    fn raw_collect_data(
        &self,
    ) -> Result<
        unsafe extern "C" fn(*mut PLUGIN_Profiler_CollectData_Args) -> *mut PLUGIN_Profiler_Error,
    > {
        self.raw_api()
            .collect_data
            .ok_or(Error::NullFunctionPointer("PLUGIN_Profiler_CollectData"))
    }

    /// Convert a profiler error into a `crate::Error`, destroying the raw error.
    ///
    /// If `err` is null, returns `Ok(())`.
    fn check_error(&self, err: *mut PLUGIN_Profiler_Error) -> Result<()> {
        if err.is_null() {
            return Ok(());
        }

        let api = self.raw_api();

        // Get error message
        let message = if let Some(error_message_fn) = api.error_message {
            let mut msg_args = PLUGIN_Profiler_Error_Message_Args {
                struct_size: std::mem::size_of::<PLUGIN_Profiler_Error_Message_Args>(),
                priv_: ptr::null_mut(),
                error: err,
                message: ptr::null(),
                message_size: 0,
            };
            // SAFETY: err is non-null, msg_args is properly initialised.
            unsafe { error_message_fn(&mut msg_args) };

            if msg_args.message.is_null() || msg_args.message_size == 0 {
                String::from("(no message)")
            } else {
                // SAFETY: message pointer is valid for message_size bytes.
                let bytes = unsafe {
                    slice::from_raw_parts(msg_args.message as *const u8, msg_args.message_size)
                };
                String::from_utf8_lossy(bytes).into_owned()
            }
        } else {
            String::from("(error_message function unavailable)")
        };

        // Get error code
        let code = if let Some(error_get_code_fn) = api.error_get_code {
            let mut code_args = PLUGIN_Profiler_Error_GetCode_Args {
                struct_size: std::mem::size_of::<PLUGIN_Profiler_Error_GetCode_Args>(),
                priv_: ptr::null_mut(),
                error: err,
                code: 0,
            };
            // SAFETY: err is non-null, code_args is properly initialised.
            let inner_err = unsafe { error_get_code_fn(&mut code_args) };
            // Ignore secondary error from get_code
            if !inner_err.is_null() {
                self.destroy_error(inner_err);
            }
            code_args.code
        } else {
            -1
        };

        // Destroy the original error
        self.destroy_error(err);

        Err(Error::ProfilerError { message, code })
    }

    /// Destroy a profiler error handle.
    fn destroy_error(&self, err: *mut PLUGIN_Profiler_Error) {
        if err.is_null() {
            return;
        }
        if let Some(error_destroy_fn) = self.raw_api().error_destroy {
            let mut args = PLUGIN_Profiler_Error_Destroy_Args {
                struct_size: std::mem::size_of::<PLUGIN_Profiler_Error_Destroy_Args>(),
                priv_: ptr::null_mut(),
                error: err,
            };
            // SAFETY: err is non-null, args is properly initialised.
            unsafe { error_destroy_fn(&mut args) };
        }
    }
}

// ---------------------------------------------------------------------------
// Profiler — an active profiler session (RAII)
// ---------------------------------------------------------------------------

/// An active profiler session.
///
/// Created via [`ProfilerApi::create`]. The session follows the lifecycle:
/// **create → start → stop → collect_data → (drop)**.
///
/// The profiler handle is automatically destroyed when this struct is dropped.
///
/// # Example
///
/// ```rust,ignore
/// let mut profiler = profiler_api.create("")?;
/// profiler.start()?;
/// // ... run workload ...
/// profiler.stop()?;
/// let data = profiler.collect_data()?;
/// ```
pub struct Profiler<'a> {
    handle: *mut pjrt_sys::PLUGIN_Profiler,
    api: &'a ProfilerApi,
}

impl std::fmt::Debug for Profiler<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Profiler")
            .field("handle", &self.handle)
            .finish()
    }
}

impl<'a> Profiler<'a> {
    /// Start profiling.
    ///
    /// Must be called before [`stop`](Profiler::stop). Starting an already-started
    /// profiler is plugin-defined behaviour (typically an error).
    pub fn start(&mut self) -> Result<()> {
        let start_fn = self.api.raw_start()?;
        let mut args = PLUGIN_Profiler_Start_Args {
            struct_size: std::mem::size_of::<PLUGIN_Profiler_Start_Args>(),
            profiler: self.handle,
        };
        let err = unsafe { start_fn(&mut args) };
        self.api.check_error(err)
    }

    /// Stop profiling.
    ///
    /// Must be called after [`start`](Profiler::start) and before
    /// [`collect_data`](Profiler::collect_data).
    pub fn stop(&mut self) -> Result<()> {
        let stop_fn = self.api.raw_stop()?;
        let mut args = PLUGIN_Profiler_Stop_Args {
            struct_size: std::mem::size_of::<PLUGIN_Profiler_Stop_Args>(),
            profiler: self.handle,
        };
        let err = unsafe { stop_fn(&mut args) };
        self.api.check_error(err)
    }

    /// Collect profiling data.
    ///
    /// Should be called after [`stop`](Profiler::stop). Uses a two-pass protocol:
    /// 1. First call with a null buffer to determine the required buffer size.
    /// 2. Second call with an allocated buffer to retrieve the data.
    ///
    /// Returns the serialised profiling data as bytes.
    pub fn collect_data(&mut self) -> Result<Vec<u8>> {
        let collect_fn = self.api.raw_collect_data()?;

        // Pass 1: query required buffer size
        let mut args = PLUGIN_Profiler_CollectData_Args {
            struct_size: std::mem::size_of::<PLUGIN_Profiler_CollectData_Args>(),
            profiler: self.handle,
            buffer: ptr::null_mut(),
            buffer_size_in_bytes: 0,
        };
        let err = unsafe { collect_fn(&mut args) };
        self.api.check_error(err)?;

        if args.buffer_size_in_bytes == 0 {
            return Ok(Vec::new());
        }

        // Pass 2: allocate buffer and collect data
        let mut buffer = vec![0u8; args.buffer_size_in_bytes];
        args.buffer = buffer.as_mut_ptr();
        let err = unsafe { collect_fn(&mut args) };
        self.api.check_error(err)?;

        // Truncate if the plugin wrote fewer bytes than allocated
        buffer.truncate(args.buffer_size_in_bytes);
        Ok(buffer)
    }
}

impl Drop for Profiler<'_> {
    fn drop(&mut self) {
        if let Ok(destroy_fn) = self.api.raw_destroy() {
            let mut args = PLUGIN_Profiler_Destroy_Args {
                struct_size: std::mem::size_of::<PLUGIN_Profiler_Destroy_Args>(),
                profiler: self.handle,
            };
            // SAFETY: handle is valid — only set to non-null in ProfilerApi::create.
            let err = unsafe { destroy_fn(&mut args) };
            // Best-effort: destroy any error returned during cleanup
            if !err.is_null() {
                self.api.destroy_error(err);
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
#[allow(clippy::field_reassign_with_default)]
mod tests {
    use super::*;

    #[test]
    fn test_profiler_extension_debug_no_api() {
        let raw = PJRT_Profiler_Extension {
            base: pjrt_sys::PJRT_Extension_Base {
                struct_size: std::mem::size_of::<pjrt_sys::PJRT_Extension_Base>(),
                type_: ExtensionType::Profiler.to_raw(),
                next: ptr::null_mut(),
            },
            profiler_api: ptr::null_mut(),
            traceme_context_id: 42,
        };

        let ext = ProfilerExtension {
            raw: Rc::new(raw),
            _api: unsafe { Api::empty_for_testing() },
        };

        assert!(!ext.has_profiler_api());
        assert!(ext.profiler_api().is_none());
        assert_eq!(ext.traceme_context_id(), 42);

        let debug = format!("{:?}", ext);
        assert!(debug.contains("has_profiler_api: false"));
        assert!(debug.contains("traceme_context_id: 42"));
    }

    #[test]
    fn test_profiler_extension_debug_with_api() {
        let mut api_table = PLUGIN_Profiler_Api::default();
        api_table.struct_size = std::mem::size_of::<PLUGIN_Profiler_Api>();

        let raw = PJRT_Profiler_Extension {
            base: pjrt_sys::PJRT_Extension_Base {
                struct_size: std::mem::size_of::<pjrt_sys::PJRT_Extension_Base>(),
                type_: ExtensionType::Profiler.to_raw(),
                next: ptr::null_mut(),
            },
            profiler_api: &mut api_table as *mut _,
            traceme_context_id: 0,
        };

        let ext = ProfilerExtension {
            raw: Rc::new(raw),
            _api: unsafe { Api::empty_for_testing() },
        };

        assert!(ext.has_profiler_api());
        assert!(ext.profiler_api().is_some());

        let debug = format!("{:?}", ext);
        assert!(debug.contains("has_profiler_api: true"));
    }

    #[test]
    fn test_profiler_api_null_create_fn() {
        // API table with all None function pointers
        let mut api_table = PLUGIN_Profiler_Api::default();
        api_table.struct_size = std::mem::size_of::<PLUGIN_Profiler_Api>();

        let ext = unsafe { std::mem::zeroed::<PJRT_Profiler_Extension>() };
        let profiler_api = ProfilerApi {
            raw: &mut api_table as *mut _,
            _ext: Rc::new(ext),
        };

        let result = profiler_api.create("");
        assert!(result.is_err());
        match result.unwrap_err() {
            Error::NullFunctionPointer(name) => {
                assert_eq!(name, "PLUGIN_Profiler_Create");
            }
            other => panic!("expected NullFunctionPointer, got: {:?}", other),
        }
    }

    #[test]
    fn test_profiler_extension_type() {
        assert_eq!(ProfilerExtension::extension_type(), ExtensionType::Profiler);
    }

    #[test]
    fn test_profiler_extension_from_raw_null() {
        let api = unsafe { Api::empty_for_testing() };
        let result = unsafe { ProfilerExtension::from_raw(ptr::null_mut(), &api) };
        assert!(result.is_none());
    }

    #[test]
    fn test_profiler_api_check_error_null() {
        let mut api_table = PLUGIN_Profiler_Api::default();
        api_table.struct_size = std::mem::size_of::<PLUGIN_Profiler_Api>();

        let ext = unsafe { std::mem::zeroed::<PJRT_Profiler_Extension>() };
        let profiler_api = ProfilerApi {
            raw: &mut api_table as *mut _,
            _ext: Rc::new(ext),
        };

        // Null error should be Ok
        let result = profiler_api.check_error(ptr::null_mut());
        assert!(result.is_ok());
    }
}
