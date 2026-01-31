#![deny(unused_must_use)]

//! # PJRT-RS: Rust Bindings for PJRT
//!
//! This crate provides safe, idiomatic Rust bindings to the [PJRT](https://github.com/openxla/pjrt)
//! (Plugin-based JAX Runtime) C API. PJRT is Google's abstraction layer that simplifies
//! machine learning hardware and framework integration.
//!
//! ## Architecture
//!
//! This crate consists of two main components:
//!
//! 1. **High-level safe API** (this crate): Idiomatic Rust interfaces wrapped around the C API
//! 2. **Low-level bindings** (`pjrt-sys` crate): Direct unsafe bindings to the PJRT C API
//!
//! ## Core Concepts
//!
//! - **Api**: Entry point for loading PJRT plugins and creating clients
//! - **Client**: Represents a PJRT runtime instance for a specific plugin
//! - **Device**: Hardware accelerator (CPU, GPU, TPU) available for computation
//! - **Buffer**: Data storage on host or device memory
//! - **Program**: Compiled computation (MLIR/HLO) that can be executed
//! - **LoadedExecutable**: A compiled program ready for execution on specific devices
//!
//! ## Features
//!
//! - Memory-safe Rust bindings with proper error handling
//! - Support for all major PJRT data types (f16, bf16, f32, f64, complex types, integers)
//! - Async operations for non-blocking execution
//! - Device memory management with automatic cleanup
//! - Comprehensive error reporting with detailed error codes
//!
//! ## Platform Support
//!
//! The crate supports all platforms where PJRT is available:
//! - Linux x86_64 (CPU, GPU, TPU)
//! - macOS (CPU only)
//! - Windows (CPU, GPU)
//!
//! For more detailed examples and advanced usage patterns, see the `examples/` directory.

mod utils;

mod error;
pub use error::{Error, ErrorCode, Result};

mod ty;
pub use ty::*;

mod plugin;
pub use plugin::plugin;

mod api;
pub use api::Api;

mod client;
pub use client::{Client, FulfillAliasBufferCallback, ProcessInfo, ProcessState};

mod buffer;
pub use buffer::{Buffer, CopyRawToHostFuture, DonateWithControlDependency};

mod host_buffer;
pub use host_buffer::{HostBuffer, TypedHostBuffer};

mod memory_layout;
pub use memory_layout::MemoryLayout;

mod compile;
pub use compile::{CompileOptions, CompileToExecutable, CompileToLoadedExecutable};

mod device;
pub use device::{
    AsyncTrackingEvent, Device, GlobalDeviceId, LocalDeviceId, LocalHardwareId, MemoryStats,
};

mod device_description;
pub use device_description::DeviceDescription;

mod device_assignment;
pub use device_assignment::{DeviceAssignment, LogicalId};

mod memory;
pub use memory::Memory;

mod topology_description;
pub use topology_description::TopologyDescription;

mod program;
pub use program::{Program, ProgramFormat};

mod loaded_executable;
pub use loaded_executable::LoadedExecutable;

mod executable;
pub use executable::{CompiledMemoryStats, Executable, SerializedCompileOptions};

mod event;
pub use event::Event;

mod named_value;
pub use named_value::{NamedValue, NamedValueMap};

mod execute;
pub use execute::{ExecuteContext, ExecuteOptions, Execution, ExecutionInputs};

mod device_stream;
pub use device_stream::CopyToDeviceStream;

mod chunk;
pub use chunk::Chunk;

mod kv_store;
pub use kv_store::KeyValueStore;

mod extension;
pub use extension::{Extension, ExtensionType};

mod stream_ext;
pub use stream_ext::{DeviceStream, StreamExt, StreamExtension};

mod layouts_ext;
pub use layouts_ext::{LayoutsExtension, LayoutsMemoryLayout, SerializedLayout};

mod ffi_ext;
pub use ffi_ext::{FfiExtension, FfiHandler, FfiHandlerTraits, FfiTypeInfo};

mod raw_buffer_ext;
pub use raw_buffer_ext::{RawBuffer, RawBufferExtension};

mod gpu_ext;
pub use gpu_ext::{CustomCallApiVersion, CustomCallHandler, GpuExtension};

mod custom_partitioner_ext;
pub use custom_partitioner_ext::CustomPartitionerExtension;

mod triton_ext;
pub use triton_ext::{TritonCompileResult, TritonExtension};

mod profiler_ext;
pub use profiler_ext::ProfilerExtension;

mod callback_ext;
pub use callback_ext::CallbackExtension;

mod memory_descriptions_ext;
pub use memory_descriptions_ext::{
    DeviceMemoryDescriptions, MemoryDescription, MemoryDescriptionsExtension, MemoryKind,
};

mod phase_compile_ext;
pub use phase_compile_ext::{PhaseCompileExtension, PhaseCompileOutput, PhaseCompiler};

mod async_transfer;
pub use async_transfer::{AsyncHostToDeviceTransferManager, BufferShape};
// re-export pjrt-sys
pub use pjrt_sys::protos;
