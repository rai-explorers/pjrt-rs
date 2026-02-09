//! Memory Descriptions Extension Example
//!
//! This example demonstrates the PJRT Memory Descriptions extension, which
//! provides information about the memory types available for each device.
//! This is useful for:
//! - Querying available memory kinds (HBM, DRAM, SRAM, etc.)
//! - Understanding memory hierarchies for AOT compilation
//! - Selecting optimal memory placement strategies
//!
//! To run this example:
//! ```
//! export PJRT_PLUGIN_PATH=/path/to/pjrt_c_api_plugin.so
//! cargo run --example memory_descriptions_extension
//! ```

use pjrt::{self, Client, MemoryDescriptionsExtension, Result};

fn main() -> Result<()> {
    let plugin_path = std::env::var("PJRT_PLUGIN_PATH")
        .expect("PJRT_PLUGIN_PATH environment variable must be set");
    let api = pjrt::plugin(&plugin_path).load()?;
    let client = Client::builder(&api).build()?;

    println!("Memory Descriptions Extension Example");
    println!("=====================================\n");

    let mem_ext = match api.get_extension::<MemoryDescriptionsExtension>() {
        Some(ext) => {
            println!("Memory Descriptions extension: available\n");
            ext
        }
        None => {
            println!("Memory Descriptions extension is not available in this plugin.");
            println!("This extension provides device memory type information.\n");
            println!("The Memory Descriptions extension provides:");
            println!("  - get_memory_descriptions(device_description)");
            println!("      Returns DeviceMemoryDescriptions with:");
            println!("        - descriptions: Vec<MemoryDescription>");
            println!("        - default_memory_index: isize (-1 if none)");
            println!();
            println!("  - MemoryDescription:");
            println!("    - kind() → MemoryKind {{ kind: String, kind_id: i32 }}");
            println!();
            println!("This information is useful for AOT compilation and");
            println!("memory placement strategies.");
            return Ok(());
        }
    };

    // Query memory descriptions for each device description
    let topology = client.topology()?;
    let device_descs = topology.device_descriptions()?;

    println!("Device Memory Descriptions:");
    println!("---------------------------");

    for (i, device_desc) in device_descs.iter().enumerate() {
        match mem_ext.get_memory_descriptions(device_desc) {
            Ok(descriptions) => {
                println!("  Device {} ({})", i, device_desc.kind()?);
                println!(
                    "    Default memory index: {}",
                    if descriptions.default_memory_index < 0 {
                        "none".to_string()
                    } else {
                        descriptions.default_memory_index.to_string()
                    }
                );
                for (j, desc) in descriptions.descriptions.iter().enumerate() {
                    let kind = desc.kind()?;
                    let is_default = j as isize == descriptions.default_memory_index;
                    println!(
                        "    [{}] kind: {} (id: {}){}",
                        j,
                        kind.kind,
                        kind.kind_id,
                        if is_default { " ← default" } else { "" }
                    );
                }
            }
            Err(e) => {
                println!("  Device {}: error — {}", i, e);
            }
        }
        println!();
    }

    Ok(())
}
