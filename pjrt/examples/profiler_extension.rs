//! Profiler Extension Example
//!
//! This example demonstrates the PJRT Profiler extension, which provides
//! performance profiling and tracing capabilities. This is useful for:
//! - Profiling execution performance
//! - Collecting trace data for analysis  
//! - Identifying bottlenecks in compilation and execution
//!
//! To run this example:
//! ```
//! export PJRT_PLUGIN_PATH=/path/to/pjrt_c_api_plugin.so
//! cargo run --example profiler_extension
//! ```

use pjrt::ProgramFormat::MLIR;
use pjrt::{self, Client, HostBuffer, ProfilerExtension, Result};

const CODE: &[u8] = include_bytes!("prog_f32.mlir");

fn main() -> Result<()> {
    let plugin_path = std::env::var("PJRT_PLUGIN_PATH")
        .expect("PJRT_PLUGIN_PATH environment variable must be set");
    let api = pjrt::plugin(&plugin_path).load()?;
    let client = Client::builder(&api).build()?;

    println!("Profiler Extension Example");
    println!("==========================\n");

    let profiler_ext = match api.get_extension::<ProfilerExtension>() {
        Some(ext) => {
            println!("Profiler extension: available");
            ext
        }
        None => {
            println!("Profiler extension is not available in this plugin.");
            println!("This extension is typically available on GPU/TPU backends.");
            println!("\nThe Profiler extension provides:");
            println!("  - profiler_api()              — get the profiling API");
            println!("  - ProfilerApi::create(opts)   — create a profiler session");
            println!("  - Profiler::start()           — start profiling");
            println!("  - Profiler::stop()            — stop profiling");
            println!("  - Profiler::collect_data()    — collect serialized trace data");
            println!("  - traceme_context_id()        — get traceme context ID");
            return Ok(());
        }
    };

    // 1. Check profiler API availability
    println!("\n1. Profiler API");
    println!("   ------------");
    println!("   Has profiler API: {}", profiler_ext.has_profiler_api());
    println!(
        "   Traceme context ID: {}",
        profiler_ext.traceme_context_id()
    );

    // 2. Create and use a profiler session
    if let Some(profiler_api) = profiler_ext.profiler_api() {
        println!("\n2. Profiling Session");
        println!("   -----------------");

        // Create a profiler with default options
        let mut profiler = profiler_api.create("")?;
        println!("   Profiler session created");

        // Start profiling
        profiler.start()?;
        println!("   Profiling started");

        // Run some work
        let device = &client.addressable_devices()?[0];
        let program = pjrt::Program::new(MLIR, CODE);
        let loaded_exe = client.compile(&program, pjrt::CompileOptions::new())?;

        for i in 0..5 {
            let host_buf = HostBuffer::from_scalar(i as f32);
            let device_buf = host_buf.to_sync(device).copy()?;
            let _result = loaded_exe.execution(device_buf).run_sync()?;
        }
        println!("   Executed 5 iterations");

        // Stop profiling
        profiler.stop()?;
        println!("   Profiling stopped");

        // Collect trace data
        let data = profiler.collect_data()?;
        println!("   Collected {} bytes of trace data", data.len());
        if !data.is_empty() {
            println!("   First 32 bytes: {:?}", &data[..data.len().min(32)]);
        }
    } else {
        println!("\n   Profiler API not available (args-only extension)");
    }

    Ok(())
}
