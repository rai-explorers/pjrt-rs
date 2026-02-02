# PJRT-RS API Gap Analysis and TODOs

This document tracks the gaps between the PJRT C API and the Rust pjrt-rs safe API wrapper.

## Summary

The pjrt-rs crate provides comprehensive bindings to the PJRT C API. Most core functionality is wrapped, but there are some gaps and improvement opportunities.

## API Coverage Status

### ✅ Fully Implemented

| Category | Functions |
|----------|-----------|
| **Error Handling** | `Error_Destroy`, `Error_Message`, `Error_GetCode` |
| **Plugin** | `Plugin_Initialize`, `Plugin_Attributes` |
| **Events** | `Event_Destroy`, `Event_IsReady`, `Event_Error`, `Event_Await`, `Event_OnReady`, `Event_Create`, `Event_Set` |
| **Client** | `Client_Create`, `Client_Destroy`, `Client_PlatformName`, `Client_ProcessIndex`, `Client_PlatformVersion`, `Client_Devices`, `Client_AddressableDevices`, `Client_LookupDevice`, `Client_LookupAddressableDevice`, `Client_AddressableMemories`, `Client_Compile`, `Client_DefaultDeviceAssignment`, `Client_BufferFromHostBuffer`, `Client_TopologyDescription`, `Client_CreateViewOfDeviceBuffer`, `Client_CreateBuffersForAsyncHostToDevice`, `Client_CreateUninitializedBuffer`, `Client_CreateErrorBuffer`, `Client_CreateAliasBuffer`, `Client_FulfillAliasBuffer`, `Client_UpdateGlobalProcessInfo`, `Client_DmaMap`, `Client_DmaUnmap` |
| **Device Description** | `DeviceDescription_Id`, `DeviceDescription_ProcessIndex`, `DeviceDescription_Attributes`, `DeviceDescription_Kind`, `DeviceDescription_DebugString`, `DeviceDescription_ToString` |
| **Device** | `Device_GetDescription`, `Device_IsAddressable`, `Device_LocalHardwareId`, `Device_AddressableMemories`, `Device_DefaultMemory`, `Device_MemoryStats`, `Device_CreateAsyncTrackingEvent`, `Device_PoisonExecution` |
| **Memory** | `Memory_Id`, `Memory_Kind`, `Memory_Kind_Id`, `Memory_DebugString`, `Memory_ToString`, `Memory_AddressableByDevices` |
| **Executable** | `Executable_Destroy`, `Executable_Name`, `Executable_NumReplicas`, `Executable_NumPartitions`, `Executable_NumOutputs`, `Executable_SizeOfGeneratedCodeInBytes`, `Executable_GetCostAnalysis`, `Executable_OutputMemoryKinds`, `Executable_OptimizedProgram`, `Executable_Serialize`, `Executable_OutputElementTypes`, `Executable_OutputDimensions`, `Executable_Fingerprint`, `Executable_GetCompiledMemoryStats`, `Executable_GetCompileOptions`, `Executable_DeserializeAndLoad` |
| **LoadedExecutable** | `LoadedExecutable_Destroy`, `LoadedExecutable_GetExecutable`, `LoadedExecutable_AddressableDevices`, `LoadedExecutable_Delete`, `LoadedExecutable_IsDeleted`, `LoadedExecutable_Execute`, `LoadedExecutable_Fingerprint` (deprecated), `LoadedExecutable_GetDeviceAssignment` |
| **Buffer** | `Buffer_Destroy`, `Buffer_ElementType`, `Buffer_Dimensions`, `Buffer_UnpaddedDimensions`, `Buffer_DynamicDimensionIndices`, `Buffer_GetMemoryLayout`, `Buffer_OnDeviceSizeInBytes`, `Buffer_Device`, `Buffer_Memory`, `Buffer_Delete`, `Buffer_IsDeleted`, `Buffer_CopyToDevice`, `Buffer_CopyToMemory`, `Buffer_ToHostBuffer`, `Buffer_IsOnCpu`, `Buffer_ReadyEvent`, `Buffer_UnsafePointer`, `Buffer_IncreaseExternalReferenceCount`, `Buffer_DecreaseExternalReferenceCount`, `Buffer_OpaqueDeviceMemoryDataPointer`, `Buffer_CopyRawToHost`, `Buffer_CopyRawToHostFuture`, `Buffer_DonateWithControlDependency` |
| **CopyToDeviceStream** | `CopyToDeviceStream_Destroy`, `CopyToDeviceStream_AddChunk`, `CopyToDeviceStream_TotalBytes`, `CopyToDeviceStream_GranuleSize`, `CopyToDeviceStream_CurrentBytes` |
| **TopologyDescription** | `TopologyDescription_Create`, `TopologyDescription_Destroy`, `TopologyDescription_PlatformName`, `TopologyDescription_PlatformVersion`, `TopologyDescription_GetDeviceDescriptions`, `TopologyDescription_Serialize`, `TopologyDescription_Attributes`, `TopologyDescription_Deserialize` |
| **ExecuteContext** | `ExecuteContext_Create`, `ExecuteContext_Destroy` |
| **AsyncHostToDeviceTransferManager** | All functions wrapped |
| **AsyncTrackingEvent** | `AsyncTrackingEvent_Destroy` |
| **Compile** | `Compile` |

### Extensions Implemented

| Extension Type | Status | Notes |
|----------------|--------|-------|
| Layouts | ✅ Implemented | `layouts_ext.rs` |
| Stream | ✅ Implemented | `stream_ext.rs` |
| FFI | ✅ Implemented | `ffi_ext.rs` |
| RawBuffer | ✅ Implemented | `raw_buffer_ext.rs` |
| GPU Custom Call | ✅ Implemented | `gpu_ext.rs` |
| Custom Partitioner | ✅ Implemented | `custom_partitioner_ext.rs` |
| Triton | ✅ Implemented | `triton_ext.rs` |
| Profiler | ✅ Implemented | `profiler_ext.rs` |
| Callback | ✅ Implemented | `callback_ext.rs` |
| MemoryDescriptions | ✅ Implemented | `memory_descriptions_ext.rs` |
| PhaseCompile | ✅ Implemented | `phase_compile_ext.rs` |

---

## TODOs - Improvements & Missing Features

### Priority: High

- [x] **TODO-001: DeviceDescription LocalHardwareId** - RESOLVED
  - Originally thought missing, but `PJRT_DeviceDescription_LocalHardwareId` doesn't exist in C API
  - The C API comments refer to `PJRT_Device_LocalHardwareId` which IS implemented in `Device::local_hardware_id()`
  - No action needed

- [x] **TODO-002: Improve Error Context in FFI Calls** ✅ COMPLETED
  - Added `function` field to `PjrtError` variant in `error.rs`
  - Updated `err_or_with_fn` method and `pjrt_api_fn_ret_err` macro in `api.rs`
  - Error messages now show which PJRT function failed

- [x] **TODO-003: Add SendCallbackInfo and RecvCallbackInfo Support** ✅ COMPLETED
  - Added `SendCallbackInfo` and `RecvCallbackInfo` structs for distributed execution callbacks
  - Added `send_callbacks()` and `recv_callbacks()` methods to `ExecuteOptions`
  - Added `ExecuteOptionsRaw` helper struct to manage callback lifetime during execution
  - Exported `SendCallbackInfo`, `RecvCallbackInfo`, `SendCallback`, `RecvCallback`, `CallbackError` from lib.rs
  - Location: `execute.rs`

### Priority: Medium

- [x] **TODO-004: Add TransferMetadata Support** ✅ COMPLETED
  - `TransferMetadata` struct for async transfers with dims, element_type, and layout
  - Includes helper methods: `new()`, `with_layout()`, `num_elements()`, `size_in_bytes()`
  - Exported from lib.rs
  - Location: `execute.rs`

- [x] **TODO-005: Add NonDonatableInputIndices to ExecuteOptions** ✅ COMPLETED
  - Already implemented with `non_donatable_input_indices()` method on `ExecuteOptions`
  - Added `get_non_donatable_input_indices()` getter method
  - Location: `execute.rs`

- [x] **TODO-006: Implement Missing Extension Types** ✅ COMPLETED
  - CrossHostTransfers extension: `cross_host_transfers_ext.rs`
  - ExecutableMetadata extension: `executable_metadata_ext.rs`
  - HostAllocator extension (experimental): `host_allocator_ext.rs`
  - TpuTopology extension: `tpu_topology_ext.rs`
  - TpuExecutable extension: `tpu_executable_ext.rs`
  - Megascale extension: `megascale_ext.rs`
  - Note: These extensions are marker types (no dedicated C API structs with methods)
  - All exported from lib.rs with proper documentation

- [x] **TODO-007: Add CallLocation Support** ✅ COMPLETED (was already implemented)
  - `CallLocation` struct with `new()`, `from_string()`, and accessor methods
  - `call_location()` method on `ExecuteOptions`
  - Exported from lib.rs
  - Location: `execute.rs`

- [x] **TODO-008: Add Task/Incarnation ID Support in ExecuteOptions** ✅ COMPLETED (was already implemented)
  - `task_incarnation_ids()` method on `ExecuteOptions`
  - Location: `execute.rs`

### Priority: Low

- [x] **TODO-009: Add Example Extension** ✅ COMPLETED
  - Added `Example` variant to `ExtensionType` enum in `extension.rs`
  - `ExampleExtension` struct properly implements the `Extension` trait
  - Comprehensive documentation explaining how extensions work
  - Unit tests for extension type and trait implementation
  - Location: `example_ext.rs`, `extension.rs`

- [x] **TODO-010: Improve HostBufferSemantics Documentation** ✅ COMPLETED
  - Added comprehensive documentation for all four semantics variants
  - Each variant now includes:
    - Detailed behavior explanation
    - Memory safety requirements and contracts
    - When to use (with decision guide)
    - Performance implications
    - Platform-specific behavior tables
    - Code examples for common scenarios
  - Added ASCII decision tree for choosing semantics
  - Added enum-level documentation with comparison tables
  - Location: `host_buffer.rs`

- [x] **TODO-011: Add Higher-Level Async Transfer API** ✅ COMPLETED
  - Added `AsyncTransferBuilder` - simple one-shot transfers with builder pattern
  - Added `TypedAsyncTransfer` - type-safe transfer operation
  - Added `RawAsyncTransfer` - raw bytes transfer operation  
  - Added `MultiBufTransfer` - multi-buffer transfer builder
  - All APIs support both async and sync transfers
  - Automatic cleanup with RAII pattern
  - Comprehensive documentation with examples
  - Location: `async_transfer.rs`
  - Exports: `AsyncTransferBuilder`, `TypedAsyncTransfer`, `RawAsyncTransfer`, `MultiBufTransfer`

---

## Code Quality Improvements

- [x] **TODO-Q01: Add Integration Tests for Extensions** ✅ COMPLETED
  - Added comprehensive unit tests in `pjrt/src/tests/core_types_tests.rs`
  - Tests cover: PrimitiveType, MemoryLayout, ErrorCode, HostBuffer, CompileOptions, ExecuteOptions, NamedValue, Program, DeviceAssignment, BufferShape
  - Extended `extension_tests.rs` with additional tests for ExtensionType variants
  - All tests run without requiring PJRT plugin (no hardware dependencies)
  - Location: `pjrt/src/tests/`

- [x] **TODO-Q02: Improve Debug Implementations** ✅ COMPLETED
  - Improved `AsyncHostToDeviceTransferManager` Debug to show buffer_count and device
  - Note: Most key structs already had good Debug implementations:
    - `Client`: shows platform_name, version, process_index, device counts
    - `Buffer`: shows primitive_type, dims, on_device_size, is_on_cpu, is_deleted
    - `Device`: shows id, kind, is_addressable, local_hardware_id
    - `LoadedExecutable`: shows name, is_deleted, num_addressable_devices
    - `Executable`: shows name, replicas, partitions, outputs, code_size
    - `Event`: shows is_ready, callback_registered
    - `Memory`: uses debug_string() from PJRT
  - Location: `async_transfer.rs` and already implemented in other files

- [x] **TODO-Q03: Add Documentation Examples** ✅ COMPLETED
  - Added comprehensive documentation examples to:
    - `api.rs`: Plugin loading, client creation, extensions usage
    - `client.rs`: Client creation, device iteration, program compilation
    - `buffer.rs`: Buffer creation, transfers, copying between devices
    - `device.rs`: Device properties, memory management, memory statistics
    - `memory.rs`: Memory properties, device relationships, addressable memories
    - `compile.rs`: Compilation options, build options, device assignment
    - `host_buffer.rs`: TypedHostBuffer creation, transfers, HostBuffer enum
    - `program.rs`: Program creation, file loading, format parsing
  - All examples use `rust` or `rust,ignore` code blocks as appropriate
  - Examples demonstrate common usage patterns

- [x] **TODO-Q04: ExternalBufferRef RAII Guard** ✅ COMPLETED
  - Added `ExternalBufferRef<'a>` struct in `buffer.rs`
  - Provides RAII guard that automatically decrements reference count on drop
  - Added `external_ref()` method to `Buffer` that returns the guard
  - Safe API prevents reference count mismanagement

---

## Completed

- [x] Core API binding coverage
- [x] Event async/await support
- [x] Buffer management
- [x] Device management
- [x] Memory management
- [x] Executable compilation and loading
- [x] Extension system framework
- [x] Most extension implementations
- [x] TODO-001: DeviceDescription LocalHardwareId (verified not needed)
- [x] TODO-002: Error context improvement
- [x] TODO-Q04: ExternalBufferRef RAII guard

---

## Notes

1. The pjrt-rs API coverage is quite comprehensive. Most C API functions are wrapped.

2. The main gaps are in advanced distributed execution features (send/recv callbacks, task IDs).

3. Some extension types are TPU-specific or experimental and may not be immediately needed.

4. The safe API successfully abstracts most unsafe FFI details from users.
