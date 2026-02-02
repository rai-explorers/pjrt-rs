# PJRT-RS

![Rust](https://github.com/rai-explorers/pjrt-rs/workflows/Rust/badge.svg)
[![Docs Status](https://docs.rs/pjrt/badge.svg)](https://docs.rs/pjrt)
[![Latest Version](https://img.shields.io/crates/v/pjrt.svg)](https://crates.io/crates/pjrt)
[![Discord](https://img.shields.io/discord/1202429682474287144.svg?color=7289da&&logo=discord)](https://discord.gg/J7X8rNZeMC)

**`pjrt`** is a safe [PJRT](https://opensource.googleblog.com/2023/05/pjrt-simplifying-ml-hardware-and-framework-integration.html) C API bindings for Rust. It provides memory-safe, idiomatic Rust interfaces to the PJRT runtime for high-performance machine learning computations.

![](docs/pjrt.png)

## Installation

Add the following to your `Cargo.toml`:

```toml
[dependencies]
pjrt = "0.4"
```

## Getting Started

### 1. Loading the PJRT API

First, you need to load the PJRT API and a plugin for your target hardware:

```rust
use pjrt::Result;

fn main() -> Result<()> {
    // Load the PJRT API
    let api = pjrt::Api::new()?;
    
    // Load a plugin (example uses CPU plugin)
    let plugin = api.load_plugin("pjrt_c_api_cpu_plugin.so")?;
    
    Ok(())
}
```

### 2. Creating a Client

A client represents a runtime instance:

```rust
    // Create a client with default options
    let client = plugin.create_client(vec![])?;
    
    // Or use the builder pattern for more options
    let client = pjrt::Client::builder(&plugin)
        .with_option("allow_index_remapping", "true")?
        .build()?;
```

### 3. Working with Devices

Access available devices and their properties:

```rust
    // Get all available devices
    let devices = client.devices();
    println!("Found {} devices", devices.len());
    
    for (i, device) in devices.iter().enumerate() {
        let desc = device.description()?;
        println!("Device {}: {}", i, desc.device_kind);
    }
```

### 4. Compiling and Executing Programs

```rust
    // Load a program (MLIR or HLO format)
    let program = pjrt::Program::new(
        pjrt::ProgramFormat::MLIR,
        include_bytes!("example.mlir")
    )?;
    
    // Compile to executable
    let executable = client.compile(
        &program,
        &compile_options,
        Some(device_assignment)
    )?;
    
    // Load for execution
    let loaded_executable = executable.load(&client)?;
```

### 5. Managing Buffers

Create and manipulate data buffers:

```rust
    // Create host buffer from scalar
    let input = pjrt::HostBuffer::from_scalar(2.0f32);
    
    // Transfer to device
    let device_buffer = input.to_async(&client).copy()?;
    
    // Execute computation
    let outputs = loaded_executable.execute(&[device_buffer])?;
    
    // Transfer result back to host
    let result = outputs[0].to_async().copy()?;
    println!("Result: {:?}", result);
```

## Quick Example
```rust
use pjrt::ProgramFormat::MLIR;
use pjrt::{self, Client, HostBuffer, LoadedExecutable, Result};

const CODE: &[u8] = include_bytes!("prog_f32.mlir");

fn main() -> Result<()> {
    let api = pjrt::plugin("pjrt_c_api_cpu_plugin.so").load()?;
    println!("api_version = {:?}", api.version());

    let client = Client::builder(&api).build()?;

    println!("platform_name = {}", client.platform_name());

    let program = pjrt::Program::new(MLIR, CODE);

    let loaded_executable = LoadedExecutable::builder(&client, &program).build()?;

    let a = HostBuffer::from_scalar(1.25f32);
    println!("input = {:?}", a);

    let inputs = a.to_sync(&client).copy()?;

    let result = loaded_executable.execution(inputs).run_sync()?;

    let ouput = &result[0][0];
    let output = ouput.to_host_sync().copy()?;
    println!("output= {:?}", output);

    Ok(())
}
```

## LICENSE
This project is licensed under either of

- Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or
  http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or
  http://opensource.org/licenses/MIT)

at your option.
