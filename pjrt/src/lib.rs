#![deny(unused_must_use)]

mod utils;

mod error;
pub use error::{Error, Result};

mod ty;
pub use ty::*;

mod plugin;
pub use plugin::plugin;

mod api;
pub use api::Api;

mod client;
pub use client::Client;

mod buffer;
pub use buffer::Buffer;

mod host_buffer;
pub use host_buffer::{HostBuffer, TypedHostBuffer};

mod memory_layout;
pub use memory_layout::MemoryLayout;

mod compile;
pub use compile::{CompileOptions, CompileToExecutable, CompileToLoadedExecutable};

mod device;
pub use device::{Device, GlobalDeviceId, LocalDeviceId, LocalHardwareId, MemoryStats};

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
pub use executable::{CompiledMemoryStats, Executable};

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
// re-export pjrt-sys
pub use pjrt_sys::protos;
