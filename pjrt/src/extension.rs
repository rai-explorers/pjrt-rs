//! PJRT Extension Framework
//!
//! This module provides the infrastructure for working with PJRT extensions.
//! Extensions allow plugins to provide additional functionality beyond the core API.
//!
//! The extension system uses a linked list structure where each extension has a type
//! and a pointer to the next extension. Extensions can be accessed through the
//! `extension_start` field in various PJRT API argument structures.
//!
//! ## Extension Types
//!
//! - Profiler: Performance profiling support
//! - Stream: GPU stream management
//! - Layouts: Advanced memory layouts
//! - FFI: Foreign Function Interface for custom operations
//! - MemoryDescriptions: Advanced memory management
//! - CrossHostTransfers: Multi-host distributed training
//! - Callback: Custom callback registration
//! - TpuTopology: TPU-specific topology
//! - TpuExecutable: TPU-specific executable features
//! - And more...
//!
//! ## Usage
//!
//! Extensions are typically accessed through the `Api` or `Client` instances:
//!
//! ```rust,ignore
//! // Check if a profiler extension is available
//! if let Some(profiler) = client.extension::<ProfilerExtension>() {
//!     profiler.start();
//!     // ... run workload ...
//!     let data = profiler.stop();
//! }
//! ```

use pjrt_sys::{
    PJRT_Extension_Base, PJRT_Extension_Type, PJRT_Extension_Type_PJRT_Extension_Type_Callback,
    PJRT_Extension_Type_PJRT_Extension_Type_CrossHostTransfers,
    PJRT_Extension_Type_PJRT_Extension_Type_Custom_Partitioner,
    PJRT_Extension_Type_PJRT_Extension_Type_Example,
    PJRT_Extension_Type_PJRT_Extension_Type_ExecutableMetadata,
    PJRT_Extension_Type_PJRT_Extension_Type_FFI,
    PJRT_Extension_Type_PJRT_Extension_Type_Gpu_Custom_Call,
    PJRT_Extension_Type_PJRT_Extension_Type_HostAllocator,
    PJRT_Extension_Type_PJRT_Extension_Type_Layouts,
    PJRT_Extension_Type_PJRT_Extension_Type_Megascale,
    PJRT_Extension_Type_PJRT_Extension_Type_MemoryDescriptions,
    PJRT_Extension_Type_PJRT_Extension_Type_PhaseCompile,
    PJRT_Extension_Type_PJRT_Extension_Type_Profiler,
    PJRT_Extension_Type_PJRT_Extension_Type_RawBuffer,
    PJRT_Extension_Type_PJRT_Extension_Type_Stream,
    PJRT_Extension_Type_PJRT_Extension_Type_TpuExecutable,
    PJRT_Extension_Type_PJRT_Extension_Type_TpuTopology,
    PJRT_Extension_Type_PJRT_Extension_Type_Triton,
};

use crate::Api;

/// Types of PJRT extensions available
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExtensionType {
    /// GPU custom call extension
    GpuCustomCall,
    /// Profiler extension for performance analysis
    Profiler,
    /// Custom partitioner extension
    CustomPartitioner,
    /// Stream extension for GPU stream management
    Stream,
    /// Layouts extension for advanced memory layouts
    Layouts,
    /// FFI extension for custom operations
    Ffi,
    /// Memory descriptions extension
    MemoryDescriptions,
    /// Triton extension for Triton kernel support
    Triton,
    /// Raw buffer extension (experimental)
    RawBuffer,
    /// Cross-host transfers extension
    CrossHostTransfers,
    /// Executable metadata extension
    ExecutableMetadata,
    /// Callback extension
    Callback,
    /// Host allocator extension (experimental)
    HostAllocator,
    /// TPU topology extension
    TpuTopology,
    /// TPU executable extension
    TpuExecutable,
    /// Megascale extension for large-scale training
    Megascale,
    /// Phase compile extension for debugging and caching
    PhaseCompile,
    /// Example extension for documentation and testing
    ///
    /// This extension type serves as a reference implementation and is
    /// typically not implemented by production plugins. It's useful for:
    /// - Learning how extensions work
    /// - Testing extension discovery code
    /// - Serving as a template for new extensions
    Example,
}

impl ExtensionType {
    /// Convert to the raw PJRT extension type
    pub fn to_raw(self) -> PJRT_Extension_Type {
        match self {
            ExtensionType::GpuCustomCall => PJRT_Extension_Type_PJRT_Extension_Type_Gpu_Custom_Call,
            ExtensionType::Profiler => PJRT_Extension_Type_PJRT_Extension_Type_Profiler,
            ExtensionType::CustomPartitioner => {
                PJRT_Extension_Type_PJRT_Extension_Type_Custom_Partitioner
            }
            ExtensionType::Stream => PJRT_Extension_Type_PJRT_Extension_Type_Stream,
            ExtensionType::Layouts => PJRT_Extension_Type_PJRT_Extension_Type_Layouts,
            ExtensionType::Ffi => PJRT_Extension_Type_PJRT_Extension_Type_FFI,
            ExtensionType::MemoryDescriptions => {
                PJRT_Extension_Type_PJRT_Extension_Type_MemoryDescriptions
            }
            ExtensionType::Triton => PJRT_Extension_Type_PJRT_Extension_Type_Triton,
            ExtensionType::RawBuffer => PJRT_Extension_Type_PJRT_Extension_Type_RawBuffer,
            ExtensionType::CrossHostTransfers => {
                PJRT_Extension_Type_PJRT_Extension_Type_CrossHostTransfers
            }
            ExtensionType::ExecutableMetadata => {
                PJRT_Extension_Type_PJRT_Extension_Type_ExecutableMetadata
            }
            ExtensionType::Callback => PJRT_Extension_Type_PJRT_Extension_Type_Callback,
            ExtensionType::HostAllocator => PJRT_Extension_Type_PJRT_Extension_Type_HostAllocator,
            ExtensionType::TpuTopology => PJRT_Extension_Type_PJRT_Extension_Type_TpuTopology,
            ExtensionType::TpuExecutable => PJRT_Extension_Type_PJRT_Extension_Type_TpuExecutable,
            ExtensionType::Megascale => PJRT_Extension_Type_PJRT_Extension_Type_Megascale,
            ExtensionType::PhaseCompile => PJRT_Extension_Type_PJRT_Extension_Type_PhaseCompile,
            ExtensionType::Example => PJRT_Extension_Type_PJRT_Extension_Type_Example,
        }
    }
}

/// Trait for PJRT extensions
///
/// This trait defines the interface that all PJRT extensions must implement.
/// Extensions provide additional functionality beyond the core PJRT API.
///
/// # Safety
///
/// Implementors must ensure that the extension is properly initialized and
/// that all raw pointer operations are safe.
pub unsafe trait Extension {
    /// The type of this extension
    fn extension_type() -> ExtensionType;

    /// Create an extension wrapper from a raw extension base pointer
    ///
    /// # Safety
    ///
    /// The `ptr` must be a valid pointer to a PJRT_Extension_Base structure
    /// with the correct extension type.
    unsafe fn from_raw(ptr: *mut PJRT_Extension_Base, api: &Api) -> Option<Self>
    where
        Self: Sized;
}

/// Iterator over extension chain
#[allow(dead_code)]
pub struct ExtensionIterator {
    current: *mut PJRT_Extension_Base,
}

impl ExtensionIterator {
    /// Create a new extension iterator starting from the given extension base
    ///
    /// # Safety
    ///
    /// The `start` pointer must be a valid pointer to a PJRT_Extension_Base
    /// structure, or null.
    pub(crate) unsafe fn new(start: *mut PJRT_Extension_Base) -> Self {
        Self { current: start }
    }
}

impl Iterator for ExtensionIterator {
    type Item = *mut PJRT_Extension_Base;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current.is_null() {
            None
        } else {
            let current = self.current;
            unsafe {
                self.current = (*current).next;
            }
            Some(current)
        }
    }
}

/// Find an extension of a specific type in an extension chain
///
/// # Safety
///
/// The `start` pointer must be a valid pointer to a PJRT_Extension_Base
/// structure, or null.
#[allow(dead_code)]
pub(crate) unsafe fn find_extension(
    start: *mut PJRT_Extension_Base,
    ext_type: ExtensionType,
) -> Option<*mut PJRT_Extension_Base> {
    let iter = ExtensionIterator::new(start);
    let raw_type = ext_type.to_raw();

    for ext in iter {
        unsafe {
            if (*ext).type_ == raw_type {
                return Some(ext);
            }
        }
    }
    None
}

/// Helper function to check if an extension is available
///
/// This can be used by `Api` and `Client` to check for extension availability.
///
/// # Safety
///
/// The `start` pointer must be a valid pointer to a PJRT_Extension_Base
/// structure, or null.
#[allow(dead_code)]
pub(crate) unsafe fn has_extension(
    start: *mut PJRT_Extension_Base,
    ext_type: ExtensionType,
) -> bool {
    find_extension(start, ext_type).is_some()
}

// Note: Real extension implementations are in separate modules:
// - stream_ext.rs: StreamExtension implementation
// - layouts_ext.rs: LayoutsExtension implementation
// - ffi_ext.rs: FfiExtension implementation
// - raw_buffer_ext.rs: RawBufferExtension implementation
// - gpu_ext.rs: GpuExtension implementation
// - profiler_ext.rs: ProfilerExtension implementation
