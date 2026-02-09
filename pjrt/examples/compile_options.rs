//! Compile Options Example
//!
//! This example demonstrates how to use compilation options to customize
//! the PJRT compilation process:
//! 1. Creating CompileOptions with custom settings
//! 2. Setting executable build options
//! 3. Compiling with specific options
//! 4. Viewing serialized compile options from an executable
//!
//! To run this example:
//! ```
//! export PJRT_PLUGIN_PATH=/path/to/pjrt_c_api_cpu_plugin.so
//! cargo run --example compile_options
//! ```

use pjrt::ProgramFormat::MLIR;
use pjrt::{
    self, Client, CompileOptions, CompiledMemoryStats, Executable, ExecutableBuildOptions,
    HostBuffer, Result,
};

// Simple MLIR program that performs an identity operation on a float32 scalar
const CODE: &[u8] = include_bytes!("prog_f32.mlir");

fn main() -> Result<()> {
    let plugin_path = std::env::var("PJRT_PLUGIN_PATH")
        .expect("PJRT_PLUGIN_PATH environment variable must be set");
    let api = pjrt::plugin(&plugin_path).load()?;

    let client = Client::builder(&api).build()?;

    println!("Compile Options Example");
    println!("======================\n");

    // Example 1: Basic compilation with default options
    demonstrate_basic_compilation(&client)?;

    // Example 2: Compilation with custom options
    demonstrate_custom_options(&client)?;

    // Example 3: Compile to Executable (not LoadedExecutable)
    // Note: PJRT_Compile is not supported by all plugins (e.g. CPU).
    if let Err(e) = demonstrate_compile_to_executable(&client, &api) {
        println!("3. Compile to Executable (vs LoadedExecutable)");
        println!("   --------------------------------------------");
        println!("   Skipped: {}\n", e);
    }

    // Example 4: Inspecting compile options from executable
    if let Err(e) = demonstrate_inspect_compile_options(&client) {
        println!("4. Inspecting Compile Options from Executable");
        println!("   -------------------------------------------");
        println!("   Skipped: {}\n", e);
    }

    Ok(())
}

/// Demonstrates basic compilation with default options
fn demonstrate_basic_compilation(client: &Client) -> Result<()> {
    println!("1. Basic Compilation with Default Options");
    println!("   ---------------------------------------");

    let program = pjrt::Program::new(MLIR, CODE);

    // Compile with default options (empty)
    let compile_options = CompileOptions::new();
    let loaded_executable = client.compile(&program, compile_options)?;

    println!("   Compiled successfully with default options");

    // Execute to verify it works
    let input = HostBuffer::from_scalar(2.5f32);
    let device_buffer = input.to_sync(client).copy()?;
    let result = loaded_executable.execution(device_buffer).run_sync()?;

    let output = &result[0][0];
    let host_output: HostBuffer = output.to_host_sync(None)?;
    println!("   Test execution result: {:?}\n", host_output);

    Ok(())
}

/// Demonstrates compilation with custom options
fn demonstrate_custom_options(client: &Client) -> Result<()> {
    println!("2. Compilation with Custom Options");
    println!("   --------------------------------");

    let program = pjrt::Program::new(MLIR, CODE);

    // Create custom executable build options
    let build_options = ExecutableBuildOptions::new();

    // Create compile options with the build options
    let compile_options = CompileOptions::new().executable_build_options(build_options);

    println!("   CompileOptions created with custom build options");

    let _loaded_executable = client.compile(&program, compile_options)?;
    println!("   Compiled successfully with custom options\n");

    Ok(())
}

/// Demonstrates compiling to an Executable (not LoadedExecutable)
fn demonstrate_compile_to_executable(client: &Client, api: &pjrt::Api) -> Result<()> {
    println!("3. Compile to Executable (vs LoadedExecutable)");
    println!("   --------------------------------------------");

    // First, we need a topology description
    let topology = client.topology()?;

    let program = pjrt::Program::new(MLIR, CODE);
    let compile_options = CompileOptions::new();

    // Compile to Executable using the topology
    // This creates an executable that can be serialized and loaded later
    let executable: Executable = api.compile(&program, &topology, compile_options, Some(client))?;

    println!("   Compiled to Executable successfully");
    println!("   Executable name: {}", executable.name()?);
    println!(
        "   Replicas: {}, Partitions: {}",
        executable.num_replicas()?,
        executable.num_partitions()?
    );

    // Get compile options used during compilation
    let serialized_options = executable.compile_options()?;
    let options_bytes = serialized_options.bytes();
    println!(
        "   Serialized compile options: {} bytes",
        options_bytes.len()
    );

    // Get memory statistics
    let memory_stats: CompiledMemoryStats = executable.compiled_memory_stats()?;
    println!("   Memory stats:");
    println!(
        "     - Generated code size: {} bytes",
        memory_stats.generated_code_size_in_bytes
    );
    println!(
        "     - Argument size: {} bytes",
        memory_stats.argument_size_in_bytes
    );
    println!(
        "     - Output size: {} bytes",
        memory_stats.output_size_in_bytes
    );
    println!(
        "     - Alias size: {} bytes\n",
        memory_stats.alias_size_in_bytes
    );

    Ok(())
}

/// Demonstrates inspecting compile options from an executable
fn demonstrate_inspect_compile_options(client: &Client) -> Result<()> {
    println!("4. Inspecting Compile Options from Executable");
    println!("   -------------------------------------------");

    let program = pjrt::Program::new(MLIR, CODE);
    let compile_options = CompileOptions::new();

    // First compile to a regular Executable
    let topology = client.topology()?;
    let executable: Executable =
        client
            .api()
            .compile(&program, &topology, compile_options, Some(client))?;

    // Retrieve the compile options that were used
    let serialized = executable.compile_options()?;
    let bytes = serialized.bytes();

    println!("   Retrieved serialized compile options");
    println!("   Size: {} bytes", bytes.len());

    if bytes.is_empty() {
        println!("   Note: Empty bytes may indicate the backend doesn't support");
        println!("         serialization of compile options\n");
    } else {
        println!(
            "   First 16 bytes (hex): {:02x?}",
            &bytes[..16.min(bytes.len())]
        );
        println!();
    }

    Ok(())
}
