# PJRT-RS Implementation TODO

**Created:** January 31, 2026  
**API Version:** 0.90

This document tracks the implementation tasks needed to close the gap between the PJRT C API and the Rust bindings.

---

## Recently Completed (February 1, 2026)

### API Coverage Analysis
- ✅ Reviewed all PJRT C API v0.90 functions (124 total)
- ✅ Documented Rust implementation status in `api_coverage.md`
- ✅ Identified gaps between C API and Rust bindings
- **Overall Coverage:** 97% (116/119 core functions fully implemented, excluding extensions)

### February 1, 2026 - Test Fixes and Additional Work

1. ✅ **Fixed Compilation Errors in Example Files**
   - Fixed `async_transfer.rs`: Changed `AsyncTransferShape` to `BufferShape`
   - Fixed `callback_extension.rs`: Added `CallbackExt` trait import
   - Fixed `memory_layout.rs`: Added `LayoutsExt` trait import, fixed borrow checker issue
   - Fixed `buffer_reference_count.rs`: Fixed pointer checks and data reading
   - All examples now compile without errors

2. ✅ **Exported Missing Public APIs**
   - Exported `TpuSliceFailureType` in `lib.rs`
   - Exported `LayoutsExt` trait for accessing layouts extension
   - Exported `CallbackExt` trait for accessing callback extension

3. ✅ **Implemented Extension Access Traits for Client**
   - Implemented `LayoutsExt` trait with `layouts_extension()` method
   - Implemented `CallbackExt` trait with `callback_extension()` method
   - Both traits allow accessing PJRT extensions from Client instances

4. ✅ **Added Missing Methods**
   - Added `size()` method to `LayoutsMemoryLayout`
   - Added `read_f32()` method to `HostBuffer` for reading F32 data
   - Added `primitive_type()` method to `HostBuffer`

5. ✅ **Added Debug Implementations**
   - Added `Debug` impl for `AsyncHostToDeviceTransferManager`
   - Added `Debug` impl for `BufferShape`
   - Total test coverage maintained with 162 tests (up from 157)

6. ✅ **Added Unit Tests**
   - Added tests for buffer external reference counting API structure
   - Added tests for CopyRawToHostFuture API structure
   - Added tests for DonateWithControlDependency API structure
   - Added tests for Client extension traits

### High Priority Implementations
1. ✅ **Buffer External Reference Counting APIs** (`pjrt/src/buffer.rs`)
   - Implemented `Buffer::unsafe_pointer()` - Returns platform-dependent buffer address
   - Implemented `Buffer::increase_external_ref_count()` - Prevents buffer deletion during external use
   - Implemented `Buffer::decrease_external_ref_count()` - Releases external reference
   - Implemented `Buffer::opaque_device_memory_pointer()` - Returns device memory pointer
   - All methods properly marked as `unsafe` with comprehensive safety documentation

2. ✅ **Executable Get Compile Options** (`pjrt/src/executable.rs`)
   - Implemented `Executable::compile_options()` - Returns serialized compile options
   - Created `SerializedCompileOptions` struct with automatic memory management
   - Properly exported in public API

3. ✅ **Key-Value Store Try-Get Callback** (`pjrt/src/kv_store.rs`)
   - Added `KeyValueStore::try_get()` method to the trait
   - Implemented `kv_try_get_callback` function for C API integration
   - Updated `Client::builder()` to use the try-get callback when creating clients
   - Returns `Result<Option<String>>` for non-blocking key lookup

4. ✅ **Extension Framework Infrastructure** (`pjrt/src/extension.rs`)
   - Created `ExtensionType` enum for all 19 PJRT extension types
   - Defined `Extension` trait for extensibility
   - Implemented `ExtensionIterator` for traversing extension chains
   - Added helper functions `find_extension()` and `has_extension()`
   - Created placeholder structs for future extension implementations
   - Exported in public API via `lib.rs`

### Documentation Updates
- ✅ Updated `api_coverage.md` with new implementations
- ✅ Coverage statistics updated (Buffer Operations: 85% → 100%, Executable Operations: 92% → 100%)
- ✅ Extension section updated to reflect framework implementation

### Code Quality
- ✅ All new code compiles without errors (only expected dead code warnings for extension framework)
- ✅ Proper error handling with `Result` types
- ✅ Comprehensive documentation for all public APIs
- ✅ Memory safety via proper Drop implementations

---

## High Priority Tasks

### 1. Buffer External Reference Counting APIs ✅ COMPLETED
**Status:** ✅ Implemented in safe API with proper unsafe markers
**Files:** `pjrt/src/buffer.rs`
**C APIs:**
- ✅ `PJRT_Buffer_UnsafePointer` - `Buffer::unsafe_pointer()`
- ✅ `PJRT_Buffer_IncreaseExternalReferenceCount` - `Buffer::increase_external_ref_count()`
- ✅ `PJRT_Buffer_DecreaseExternalReferenceCount` - `Buffer::decrease_external_ref_count()`
- ✅ `PJRT_Buffer_OpaqueDeviceMemoryDataPointer` - `Buffer::opaque_device_memory_pointer()`

**Rationale:** These APIs are needed for interop with other frameworks (NumPy, dlpack, PyTorch, etc.) that need direct access to buffer memory.

**Implementation Notes:**
- All methods are marked as `unsafe` due to raw pointer operations
- Comprehensive safety documentation added
- Methods allow external frameworks to manage buffer lifecycle
- Proper reference counting prevents use-after-free scenarios

**Implementation Details:**
- Added to `pjrt/src/buffer.rs` in a separate `impl Buffer` block
- Each method includes detailed safety documentation
- Methods work together: `increase_external_ref_count()` must be called before obtaining pointers
- `decrease_external_ref_count()` must be called when done to avoid memory leaks

---

### 2. Executable Get Compile Options ✅ COMPLETED
**Status:** ✅ Implemented in safe API
**Files:** `pjrt/src/executable.rs`
**C API:** `PJRT_Executable_GetCompileOptions`

**Rationale:** Useful for debugging and serialization to understand what compile options were used.

**Implementation Details:**
- Added `Executable::compile_options()` method that returns `SerializedCompileOptions`
- Created `SerializedCompileOptions` struct with proper memory management (Drop impl)
- Returns serialized `CompileOptionsProto` bytes
- Added `bytes()` method to access the serialized data
- Properly exported in `lib.rs`

**Usage Example:**
```rust
let executable = client.compile(&program, &options)?;
let compile_options = executable.compile_options();
let bytes = compile_options.bytes();
// bytes can be deserialized using XLA protobuf definitions
```

---

### 3. Key-Value Store Try-Get Callback ✅ COMPLETED
**Status:** ✅ Fully implemented
**Files:** `pjrt/src/kv_store.rs`, `pjrt/src/api.rs`
**C API:** `PJRT_KeyValueTryGetCallback`

**Rationale:** Needed for distributed/multi-node setups where some processes may not have all keys.

**Implementation Details:**
- ✅ Added `KeyValueStore::try_get(&self, key: &str) -> Result<Option<String>>` to the trait
- ✅ Implemented `kv_try_get_callback` unsafe extern "C" function for C API integration
- ✅ Updated `Api::create_client()` to set both callback and user_arg for try-get
- ✅ Returns `Ok(Some(value))` if key exists, `Ok(None)` if not found, `Err` for other errors
- ✅ Uses `PJRT_Error_Code_PJRT_Error_Code_NOT_FOUND` when key is not found

---

## Medium Priority Tasks

### 4. PJRT Extension APIs

**Status:** ✅ Framework implemented, specific extensions require header files
**Files:** `pjrt/src/extension.rs`

**Completed:**
- ✅ Created extension framework infrastructure
- ✅ Defined `ExtensionType` enum covering all 19 PJRT extension types
- ✅ Implemented `Extension` trait for extensibility
- ✅ Created `ExtensionIterator` for traversing extension chains
- ✅ Added helper functions for finding and checking extensions
- ✅ Created placeholder structs for major extensions
- ✅ Exported in public API

**Prerequisites for Specific Extensions:**
The following extension headers need to be added to the project to implement specific extension APIs:
- `pjrt_c_api_profiler_extension.h`
- `pjrt_c_api_stream_extension.h`
- `pjrt_c_api_layouts_extension.h`
- `pjrt_c_api_ffi_extension.h`
- `pjrt_c_api_cross_host_transfers_extension.h`
- And other platform-specific extension headers

Once headers are available, the following extensions can be implemented:

#### 4.1 Profiler Extension
**Extension Type:** `PJRT_Extension_Type_Profiler`
**Priority:** Medium  
**Status:** Placeholder struct created, awaiting header
**Placeholder:** `ProfilerExtension` in `extension.rs`

#### 4.2 Stream Extension
**Extension Type:** `PJRT_Extension_Type_Stream`
**Priority:** Medium  
**Status:** Placeholder struct created, awaiting header
**Placeholder:** `StreamExtension` in `extension.rs`

#### 4.3 Layouts Extension
**Extension Type:** `PJRT_Extension_Type_Layouts`
**Priority:** Medium  
**Status:** Placeholder struct created, awaiting header
**Placeholder:** `LayoutsExtension` in `extension.rs`

#### 4.4 FFI Extension
**Extension Type:** `PJRT_Extension_Type_FFI`
**Priority:** Medium  
**Status:** Placeholder struct created, awaiting header
**Placeholder:** `FfiExtension` in `extension.rs`

#### 4.5 Memory Descriptions Extension
**Extension Type:** `PJRT_Extension_Type_MemoryDescriptions`
**Priority:** Medium  
**Status:** Awaiting header file

#### 4.6 Cross-Host Transfers Extension
**Extension Type:** `PJRT_Extension_Type_CrossHostTransfers`
**Priority:** Medium  
**Status:** Placeholder struct created, awaiting header
**Placeholder:** `CrossHostTransfersExtension` in `extension.rs`

#### 4.7 Callback Extension
**Extension Type:** `PJRT_Extension_Type_Callback`
**Priority:** Medium  
**Rationale:** Custom callbacks for various operations

**Tasks:**
- [ ] Define callback extension header bindings
- [ ] Create `callback.rs` module
- [ ] Implement callback registration APIs
- [ ] Add documentation

---

## Low Priority Tasks

### 5. Advanced Extension APIs

These extensions are platform-specific or experimental:

#### 5.1 GPU Custom Call Extension
**Extension Type:** `PJRT_Extension_Type_Gpu_Custom_Call`
**Priority:** Low  
**Rationale:** GPU-specific custom operations

**Tasks:**
- [ ] Define GPU custom call extension
- [ ] Create `gpu_custom_call.rs` module
- [ ] Implement custom call registration

#### 5.2 Custom Partitioner Extension
**Extension Type:** `PJRT_Extension_Type_Custom_Partitioner`
**Priority:** Low  
**Rationale:** Custom partitioning for model parallelism

**Tasks:**
- [ ] Define custom partitioner extension
- [ ] Create `partitioner.rs` module
- [ ] Implement partitioner APIs

#### 5.3 Triton Extension
**Extension Type:** `PJRT_Extension_Type_Triton`
**Priority:** Low  
**Rationale:** Triton kernel support

**Tasks:**
- [ ] Define Triton extension
- [ ] Create `triton.rs` module
- [ ] Implement Triton compilation APIs

#### 5.4 Raw Buffer Extension
**Extension Type:** `PJRT_Extension_Type_RawBuffer`
**Priority:** Low  
**Rationale:** Raw buffer access (experimental)

**Tasks:**
- [ ] Define raw buffer extension
- [ ] Create `raw_buffer.rs` module
- [ ] Implement raw buffer APIs

#### 5.5 Phase Compile Extension
**Extension Type:** `PJRT_Extension_Type_PhaseCompile`
**Priority:** Low  
**Rationale:** Phase-based compilation (experimental)

**Tasks:**
- [ ] Define phase compile extension
- [ ] Create `phase_compile.rs` module
- [ ] Implement phase compile APIs

#### 5.6 TPU-Specific Extensions
**Extension Types:** `PJRT_Extension_Type_TpuTopology`, `PJRT_Extension_Type_TpuExecutable`
**Priority:** Low  
**Rationale:** TPU-specific features

**Tasks:**
- [ ] Define TPU topology extension
- [ ] Define TPU executable extension
- [ ] Create `tpu.rs` module
- [ ] Implement TPU-specific APIs

#### 5.7 Host Allocator Extension
**Extension Type:** `PJRT_Extension_Type_HostAllocator`
**Priority:** Low  
**Rationale:** Custom host memory allocation (experimental)

**Tasks:**
- [ ] Define host allocator extension
- [ ] Create `host_allocator.rs` module
- [ ] Implement allocator APIs

#### 5.8 Executable Metadata Extension
**Extension Type:** `PJRT_Extension_Type_ExecutableMetadata`
**Priority:** Low  
**Rationale:** Additional executable metadata

**Tasks:**
- [ ] Define executable metadata extension
- [ ] Extend `executable.rs` with metadata APIs

#### 5.9 Megascale Extension
**Extension Type:** `PJRT_Extension_Type_Megascale`
**Priority:** Low  
**Rationale:** Large-scale distributed training

**Tasks:**
- [ ] Define megascale extension
- [ ] Create `megascale.rs` module
- [ ] Implement megascale APIs

---

## Documentation Tasks

### 6. API Documentation Improvements

**Priority:** Medium

**Tasks:**
- [ ] Add examples for all public APIs
- [ ] Create comprehensive API reference
- [ ] Add performance tuning guide
- [ ] Document unsafe API usage patterns
- [ ] Add troubleshooting guide

---

## Testing Tasks

### 7. Test Coverage Improvements

**Priority:** High

**Tasks:**
- [x] Add unit tests for buffer reference counting APIs ✅ COMPLETED (4 API structure tests added)
- [ ] Add integration tests for multi-device scenarios
- [x] Add tests for error buffer creation ✅ COMPLETED (API structure test in client.rs)
- [x] Add tests for alias buffer functionality ✅ COMPLETED (API structure test in client.rs)
- [ ] Add tests for async transfer manager (runtime tests require PJRT plugin)
- [ ] Add tests for device stream operations (runtime tests require PJRT plugin)
- [ ] Add tests for memory layout conversions (runtime tests require PJRT plugin with layouts extension)
- [ ] Add tests for all buffer types (f8, f4, s2, u2, etc.) - awaiting upstream support

---

## Refactoring Tasks

### 8. Code Quality Improvements

**Priority:** Medium

**Tasks:**
- [x] Review and improve error handling consistency ✅ COMPLETED (consistent Result types used throughout)
- [x] Add more Debug impls for public types ✅ COMPLETED (added for AsyncHostToDeviceTransferManager, BufferShape)
- [ ] Review unsafe code blocks for soundness (ongoing - all unsafe blocks documented)
- [ ] Add benchmarks for critical paths (requires integration testing setup)
- [ ] Optimize memory allocations in hot paths (performance work - future iteration)
- [ ] Reduce code duplication between sync/async variants (refactoring work - future iteration)

---

## Completed Tasks

- [x] Explore codebase structure
- [x] Review PJRT C APIs (v0.90)
- [x] Document Rust API implementations
- [x] Identify gaps between C API and Rust bindings
- [x] Create API coverage documentation (`api_coverage.md`)
- [x] Create implementation todo list (`todo.md`)
- [x] Implement buffer external reference counting APIs (`Buffer::unsafe_pointer`, `increase_external_ref_count`, `decrease_external_ref_count`, `opaque_device_memory_pointer`)
- [x] Implement `Executable::compile_options()` method with `SerializedCompileOptions` struct
- [x] Implement KV Store Try-Get callback (`KeyValueStore::try_get` method and `kv_try_get_callback`)
- [x] Create extension framework infrastructure (`extension.rs` with types, traits, and utilities)
- [x] Update `api_coverage.md` with new implementations and improved statistics (97% coverage)
- [x] Fix compilation warnings in `raw_buffer_ext.rs`, `gpu_ext.rs`, and `ffi_ext.rs`
- [x] Verify all code compiles without critical warnings (only expected dead code warnings remain)
- [x] Add Debug implementations for Buffer, CopyRawToHostFuture, DonateWithControlDependency, Event, Client, FulfillAliasBufferCallback
- [x] Add unit tests for buffer external reference counting APIs (4 tests)
- [x] Add unit tests for client operations including ProcessInfo (8 tests)
- [x] Total test count increased from 146 to 162 tests (all passing)
- [x] Fixed all compilation errors in example files (4 files updated)
- [x] Added Debug implementations for AsyncHostToDeviceTransferManager and BufferShape
- [x] Exported TpuSliceFailureType, LayoutsExt, and CallbackExt in public API
- [x] Implemented LayoutsExt and CallbackExt traits for Client
- [x] Added read_f32() and primitive_type() methods to HostBuffer

---

## Notes

### Implementation Guidelines

1. **Safety First:** All new APIs should be memory-safe unless they are explicitly marked unsafe
2. **Documentation:** Every public API must have comprehensive documentation with examples
3. **Testing:** Every new feature must have unit and integration tests
4. **Consistency:** Follow existing code style and patterns
5. **Error Handling:** Use the `Result` type consistently with proper error messages

### Priority Definitions

- **High:** Critical for basic functionality or commonly used features
- **Medium:** Important for advanced use cases or performance
- **Low:** Nice-to-have or platform-specific features

### Extension Implementation Strategy

Extensions should be implemented as:
1. First, add bindings in `pjrt-sys` (low-level unsafe bindings)
2. Then, create safe wrappers in `pjrt` crate
3. Follow the extension trait pattern (see existing extension implementations)
4. Add tests and documentation

---

**Last Updated:** February 1, 2026
