//! Event Example
//!
//! This example demonstrates PJRT's asynchronous operation model.
//! Events are used internally to coordinate operations.
//!
//! To run this example:
//! ```
//! export PJRT_PLUGIN_PATH=/path/to/pjrt_c_api_cpu_plugin.so
//! cargo run --example event
//! ```

use std::time::Instant;

use pjrt::ProgramFormat::MLIR;
use pjrt::{self, Client, HostBuffer, LoadedExecutable, Result};

const CODE: &[u8] = include_bytes!("prog_f32.mlir");

fn main() -> Result<()> {
    let plugin_path = std::env::var("PJRT_PLUGIN_PATH")
        .expect("PJRT_PLUGIN_PATH environment variable must be set");
    let api = pjrt::plugin(&plugin_path).load()?;
    let client = Client::builder(&api).build()?;

    println!("Event Example");
    println!("=============\n");

    demonstrate_async_operations(&client)?;
    demonstrate_execution_timing(&client)?;
    demonstrate_buffer_transfers(&client)?;

    Ok(())
}

/// Demonstrates asynchronous buffer operations
fn demonstrate_async_operations(client: &Client) -> Result<()> {
    println!("1. Asynchronous Buffer Operations");
    println!("   ------------------------------");

    let input = HostBuffer::from_scalar(1.0f32);
    let _buffer = input.to_sync(client).copy()?;

    println!("   Buffer created and ready for use");
    println!("   PJRT tracks buffer readiness internally using events\n");

    Ok(())
}

/// Demonstrates timing execution operations
fn demonstrate_execution_timing(client: &Client) -> Result<()> {
    println!("2. Execution Timing");
    println!("   ----------------");

    let program = pjrt::Program::new(MLIR, CODE);
    let loaded_executable = LoadedExecutable::builder(client, &program).build()?;

    let input = HostBuffer::from_scalar(std::f32::consts::PI);
    let device_buffer = input.to_sync(client).copy()?;

    let start = Instant::now();
    let result = loaded_executable.execution(device_buffer).run_sync()?;
    let execution_time = start.elapsed();

    println!("   Program executed in {:?}", execution_time);

    let output_buffer = &result[0][0];
    let host_output: HostBuffer = output_buffer.to_host_sync(None)?;
    println!("   Output retrieved: {:?}", host_output);
    println!();

    Ok(())
}

/// Demonstrates multi-device buffer transfers
fn demonstrate_buffer_transfers(client: &Client) -> Result<()> {
    println!("3. Multi-Device Buffer Transfers");
    println!("   -----------------------------");

    let devices = client.addressable_devices()?;
    if devices.len() < 2 {
        println!("   Skipping: Need at least 2 devices");
        println!();
        return Ok(());
    }

    let device0 = &devices[0];
    let device1 = &devices[1];

    println!(
        "   Using devices: {:?} and {:?}",
        device0.description()?.id()?,
        device1.description()?.id()?
    );

    let input = HostBuffer::from_scalar(42.0f32);
    let buffer0 = input.to_sync(device0).copy()?;

    println!("   Buffer created on device 0");

    // Copy to device 1
    let buffer1 = buffer0.to_device_sync(device1).copy()?;
    println!("   Buffer copied to device 1 (async internally)");

    // Verify the copy worked
    let output: HostBuffer = buffer1.to_host_sync(None)?;
    println!("   Transfer result: {:?}", output);
    println!();

    Ok(())
}
