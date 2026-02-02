//! Basic PJRT Example
//!
//! This example demonstrates fundamental PJRT operations:
//! 1. Loading the PJRT API from a plugin
//! 2. Creating a client
//! 3. Loading and compiling a program
//! 4. Executing the program with input data
//! 5. Retrieving the output
//!
//! To run this example:
//! ```
//! export PJRT_PLUGIN_PATH=/path/to/pjrt_c_api_cpu_plugin.so
//! cargo run --example basic
//! ```

use pjrt::ProgramFormat::MLIR;
use pjrt::{self, Buffer, Client, HostBuffer, LoadedExecutable, Result};

// Simple MLIR program that performs an identity operation on a float32 scalar
const CODE: &[u8] = include_bytes!("prog_f32.mlir");

fn main() -> Result<()> {
    let plugin_path = std::env::var("PJRT_PLUGIN_PATH")
        .expect("PJRT_PLUGIN_PATH environment variable must be set");
    let api = pjrt::plugin(&plugin_path).load()?;
    println!("api_version = {:?}", api.version());

    let client = Client::builder(&api).build()?;

    println!("platform_name = {}", client.platform_name());

    let program = pjrt::Program::new(MLIR, CODE);

    let loaded_executable = LoadedExecutable::builder(&client, &program).build()?;

    let a = HostBuffer::from_scalar(1.25f32);
    println!("input = {:?}", a);

    let inputs: Buffer = a.to_sync(&client).copy()?;

    let result = loaded_executable.execution(inputs).run_sync()?;

    let ouput = &result[0][0];
    let output: HostBuffer = ouput.to_host_sync(None)?;
    println!("output= {:?}", output);

    Ok(())
}
