//! PJRT Example Extension
//!
//! This module provides a reference implementation for PJRT extensions.
//! The Example extension serves as documentation for how extensions work
//! and can be used as a template for implementing new extensions.
//!
//! ## Overview
//!
//! PJRT uses an extension system to provide optional functionality beyond
//! the core API. Extensions are identified by their type and are accessed
//! through a linked list of extension bases attached to PJRT API responses.
//!
//! ## How Extensions Work
//!
//! 1. **Extension Base**: All extensions start with a `PJRT_Extension_Base`
//!    structure containing:
//!    - `struct_size`: Size of the extension structure
//!    - `type_`: The extension type (e.g., `PJRT_Extension_Type_Example`)
//!    - `next`: Pointer to the next extension in the chain
//!
//! 2. **Extension Discovery**: Plugins advertise available extensions through
//!    the `extension_start` field in API argument structures. Clients can
//!    iterate through the linked list to find extensions of interest.
//!
//! 3. **Type Safety**: The `Extension` trait in Rust provides type-safe
//!    access to extensions, ensuring the correct type is retrieved.
//!
//! ## Implementing a New Extension
//!
//! To implement a new extension, follow this pattern:
//!
//! ```rust,ignore
//! use pjrt::extension::{Extension, ExtensionType};
//! use pjrt::Api;
//!
//! // 1. Define the extension struct
//! pub struct MyExtension {
//!     raw: *mut pjrt_sys::PJRT_Extension_Base,
//!     api: Api,
//! }
//!
//! // 2. Implement Debug for the extension
//! impl std::fmt::Debug for MyExtension {
//!     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//!         f.debug_struct("MyExtension").finish()
//!     }
//! }
//!
//! // 3. Implement the Extension trait
//! unsafe impl Extension for MyExtension {
//!     fn extension_type() -> ExtensionType {
//!         ExtensionType::MyType // Add to ExtensionType enum first
//!     }
//!
//!     unsafe fn from_raw(ptr: *mut pjrt_sys::PJRT_Extension_Base, api: &Api) -> Option<Self> {
//!         if ptr.is_null() {
//!             return None;
//!         }
//!         if (*ptr).type_ != Self::extension_type().to_raw() {
//!             return None;
//!         }
//!         Some(Self { raw: ptr, api: api.clone() })
//!     }
//! }
//!
//! // 4. Add extension-specific methods
//! impl MyExtension {
//!     pub fn do_something(&self) -> Result<()> {
//!         // Implementation...
//!     }
//! }
//! ```
//!
//! ## Usage
//!
//! ```rust,ignore
//! use pjrt::ExampleExtension;
//!
//! // Get the example extension if available
//! if let Some(ext) = api.get_extension::<ExampleExtension>() {
//!     println!("Example extension is available");
//! }
//! ```
//!
//! ## Note
//!
//! The Example extension type is defined in the PJRT C API as a placeholder.
//! It is not typically implemented by any real plugin, but serves as
//! documentation for the extension system.

use std::rc::Rc;

use pjrt_sys::{
    PJRT_ExampleExtension_CreateExampleExtensionCpp_Args, PJRT_ExampleExtension_ExampleMethod_Args,
    PJRT_Example_Extension, PJRT_Extension_Type,
};

use crate::extension::{Extension, ExtensionType};
use crate::{Api, Error, Result};

/// Example extension type constant from PJRT.
///
/// This is the raw value of `PJRT_Extension_Type_Example` from the C API.
/// It is defined between `PJRT_Extension_Type_PhaseCompile` and
/// `PJRT_Extension_Type_Unknown` in the extension type enum.
#[allow(dead_code)]
const PJRT_EXTENSION_TYPE_EXAMPLE: PJRT_Extension_Type =
    pjrt_sys::PJRT_Extension_Type_PJRT_Extension_Type_Example;

/// Safe wrapper for PJRT Example extension.
///
/// This extension serves as a reference implementation and documentation
/// for how PJRT extensions work. It demonstrates:
///
/// - The extension trait implementation pattern
/// - Safe wrapping of raw extension pointers
/// - Extension discovery and type checking
///
/// ## API
///
/// - [`create`](Self::create) — Create a new example extension C++ object
/// - [`example_method`](Self::example_method) — Call the example method with a value
/// - [`destroy`](Self::destroy) — Destroy a previously created extension C++ object
///
/// ## When to Use
///
/// This extension type is primarily useful for:
/// - Learning how extensions work
/// - Testing extension discovery code
/// - Serving as a template for new extensions
pub struct ExampleExtension {
    raw: Rc<PJRT_Example_Extension>,
    api: Api,
}

impl std::fmt::Debug for ExampleExtension {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ExampleExtension")
            .field("type", &"Example")
            .field("type_id", &PJRT_EXTENSION_TYPE_EXAMPLE)
            .finish()
    }
}

unsafe impl Extension for ExampleExtension {
    fn extension_type() -> ExtensionType {
        ExtensionType::Example
    }

    unsafe fn from_raw(ptr: *mut pjrt_sys::PJRT_Extension_Base, api: &Api) -> Option<Self>
    where
        Self: Sized,
    {
        if ptr.is_null() {
            return None;
        }

        if (*ptr).type_ != Self::extension_type().to_raw() {
            return None;
        }

        let ext = ptr as *mut PJRT_Example_Extension;
        Some(Self {
            raw: Rc::new(*ext),
            api: api.clone(),
        })
    }
}

/// Handle to an example extension C++ object.
///
/// Created by [`ExampleExtension::create`] and destroyed by
/// [`ExampleExtension::destroy`].
pub struct ExampleExtensionCpp {
    ptr: *mut pjrt_sys::PJRT_ExampleExtensionCpp,
}

impl std::fmt::Debug for ExampleExtensionCpp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ExampleExtensionCpp")
            .field("ptr", &self.ptr)
            .finish()
    }
}

impl ExampleExtension {
    /// Returns the raw extension pointer.
    pub fn raw_ptr(&self) -> *mut pjrt_sys::PJRT_Extension_Base {
        &self.raw.base as *const pjrt_sys::PJRT_Extension_Base as *mut pjrt_sys::PJRT_Extension_Base
    }

    /// Returns the extension type ID as a raw value.
    pub fn type_id(&self) -> PJRT_Extension_Type {
        PJRT_EXTENSION_TYPE_EXAMPLE
    }

    /// Create a new example extension C++ object.
    ///
    /// This demonstrates the lifecycle management pattern for extension objects:
    /// the plugin allocates the object, the caller uses it, then calls `destroy`.
    pub fn create(&self) -> Result<ExampleExtensionCpp> {
        let mut args: PJRT_ExampleExtension_CreateExampleExtensionCpp_Args =
            unsafe { std::mem::zeroed() };

        let ext_fn = self
            .raw
            .create
            .ok_or(Error::NullFunctionPointer("PJRT_ExampleExtension_Create"))?;

        let err = unsafe { ext_fn(&mut args) };
        self.api.err_or(err, ())?;

        Ok(ExampleExtensionCpp {
            ptr: args.extension_cpp,
        })
    }

    /// Call the example method with a value.
    ///
    /// This demonstrates how extension methods operate on extension-managed objects.
    ///
    /// # Arguments
    ///
    /// * `cpp` - The extension C++ object created by [`create`](Self::create)
    /// * `value` - A value to pass to the example method
    pub fn example_method(&self, cpp: &mut ExampleExtensionCpp, value: i64) -> Result<()> {
        let mut args: PJRT_ExampleExtension_ExampleMethod_Args = unsafe { std::mem::zeroed() };
        args.extension_cpp = cpp.ptr;
        args.value = value;

        let ext_fn = self.raw.example_method.ok_or(Error::NullFunctionPointer(
            "PJRT_ExampleExtension_ExampleMethod",
        ))?;

        let err = unsafe { ext_fn(&mut args) };
        self.api.err_or(err, ())?;

        Ok(())
    }

    /// Destroy a previously created example extension C++ object.
    ///
    /// After calling this, the `cpp` handle is no longer valid.
    pub fn destroy(&self, cpp: ExampleExtensionCpp) -> Result<()> {
        let mut args: PJRT_ExampleExtension_CreateExampleExtensionCpp_Args =
            unsafe { std::mem::zeroed() };
        args.extension_cpp = cpp.ptr;

        let ext_fn = self
            .raw
            .destroy
            .ok_or(Error::NullFunctionPointer("PJRT_ExampleExtension_Destroy"))?;

        let err = unsafe { ext_fn(&mut args) };
        self.api.err_or(err, ())?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_example_extension_type_constant() {
        assert_eq!(
            PJRT_EXTENSION_TYPE_EXAMPLE,
            pjrt_sys::PJRT_Extension_Type_PJRT_Extension_Type_Example
        );
    }

    #[test]
    fn test_example_extension_type_in_enum() {
        let ext_type = ExtensionType::Example;
        assert_eq!(
            ext_type.to_raw(),
            pjrt_sys::PJRT_Extension_Type_PJRT_Extension_Type_Example
        );
    }

    #[test]
    fn test_example_extension_trait_type() {
        assert_eq!(ExampleExtension::extension_type(), ExtensionType::Example);
    }

    #[test]
    fn test_from_raw_null_returns_none() {
        let api = unsafe { Api::empty_for_testing() };
        let result = unsafe { ExampleExtension::from_raw(std::ptr::null_mut(), &api) };
        assert!(result.is_none());
    }

    #[test]
    fn test_from_raw_wrong_type_returns_none() {
        let api = unsafe { Api::empty_for_testing() };
        let mut ext: PJRT_Example_Extension = unsafe { std::mem::zeroed() };
        ext.base.struct_size = std::mem::size_of::<PJRT_Example_Extension>();
        ext.base.type_ = ExtensionType::Stream.to_raw();
        let result = unsafe {
            ExampleExtension::from_raw(&mut ext.base as *mut pjrt_sys::PJRT_Extension_Base, &api)
        };
        assert!(result.is_none());
    }

    #[test]
    fn test_from_raw_correct_type() {
        let api = unsafe { Api::empty_for_testing() };
        let mut ext: PJRT_Example_Extension = unsafe { std::mem::zeroed() };
        ext.base.struct_size = std::mem::size_of::<PJRT_Example_Extension>();
        ext.base.type_ = ExtensionType::Example.to_raw();
        let result = unsafe {
            ExampleExtension::from_raw(&mut ext.base as *mut pjrt_sys::PJRT_Extension_Base, &api)
        };
        assert!(result.is_some());
    }

    #[test]
    fn test_type_id() {
        let api = unsafe { Api::empty_for_testing() };
        let mut ext: PJRT_Example_Extension = unsafe { std::mem::zeroed() };
        ext.base.struct_size = std::mem::size_of::<PJRT_Example_Extension>();
        ext.base.type_ = ExtensionType::Example.to_raw();
        let wrapper = unsafe {
            ExampleExtension::from_raw(&mut ext.base as *mut pjrt_sys::PJRT_Extension_Base, &api)
        }
        .unwrap();
        assert_eq!(wrapper.type_id(), PJRT_EXTENSION_TYPE_EXAMPLE);
    }

    #[test]
    fn test_debug_format() {
        let api = unsafe { Api::empty_for_testing() };
        let mut ext: PJRT_Example_Extension = unsafe { std::mem::zeroed() };
        ext.base.struct_size = std::mem::size_of::<PJRT_Example_Extension>();
        ext.base.type_ = ExtensionType::Example.to_raw();
        let wrapper = unsafe {
            ExampleExtension::from_raw(&mut ext.base as *mut pjrt_sys::PJRT_Extension_Base, &api)
        }
        .unwrap();
        let debug = format!("{:?}", wrapper);
        assert!(debug.contains("ExampleExtension"));
        assert!(debug.contains("Example"));
    }

    #[test]
    fn test_create_null_fn_pointer() {
        let api = unsafe { Api::empty_for_testing() };
        let mut ext: PJRT_Example_Extension = unsafe { std::mem::zeroed() };
        ext.base.struct_size = std::mem::size_of::<PJRT_Example_Extension>();
        ext.base.type_ = ExtensionType::Example.to_raw();
        let wrapper = unsafe {
            ExampleExtension::from_raw(&mut ext.base as *mut pjrt_sys::PJRT_Extension_Base, &api)
        }
        .unwrap();
        let result = wrapper.create();
        assert!(result.is_err());
    }
}
