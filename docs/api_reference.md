# PJRT-RS API Reference

This document provides a comprehensive reference for the PJRT-RS public API.

## Table of Contents

1. [Core Types](#core-types)
2. [Plugin Management](#plugin-management)
3. [Client Operations](#client-operations)
4. [Buffer Management](#buffer-management)
5. [Device Management](#device-management)
6. [Compilation](#compilation)
7. [Execution](#execution)
8. [Extensions](#extensions)
9. [Error Handling](#error-handling)

---

## Core Types

### `Api`

The entry point for interacting with PJRT plugins.

```rust
use pjrt::{Api, plugin};

// Load a PJRT plugin
let api = plugin("/path/to/plugin.so").load()?;

// Get API version
let version = api.version();
println!("API: {}.{}", version.major, version.minor);
```

**Methods:**
- `version()` - Get the PJRT API version
- `compile()` - Compile a program to an Executable using a topology

### `Client`

The main interface for device operations and program execution.

```rust
use pjrt::Client;

// Create a client
let client = Client::builder(&api).build()?;

// Get platform information
println!("Platform: {}", client.platform_name());
println!("Version: {}", client.platform_version());
```

**Methods:**
- `platform_name()` - Get the platform name (e.g., "cpu", "gpu", "tpu")
- `platform_version()` - Get the platform version string
- `process_index()` - Get the process index in multi-process setups
- `devices()` - Get all available devices
- `addressable_devices()` - Get devices addressable by this client
- `addressable_memories()` - Get memory spaces addressable by this client
- `compile()` - Compile a program to a LoadedExecutable
- `topology()` - Get the topology description

---

## Plugin Management

### Loading Plugins

```rust
use pjrt::plugin;

// Load from a file path
let api = plugin("/path/to/pjrt_c_api_cpu_plugin.so").load()?;

// The plugin function returns a Plugin struct that can be loaded
```

---

## Client Operations

### Creating a Client

```rust
use pjrt::Client;

// Basic client creation
let client = Client::builder(&api).build()?;

// Client with custom configuration
let client = Client::builder(&api)
    .key_value_store(&kv_store)  // For distributed setups
    .process_index(0)
    .process_count(4)
    .build()?;
```

### Device Discovery

```rust
// List all devices
for device in client.devices() {
    println!("Device: {:?}", device.description().id());
}

// List addressable devices (can execute programs)
for device in client.addressable_devices() {
    println!("Addressable: {:?}", device.description().id());
}

// Lookup specific devices
let device = client.lookup_device(device_id)?;
let device = client.lookup_addressable_device(hardware_id)?;
```

---

## Buffer Management

### Creating Buffers

```rust
use pjrt::{HostBuffer, TypedHostBuffer, F32};

// From a scalar
let buffer = HostBuffer::from_scalar(1.0f32);

// From typed data
let data = vec![1.0f32, 2.0, 3.0];
let typed_buffer = TypedHostBuffer::<F32>::from_data(data, None, None);

// From raw bytes
let bytes = vec![0u8; 1024];
let buffer = HostBuffer::from_bytes(&bytes, dims, layout)?;
```

### Host to Device Transfer

```rust
// Synchronous transfer
let host_buffer = HostBuffer::from_scalar(1.0f32);
let device_buffer = host_buffer.to_sync(&client).copy()?;

// Transfer to specific device
let device_buffer = host_buffer.to_sync(&device).copy()?;

// Transfer to specific memory space
let device_buffer = host_buffer.to_sync(&memory).copy()?;
```

### Device to Host Transfer

```rust
// Synchronous transfer
let host_buffer: HostBuffer = device_buffer.to_host_sync(None)?;

// With custom layout
let host_buffer = device_buffer.to_host_sync(Some(layout))?;

// Raw data transfer
let mut dst = vec![0u8; size];
device_buffer.copy_raw_to_host_sync(&mut dst, offset)?;
```

### Device to Device Transfer

```rust
// Copy buffer to another device
let buffer_on_device2 = buffer_on_device1.to_device_sync(&device2).copy()?;

// Copy to specific memory space
let buffer_in_memory = buffer.to_memory_sync(&memory).copy()?;
```

### Buffer Properties

```rust
// Query buffer properties
let dtype = buffer.primitive_type();
let dims = buffer.dims();
let unpadded_dims = buffer.unpadded_dims();
let layout = buffer.layout();
let size = buffer.on_device_size();

// Check buffer state
let is_on_cpu = buffer.is_on_cpu();
let is_deleted = buffer.is_deleted();

// Get associated resources
let device = buffer.device();
let memory = buffer.memory();
```

---

## Device Management

### Device Information

```rust
let device = client.addressable_devices()[0];

// Basic info
let is_addressable = device.is_addressable();
let hardware_id = device.local_hardware_id();

// Get description
let desc = device.description();
let device_id = desc.id();
let kind = desc.kind();
let attributes = desc.attributes();
```

### Memory Management

```rust
// Get device memories
let memories = device.addressable_memories();
let default_memory = device.default_memory();

// Query memory info
for memory in &memories {
    println!("Memory: {} (kind: {})", memory.id(), memory.kind());
    
    // Check which devices can access this memory
    let accessible = memory.addressable_by_devices();
}

// Memory statistics
match device.memory_stats() {
    Ok(stats) => {
        println!("Bytes in use: {}", stats.bytes_in_use);
        if stats.bytes_limit_is_set {
            println!("Limit: {}", stats.bytes_limit);
        }
    }
    Err(e) => println!("Stats unavailable: {:?}", e),
}
```

---

## Compilation

### CompileOptions

```rust
use pjrt::{CompileOptions, ExecutableBuildOptions};

// Basic options
let options = CompileOptions::new();

// With build options
let build_opts = ExecutableBuildOptions::new()
    .num_replicas(1)
    .num_partitions(1)
    .device_ordinal(0);

let options = CompileOptions::new()
    .executable_build_options(build_opts);
```

### Compiling to LoadedExecutable

```rust
use pjrt::{Program, ProgramFormat};

// Create a program
let program = Program::new(ProgramFormat::MLIR, code);

// Compile
let loaded_executable = client.compile(&program, &options)?;

// Execute
let result = loaded_executable.execution(input_buffer).run_sync()?;
```

### Compiling to Executable

```rust
// Compile to Executable (can be serialized)
let topology = client.topology();
let executable = api.compile(&program, &topology, options, Some(&client))?;

// Query executable info
println!("Name: {}", executable.name());
println!("Code size: {} bytes", executable.code_size());
println!("Replicas: {}", executable.num_replicas());
println!("Partitions: {}", executable.num_partitions());

// Get compile options used
let serialized_options = executable.compile_options();

// Serialize executable
let serialized = executable.serialize();
std::fs::write("model.pjrt", serialized.bytes())?;
```

---

## Execution

### ExecuteOptions

```rust
use pjrt::ExecuteOptions;

// Create options
let options = ExecuteOptions::new()
    .launch_id(1)  // For profiling/debugging
    .non_donatable_input_indices(vec![0]);  // Prevent input donation
```

### Running Executions

```rust
// Basic execution
let result = loaded_executable
    .execution(input_buffer)
    .run_sync()?;

// With custom options
let result = loaded_executable
    .execution(input_buffer)
    .launch_id(42)
    .non_donatable_input_indices(vec![0])
    .run_sync()?;

// Async execution (requires async runtime)
let result = loaded_executable
    .execution(input_buffer)
    .run().await?;
```

### Handling Results

```rust
let outputs = result[0];  // Get outputs for first device
let output_buffer = &outputs[0];  // Get first output

// Transfer to host
let host_output = output_buffer.to_host_sync(None)?;
```

---

## Extensions

### Accessing Extensions

```rust
use pjrt::{LayoutsExt, CallbackExt};

// Layouts extension
if let Some(layouts_ext) = client.layouts_extension() {
    // Use layouts extension
}

// Callback extension
if let Some(callback_ext) = client.callback_extension() {
    // Register callbacks
}
```

### Available Extensions

- `CallbackExtension` - Custom callbacks for events
- `LayoutsExtension` - Memory layout operations
- `StreamExtension` - Device stream management
- `FfiExtension` - Foreign function interface
- `GpuExtension` - GPU-specific operations
- `ProfilerExtension` - Performance profiling
- `MemoryDescriptionsExtension` - Memory kind descriptions

---

## Error Handling

### Error Types

```rust
use pjrt::{Error, ErrorCode, Result};

match result {
    Ok(value) => println!("Success: {:?}", value),
    Err(Error::PjrtError { code, msg, .. }) => {
        println!("PJRT Error {:?}: {}", code, msg);
    }
    Err(Error::InvalidArgument(msg)) => {
        println!("Invalid argument: {}", msg);
    }
    Err(e) => println!("Error: {:?}", e),
}
```

### Error Codes

- `Ok` - Success
- `Cancelled` - Operation cancelled
- `Unknown` - Unknown error
- `InvalidArgument` - Invalid argument provided
- `DeadlineExceeded` - Operation timed out
- `NotFound` - Resource not found
- `AlreadyExists` - Resource already exists
- `PermissionDenied` - Insufficient permissions
- `ResourceExhausted` - Out of resources
- `FailedPrecondition` - Precondition failed
- `Aborted` - Operation aborted
- `OutOfRange` - Index out of range
- `Unimplemented` - Feature not implemented
- `Internal` - Internal error
- `Unavailable` - Service unavailable
- `DataLoss` - Data loss/corruption
- `Unauthenticated` - Authentication required

---

## Examples

See the `examples/` directory for complete working examples:

- `basic.rs` - Basic usage
- `compile_options.rs` - Compilation options
- `event.rs` - Event handling
- `execution_context.rs` - Execution configuration
- `kv_store.rs` - Key-value store
- `memory.rs` - Memory management
- `multi_device.rs` - Multi-device operations
- `callback_extension.rs` - Callback extension
- `memory_layout.rs` - Layout extension

---

## Best Practices

1. **Error Handling**: Always handle errors appropriately using `Result`
2. **Resource Management**: Buffers and executables are automatically cleaned up via `Drop`
3. **Memory Transfer**: Minimize host-device transfers for better performance
4. **Device Selection**: Use `lookup_device()` or `lookup_addressable_device()` for specific devices
5. **Input Donation**: Allow input donation when inputs won't be reused
6. **Async Operations**: Use async operations for better throughput when possible

---

## See Also

- [API Coverage Analysis](api_coverage.md)
- [Performance Tuning Guide](performance_tuning.md)
- [Troubleshooting Guide](troubleshooting.md)
- [Unsafe API Patterns](unsafe_patterns.md)
