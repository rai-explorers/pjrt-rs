//! Multi-Device Example
//!
//! This example demonstrates working with multiple devices:
//! 1. Enumerating available devices
//! 2. Creating device assignments for multi-device execution
//! 3. Transferring buffers between devices
//! 4. Running computations on specific devices
//!
//! To run this example:
//! ```
//! export PJRT_PLUGIN_PATH=/path/to/pjrt_c_api_multi_device_plugin.so
//! cargo run --example multi_device
//! ```

use pjrt::{self, Client, HostBuffer, Result};

#[tokio::main]
async fn main() -> Result<()> {
    let plugin_path = std::env::var("PJRT_PLUGIN_PATH")
        .expect("PJRT_PLUGIN_PATH environment variable must be set");
    let api = pjrt::plugin(&plugin_path).load()?;

    let client = Client::builder(&api).build()?;

    // Enumerate all available devices
    let devices = client.devices();
    println!("Found {} devices:", devices.len());

    for (i, device) in devices.iter().enumerate() {
        let desc = device.description();
        println!("  Device {}: {} ({})", i, desc.kind(), desc.to_string());

        let hw_id = device.local_hardware_id();
        let hw_id_str = if hw_id >= 0 {
            format!("{}", hw_id)
        } else {
            "undefined".to_string()
        };
        println!("    Local Hardware ID: {}", hw_id_str);
    }

    // Get addressable devices (devices that can be used for computation)
    let addressable_devices = client.addressable_devices();
    println!("{} addressable devices", addressable_devices.len());

    // Create a device assignment for multi-device execution
    if addressable_devices.len() >= 2 {
        let num_replicas = 2;
        let num_partitions = 1;

        let _device_assignment = client.default_device_assignment(num_replicas, num_partitions)?;
        println!("Created device assignment with {} replicas", num_replicas);
        // Note: The actual device assignment details depend on the PJRT implementation

        // Create a buffer on the first device
        let host_buffer = HostBuffer::from_scalar(42.0f32);
        let device_buffer = host_buffer.to_sync(&client).copy()?;

        // Transfer buffer to the second device
        let transferred_buffer = device_buffer
            .to_device(&addressable_devices[1])
            .copy()
            .await?;

        // Transfer back to host
        let result_host = transferred_buffer.to_host(None).await?;
        // For demonstration, just show that we have a result
        println!("Got host buffer with shape: {:?}", result_host.dims());

        // Demonstrate memory stats
        for (i, device) in addressable_devices.iter().take(2).enumerate() {
            if let Ok(stats) = device.memory_stats() {
                println!("Device {} memory stats: {:?}", i, stats);
            }
        }
    } else {
        println!("Not enough addressable devices for multi-device example");
    }

    Ok(())
}
