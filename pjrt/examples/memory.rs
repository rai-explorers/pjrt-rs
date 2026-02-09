//! Memory Management Example
//!
//! This example demonstrates how to work with PJRT Memory spaces:
//! 1. Querying memory information from devices
//! 2. Understanding memory kinds and IDs
//! 3. Finding which devices can access specific memory
//! 4. Working with memory topology
//!
//! Understanding memory spaces is crucial for:
//! - Optimizing data placement (CPU vs GPU memory)
//! - Multi-GPU memory management
//! - NUMA-aware data placement
//!
//! To run this example:
//! ```
//! export PJRT_PLUGIN_PATH=/path/to/pjrt_c_api_cpu_plugin.so
//! cargo run --example memory
//! ```

use pjrt::{self, Client, Result};

fn main() -> Result<()> {
    let plugin_path = std::env::var("PJRT_PLUGIN_PATH")
        .expect("PJRT_PLUGIN_PATH environment variable must be set");
    let api = pjrt::plugin(&plugin_path).load()?;

    let client = Client::builder(&api).build()?;

    println!("Memory Management Example");
    println!("========================\n");

    // Example 1: List all addressable memories
    demonstrate_list_memories(&client)?;

    // Example 2: Query memory information
    demonstrate_memory_info(&client)?;

    // Example 3: Memory-device relationships
    demonstrate_memory_devices(&client)?;

    // Example 4: Device memory stats
    demonstrate_memory_stats(&client)?;

    Ok(())
}

/// Demonstrates listing all addressable memories
fn demonstrate_list_memories(client: &Client) -> Result<()> {
    println!("1. Listing All Addressable Memories");
    println!("   ---------------------------------");

    let memories = client.addressable_memories()?;

    if memories.is_empty() {
        println!("   No addressable memories found.");
    } else {
        println!("   Found {} addressable memory space(s):\n", memories.len());

        for (i, memory) in memories.iter().enumerate() {
            println!("   Memory {}:", i);
            println!("     - ID: {}", memory.id()?);
            println!("     - Kind: {}", memory.kind()?);
            println!("     - Kind ID: {}", memory.kind_id()?);
            println!("     - String: {}", memory.to_string()?);
            println!("     - Debug: {}", memory.debug_string()?);
            println!();
        }
    }

    Ok(())
}

/// Demonstrates querying detailed memory information
fn demonstrate_memory_info(client: &Client) -> Result<()> {
    println!("2. Memory Information Query");
    println!("   ------------------------");

    let memories = client.addressable_memories()?;

    if let Some(memory) = memories.first() {
        println!("   Detailed information for Memory {}:\n", memory.id()?);

        // Memory kind describes the type of memory
        let kind = memory.kind()?;
        println!("   Memory Kind: {}", kind);

        // Kind ID is a numeric identifier
        let kind_id = memory.kind_id()?;
        println!("   Kind ID: {} (numeric identifier)", kind_id);

        // ToString provides a human-readable description
        let description = memory.to_string()?;
        println!("   Description: {}", description);

        // Debug string provides additional implementation details
        let debug_info = memory.debug_string()?;
        println!("   Debug Info: {}", debug_info);

        println!();

        // Memory kind interpretations
        println!("   Common memory kinds:");
        println!("     - 'default': Default device memory");
        println!("     - 'pinned': Pinned/host memory for faster CPU-GPU transfers");
        println!("     - 'unified': Unified memory accessible from CPU and GPU");
        println!("     - 'device': Device-specific memory (e.g., GPU VRAM)");
        println!();
    } else {
        println!("   No memories available to query.\n");
    }

    Ok(())
}

/// Demonstrates memory-device relationships
fn demonstrate_memory_devices(client: &Client) -> Result<()> {
    println!("3. Memory-Device Relationships");
    println!("   ---------------------------");

    let memories = client.addressable_memories()?;
    let devices = client.addressable_devices()?;

    if memories.is_empty() || devices.is_empty() {
        println!("   Insufficient resources to demonstrate relationships.\n");
        return Ok(());
    }

    println!("   Memory accessibility by devices:\n");

    for memory in &memories {
        println!("   Memory {} ({}):", memory.id()?, memory.kind()?);

        let accessible_devices = memory.addressable_by_devices()?;

        if accessible_devices.is_empty() {
            println!("     - Not directly addressable by any device");
        } else {
            println!(
                "     - Addressable by {} device(s):",
                accessible_devices.len()
            );
            for device in &accessible_devices {
                let desc = device.description()?;
                println!("       * Device {} ({})", desc.id()?, desc.kind()?);
            }
        }
        println!();
    }

    // Cross-device memory access patterns
    if devices.len() > 1 {
        println!("   Multi-device memory access patterns:");
        println!("     - Devices can access memory spaces from other devices");
        println!("     - Cross-device access may be slower than local access");
        println!("     - Use device-specific memory for best performance");
        println!();
    }

    Ok(())
}

/// Demonstrates device memory statistics
fn demonstrate_memory_stats(client: &Client) -> Result<()> {
    println!("4. Device Memory Statistics");
    println!("   ------------------------");

    let devices = client.addressable_devices()?;

    if devices.is_empty() {
        println!("   No devices available.\n");
        return Ok(());
    }

    for (i, device) in devices.iter().enumerate() {
        println!("   Device {}:", i);

        match device.memory_stats() {
            Ok(stats) => {
                // Bytes in use
                println!("     - Bytes in use: {}", stats.bytes_in_use);

                // Bytes limit (check if set)
                if let Some(limit) = stats.bytes_limit {
                    println!("     - Bytes limit: {}", limit);
                } else {
                    println!("     - Bytes limit: unlimited");
                }

                // Peak bytes (check if set)
                if let Some(peak) = stats.peak_bytes_in_use {
                    println!("     - Peak bytes in use: {}", peak);
                }

                // Calculate utilization if limit is set
                if let Some(limit) = stats.bytes_limit {
                    if limit > 0 {
                        let utilization = (stats.bytes_in_use as f64 / limit as f64) * 100.0;
                        println!("     - Current utilization: {:.1}%", utilization);
                    }
                }

                // Calculate peak utilization if both are set
                if let (Some(peak), Some(limit)) = (stats.peak_bytes_in_use, stats.bytes_limit) {
                    if limit > 0 {
                        let peak_util = (peak as f64 / limit as f64) * 100.0;
                        println!("     - Peak utilization: {:.1}%", peak_util);
                    }
                }
            }
            Err(e) => {
                println!("     - Memory statistics not available: {:?}", e);
            }
        }
        println!();
    }

    // Memory management best practices
    println!("   Memory Management Best Practices:");
    println!("     - Monitor memory_stats() to track usage");
    println!("     - Use device-specific memory for computation");
    println!("     - Minimize cross-device memory access");
    println!("     - Free buffers promptly when no longer needed");
    println!("     - Use Buffer::to_device_sync() for explicit transfers");
    println!();

    Ok(())
}
