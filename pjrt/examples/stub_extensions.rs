//! Stub Extensions Example
//!
//! This example demonstrates the PJRT extensions that are currently stubs —
//! they expose the raw extension pointer for direct C API access but do not
//! yet have higher-level Rust wrappers for their operations.
//!
//! These extensions exist in the PJRT specification and will be implemented
//! as plugins add support for them:
//!
//! - **CrossHostTransfers**: Multi-host buffer transfer protocols
//! - **ExecutableMetadata**: Metadata about compiled executables
//! - **HostAllocator**: Custom host-side memory allocation
//! - **TpuTopology**: TPU-specific topology information
//! - **TpuExecutable**: TPU-specific executable features
//! - **Megascale**: Large-scale distributed training support
//! - **Example**: Reference extension for the PJRT extension architecture
//!
//! To run this example:
//! ```
//! export PJRT_PLUGIN_PATH=/path/to/pjrt_c_api_plugin.so
//! cargo run --example stub_extensions
//! ```

use pjrt::{
    self, Client, CrossHostTransfersExtension, ExampleExtension, ExecutableMetadataExtension,
    HostAllocatorExtension, MegascaleExtension, Result, TpuExecutableExtension,
    TpuTopologyExtension,
};

fn main() -> Result<()> {
    let plugin_path = std::env::var("PJRT_PLUGIN_PATH")
        .expect("PJRT_PLUGIN_PATH environment variable must be set");
    let api = pjrt::plugin(&plugin_path).load()?;
    let _client = Client::builder(&api).build()?;

    println!("Stub Extensions Discovery Example");
    println!("==================================\n");
    println!(
        "Plugin: {}\n",
        std::path::Path::new(&plugin_path)
            .file_name()
            .unwrap()
            .to_string_lossy()
    );

    // Check each stub extension and report availability
    let extensions: Vec<(&str, bool, &str)> = vec![
        (
            "CrossHostTransfers",
            api.has_extension::<CrossHostTransfersExtension>(),
            "Multi-host buffer transfer protocols for distributed setups",
        ),
        (
            "ExecutableMetadata",
            api.has_extension::<ExecutableMetadataExtension>(),
            "Metadata about compiled executables (fingerprints, cost analysis)",
        ),
        (
            "HostAllocator",
            api.has_extension::<HostAllocatorExtension>(),
            "Custom host-side memory allocation strategies",
        ),
        (
            "TpuTopology",
            api.has_extension::<TpuTopologyExtension>(),
            "TPU-specific topology information and chip coordinates",
        ),
        (
            "TpuExecutable",
            api.has_extension::<TpuExecutableExtension>(),
            "TPU-specific executable features and optimizations",
        ),
        (
            "Megascale",
            api.has_extension::<MegascaleExtension>(),
            "Large-scale distributed training support",
        ),
        (
            "Example",
            api.has_extension::<ExampleExtension>(),
            "Reference implementation for the PJRT extension architecture",
        ),
    ];

    let available_count = extensions.iter().filter(|(_, avail, _)| *avail).count();
    println!(
        "Found {}/{} stub extensions available:\n",
        available_count,
        extensions.len()
    );

    for (name, available, description) in &extensions {
        let status = if *available { "✓" } else { "✗" };
        println!("  {} {}", status, name);
        println!("    {}", description);
    }

    // Demonstrate the HostAllocator's is_experimental() method if available
    if let Some(host_alloc) = api.get_extension::<HostAllocatorExtension>() {
        println!("\nHostAllocator details:");
        println!("  is_experimental: {}", host_alloc.is_experimental());
    }

    // Demonstrate the Example extension's type_id() method if available
    if let Some(example) = api.get_extension::<ExampleExtension>() {
        println!("\nExample extension details:");
        println!("  type_id: {:?}", example.type_id());
    }

    println!("\nNote: These extensions expose raw_ptr() for direct C API access.");
    println!("Higher-level Rust wrappers will be added as plugin support matures.");

    Ok(())
}
