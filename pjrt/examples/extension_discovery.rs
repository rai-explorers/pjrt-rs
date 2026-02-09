//! Extension Discovery Example
//!
//! This example demonstrates how to discover and check for available
//! PJRT extensions on a plugin. Extensions provide additional functionality
//! beyond the core PJRT API, such as profiling, custom layouts, FFI handlers,
//! and hardware-specific features.
//!
//! To run this example:
//! ```
//! export PJRT_PLUGIN_PATH=/path/to/pjrt_c_api_plugin.so
//! cargo run --example extension_discovery
//! ```

use pjrt::{self, Client, Result};

fn main() -> Result<()> {
    let plugin_path = std::env::var("PJRT_PLUGIN_PATH")
        .expect("PJRT_PLUGIN_PATH environment variable must be set");
    let api = pjrt::plugin(&plugin_path).load()?;
    let client = Client::builder(&api).build()?;

    println!("Extension Discovery Example");
    println!("===========================\n");
    println!("Plugin: {}", plugin_path);
    println!(
        "Platform: {} (version: {})",
        client.platform_name()?,
        client.platform_version()?
    );
    println!("API version: {:?}\n", api.version());

    println!("Available Extensions:");
    println!("---------------------");

    macro_rules! check_ext {
        ($label:expr, $ty:ty) => {
            let status = if api.has_extension::<$ty>() {
                "YES"
            } else {
                "no"
            };
            println!("  {:28} {}", concat!($label, ":"), status);
        };
    }

    // Core extensions
    check_ext!("Stream", pjrt::StreamExtension);
    check_ext!("Layouts", pjrt::LayoutsExtension);
    check_ext!("FFI", pjrt::FfiExtension);
    check_ext!("RawBuffer", pjrt::RawBufferExtension);
    check_ext!("Profiler", pjrt::ProfilerExtension);
    check_ext!("Callback", pjrt::CallbackExtension);
    check_ext!("MemoryDescriptions", pjrt::MemoryDescriptionsExtension);
    check_ext!("PhaseCompile", pjrt::PhaseCompileExtension);
    check_ext!("ExecutableMetadata", pjrt::ExecutableMetadataExtension);
    check_ext!("HostAllocator", pjrt::HostAllocatorExtension);
    check_ext!("CrossHostTransfers", pjrt::CrossHostTransfersExtension);

    // GPU-specific extensions
    check_ext!("GpuCustomCall", pjrt::GpuExtension);
    check_ext!("Triton", pjrt::TritonExtension);
    check_ext!("CustomPartitioner", pjrt::CustomPartitionerExtension);

    // TPU-specific extensions
    check_ext!("TpuTopology", pjrt::TpuTopologyExtension);
    check_ext!("TpuExecutable", pjrt::TpuExecutableExtension);
    check_ext!("Megascale", pjrt::MegascaleExtension);

    // Reference / testing
    check_ext!("Example", pjrt::ExampleExtension);

    Ok(())
}
