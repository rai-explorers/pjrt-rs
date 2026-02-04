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

use pjrt_sys::PJRT_Extension_Type;

use crate::extension::{Extension, ExtensionType};
use crate::Api;

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
/// ## Extension Architecture
///
/// ```text
/// PJRT_Api_Args
/// └── extension_start: *mut PJRT_Extension_Base
///     ├── type_: PJRT_Extension_Type_Stream
///     ├── next ─────────────────────────────────────┐
///     │                                             │
///     │   PJRT_Extension_Base                       │
///     └── type_: PJRT_Extension_Type_Layouts  <─────┘
///         ├── next ─────────────────────────────────┐
///         │                                         │
///         │   PJRT_Extension_Base                   │
///         └── type_: PJRT_Extension_Type_Example ◄──┘
///             └── next: nullptr (end of chain)
/// ```
///
/// ## When to Use
///
/// This extension type is primarily useful for:
/// - Learning how extensions work
/// - Testing extension discovery code
/// - Serving as a template for new extensions
pub struct ExampleExtension {
    raw: *mut pjrt_sys::PJRT_Extension_Base,
    _api: Api,
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

        // Check if this is an Example extension
        if (*ptr).type_ != Self::extension_type().to_raw() {
            return None;
        }

        Some(Self {
            raw: ptr,
            _api: api.clone(),
        })
    }
}

impl ExampleExtension {
    /// Returns the raw extension pointer.
    ///
    /// This can be used for interop with other C APIs or for advanced use cases.
    /// The returned pointer is valid only for the lifetime of this extension.
    pub fn raw_ptr(&self) -> *mut pjrt_sys::PJRT_Extension_Base {
        self.raw
    }

    /// Returns the extension type ID as a raw u32 value.
    ///
    /// This is primarily useful for debugging and logging.
    pub fn type_id(&self) -> u32 {
        PJRT_EXTENSION_TYPE_EXAMPLE
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_example_extension_type_constant() {
        // Verify the Example extension type constant is defined
        assert_eq!(
            PJRT_EXTENSION_TYPE_EXAMPLE,
            pjrt_sys::PJRT_Extension_Type_PJRT_Extension_Type_Example as u32
        );
    }

    #[test]
    fn test_example_extension_type_in_enum() {
        // Verify the ExtensionType enum includes Example
        let ext_type = ExtensionType::Example;
        assert_eq!(
            ext_type.to_raw(),
            pjrt_sys::PJRT_Extension_Type_PJRT_Extension_Type_Example
        );
    }

    #[test]
    fn test_example_extension_trait_type() {
        // Verify the Extension trait implementation returns the correct type
        assert_eq!(ExampleExtension::extension_type(), ExtensionType::Example);
    }
}
