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

- [ ] **TODO-003: Add SendCallbackInfo and RecvCallbackInfo Support**
  - The `ExecuteOptions` struct in C API supports send/recv callbacks for distributed execution
  - Currently not exposed in the safe Rust API
  - Location: `execute.rs`

### Priority: Medium

- [ ] **TODO-004: Add TransferMetadata Support**
  - `PJRT_TransferMetadata` struct for async transfers
  - Used with send/recv callbacks
  - Location: `execute.rs`

- [ ] **TODO-005: Add NonDonatableInputIndices to ExecuteOptions**
  - The C API supports specifying which inputs should not be donated
  - Location: `execute.rs`

- [ ] **TODO-006: Implement Missing Extension Types**
  - CrossHostTransfers extension
  - ExecutableMetadata extension
  - HostAllocator extension (experimental)
  - TpuTopology extension
  - TpuExecutable extension
  - Megascale extension

- [ ] **TODO-007: Add CallLocation Support**
  - The `call_location` field in `PJRT_ExecuteOptions` allows passing source location information
  - Useful for debugging and error reporting
  - Location: `execute.rs`

- [ ] **TODO-008: Add Task/Incarnation ID Support in ExecuteOptions**
  - The C API has `num_tasks`, `task_ids`, `incarnation_ids` for distributed execution
  - Location: `execute.rs`

### Priority: Low

- [ ] **TODO-009: Add Example Extension**
  - `PJRT_Extension_Type_Example` exists but is not wrapped
  - Could serve as documentation for how extensions work

- [ ] **TODO-010: Improve HostBufferSemantics Documentation**
  - Document the different semantics (ImmutableOnlyDuringCall, ImmutableUntilTransferCompletes, ImmutableZeroCopy, MutableZeroCopy)
  - Add examples showing when to use each
  - Location: `host_buffer.rs`

- [ ] **TODO-011: Add Higher-Level Async Transfer API**
  - The current async transfer API is low-level
  - Consider adding a more ergonomic high-level wrapper
  - Location: `async_transfer.rs`

---

## Code Quality Improvements

- [ ] **TODO-Q01: Add Integration Tests for Extensions**
  - Most extensions don't have integration tests
  - Add tests that verify extension availability and basic functionality

- [ ] **TODO-Q02: Improve Debug Implementations**
  - Some structs only show pointers in Debug output
  - Add more meaningful information where possible

- [ ] **TODO-Q03: Add Documentation Examples**
  - Many public functions lack doc examples
  - Add `# Examples` sections to public API

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
