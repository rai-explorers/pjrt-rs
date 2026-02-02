//! Execution Context Example
//!
//! This example demonstrates how to use PJRT Execution Context and Options:
//! 1. Creating ExecuteContext for execution state management
//! 2. Configuring ExecuteOptions with launch IDs and non-donatable indices
//! 3. Using Execution builder pattern
//! 4. Understanding input donation and buffer management
//!
//! Execution context is useful for:
//! - Managing execution state across multiple runs
//! - Configuring launch identifiers for debugging/profiling
//! - Controlling which inputs can be donated (reused)
//!
//! To run this example:
//! ```
//! export PJRT_PLUGIN_PATH=/path/to/pjrt_c_api_cpu_plugin.so
//! cargo run --example execution_context
//! ```

use pjrt::ProgramFormat::MLIR;
use pjrt::{self, Client, ExecuteOptions, Execution, HostBuffer, LoadedExecutable, Result};

// Simple MLIR program that performs an identity operation on a float32 scalar
const CODE: &[u8] = include_bytes!("prog_f32.mlir");

fn main() -> Result<()> {
    let plugin_path = std::env::var("PJRT_PLUGIN_PATH")
        .expect("PJRT_PLUGIN_PATH environment variable must be set");
    let api = pjrt::plugin(&plugin_path).load()?;

    let client = Client::builder(&api).build()?;

    println!("Execution Context Example");
    println!("========================\n");

    // Example 1: Basic execution with default options
    demonstrate_basic_execution(&client)?;

    // Example 2: Execution with custom options
    demonstrate_custom_options(&client)?;

    // Example 3: Understanding input donation
    demonstrate_input_donation(&client)?;

    // Example 4: Multiple executions with launch IDs
    demonstrate_launch_ids(&client)?;

    Ok(())
}

/// Demonstrates basic execution without custom options
fn demonstrate_basic_execution(client: &Client) -> Result<()> {
    println!("1. Basic Execution");
    println!("   ----------------");

    let program = pjrt::Program::new(MLIR, CODE);
    let loaded_executable = LoadedExecutable::builder(client, &program).build()?;

    let input = HostBuffer::from_scalar(1.0f32);
    let device_buffer = input.to_sync(client).copy()?;

    // Basic execution - all default options
    let result = loaded_executable.execution(device_buffer).run_sync()?;

    let output = &result[0][0];
    let host_output: HostBuffer = output.to_host_sync(None)?;

    println!("   Input: 1.0");
    println!("   Output: {:?}", host_output);
    println!("   Execution completed with default options\n");

    Ok(())
}

/// Demonstrates execution with custom options
fn demonstrate_custom_options(client: &Client) -> Result<()> {
    println!("2. Execution with Custom Options");
    println!("   ------------------------------");

    let program = pjrt::Program::new(MLIR, CODE);
    let loaded_executable = LoadedExecutable::builder(client, &program).build()?;

    let input = HostBuffer::from_scalar(2.0f32);
    let device_buffer = input.to_sync(client).copy()?;

    // Create execution with custom options
    let execution = Execution::new(&loaded_executable, device_buffer)
        .launch_id(42) // Assign a specific launch ID
        .non_donatable_input_indices(vec![0]); // Mark input 0 as non-donatable

    println!("   Configured execution:");
    println!("     - Launch ID: 42");
    println!("     - Non-donatable input indices: [0]");

    let result = execution.run_sync()?;

    let output = &result[0][0];
    let host_output: HostBuffer = output.to_host_sync(None)?;

    println!("   Input: 2.0");
    println!("   Output: {:?}", host_output);
    println!("   Execution completed with custom options\n");

    Ok(())
}

/// Demonstrates understanding input donation
fn demonstrate_input_donation(client: &Client) -> Result<()> {
    println!("3. Input Donation");
    println!("   ---------------");

    println!("   Input donation allows PJRT to reuse input buffers for outputs,");
    println!("   reducing memory allocations and improving performance.\n");

    let program = pjrt::Program::new(MLIR, CODE);
    let loaded_executable = LoadedExecutable::builder(client, &program).build()?;

    // Create input buffers
    let input1 = HostBuffer::from_scalar(3.0f32);
    let device_buffer1 = input1.to_sync(client).copy()?;

    println!("   Created input buffer");

    // Execution that allows donation (default behavior)
    println!("   Executing with donation allowed (default)...");
    let result = loaded_executable.execution(device_buffer1).run_sync()?;

    let output1 = &result[0][0];
    let host_output1: HostBuffer = output1.to_host_sync(None)?;
    println!("   First execution output: {:?}", host_output1);

    // Create second input buffer (cannot reuse first due to potential donation)
    let input2 = HostBuffer::from_scalar(3.0f32);
    let device_buffer2 = input2.to_sync(client).copy()?;

    // Execution that prevents donation of input 0
    println!("   Executing with donation prevented for input 0...");
    let result2 = loaded_executable
        .execution(device_buffer2)
        .non_donatable_input_indices(vec![0])
        .run_sync()?;

    let output2 = &result2[0][0];
    let host_output2: HostBuffer = output2.to_host_sync(None)?;
    println!("   Second execution output: {:?}", host_output2);

    println!();
    println!("   When to prevent donation:");
    println!("   - When the input buffer is needed for multiple executions");
    println!("   - When the input is a constant/shared weight");
    println!("   - When you need to preserve the input for later use\n");

    Ok(())
}

/// Demonstrates using launch IDs for multiple executions
fn demonstrate_launch_ids(client: &Client) -> Result<()> {
    println!("4. Launch IDs for Multiple Executions");
    println!("   -----------------------------------");

    println!("   Launch IDs help identify specific execution instances,");
    println!("   useful for debugging, profiling, and logging.\n");

    let program = pjrt::Program::new(MLIR, CODE);
    let loaded_executable = LoadedExecutable::builder(client, &program).build()?;

    // Execute multiple times with different launch IDs
    for launch_id in 1..=3 {
        let input = HostBuffer::from_scalar(launch_id as f32);
        let device_buffer = input.to_sync(client).copy()?;

        let execution = Execution::new(&loaded_executable, device_buffer).launch_id(launch_id);

        let result = execution.run_sync()?;
        let output = &result[0][0];
        let host_output: HostBuffer = output.to_host_sync(None)?;

        println!(
            "   Launch ID {}: Input = {}, Output = {:?}",
            launch_id, launch_id as f32, host_output
        );
    }

    println!();
    println!("   Launch ID use cases:");
    println!("   - Profiling specific execution instances");
    println!("   - Correlating logs with execution traces");
    println!("   - Debugging distributed/multi-step computations");
    println!("   - Identifying which step in a training loop caused an error\n");

    println!("   ExecuteOptions can also be created standalone:");
    let _options = ExecuteOptions::new()
        .launch_id(100)
        .non_donatable_input_indices(vec![0, 1]);
    println!("   - Created options with launch_id=100 and non-donatable=[0, 1]\n");

    Ok(())
}
