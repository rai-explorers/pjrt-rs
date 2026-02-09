//! Multi-Device Example
//!
//! This example demonstrates working with multiple devices:
//! 1. Enumerating and inspecting available devices
//! 2. Querying device properties (description, kind, memories)
//! 3. Exploring the topology description
//! 4. Transferring buffers between devices (sync and async)
//! 5. Running the same computation on different devices
//! 6. Device memory stats
//!
//! To run this example:
//! ```
//! export PJRT_PLUGIN_PATH=/path/to/pjrt_c_api_cpu_plugin.so
//! cargo run --example multi_device
//! ```

use pjrt::ProgramFormat::MLIR;
use pjrt::{self, Client, CompileOptions, ExecutableBuildOptions, HostBuffer, Result};

const CODE: &[u8] = include_bytes!("prog_f32.mlir");

#[tokio::main]
async fn main() -> Result<()> {
    let plugin_path = std::env::var("PJRT_PLUGIN_PATH")
        .expect("PJRT_PLUGIN_PATH environment variable must be set");
    let api = pjrt::plugin(&plugin_path).load()?;
    let client = Client::builder(&api).build()?;

    println!("Multi-Device Example");
    println!("====================\n");

    demonstrate_device_enumeration(&client)?;
    demonstrate_topology(&client)?;
    demonstrate_per_device_execution(&client)?;
    demonstrate_device_transfers(&client).await?;
    demonstrate_memory_stats(&client)?;

    Ok(())
}

/// Enumerates all devices and shows their properties.
fn demonstrate_device_enumeration(client: &Client) -> Result<()> {
    println!("1. Device Enumeration");
    println!("   -------------------");

    let all_devices = client.devices()?;
    let addressable = client.addressable_devices()?;

    println!(
        "   Platform: {} (version: {})",
        client.platform_name()?,
        client.platform_version()?
    );
    println!("   Process index: {}", client.process_index()?);
    println!(
        "   Total devices: {}, addressable: {}\n",
        all_devices.len(),
        addressable.len()
    );

    for (i, device) in all_devices.iter().enumerate() {
        let desc = device.description()?;

        println!("   Device {}:", i);
        println!("     Global ID:   {}", desc.id()?);
        println!("     Kind:        {}", desc.kind()?);
        println!("     Debug:       {}", desc.debug_string()?);
        println!("     Addressable: {}", device.is_addressable()?);

        let hw_id = device.local_hardware_id()?;
        if hw_id >= 0 {
            println!("     HW ID:       {}", hw_id);
        }

        // Show device attributes
        if let Ok(attrs) = desc.attributes() {
            let attrs = attrs.into_inner();
            if !attrs.is_empty() {
                println!("     Attributes:");
                for (key, value) in &attrs {
                    println!("       {}: {:?}", key, value);
                }
            }
        }

        // Show memories attached to this device
        let memories = device.addressable_memories()?;
        if !memories.is_empty() {
            println!("     Memories ({}):", memories.len());
            for mem in &memories {
                println!("       - {} (id: {})", mem.kind()?, mem.id()?);
            }
        }

        let default_mem = device.default_memory()?;
        println!(
            "     Default memory: {} (id: {})",
            default_mem.kind()?,
            default_mem.id()?
        );
        println!();
    }

    Ok(())
}

/// Explores the topology description.
fn demonstrate_topology(client: &Client) -> Result<()> {
    println!("2. Topology Description");
    println!("   ---------------------");

    let topology = client.topology()?;

    println!(
        "   Platform:  {} (version: {})",
        topology.platform_name()?,
        topology.platform_version()?
    );

    let device_descs = topology.device_descriptions()?;
    println!("   Devices in topology: {}", device_descs.len());

    for (i, desc) in device_descs.iter().enumerate() {
        println!(
            "     [{}] Global ID: {}, kind: {}, process: {}",
            i,
            desc.id()?,
            desc.kind()?,
            desc.process_index()?
        );
    }

    // Show topology attributes
    if let Ok(attrs) = topology.attributes() {
        let attrs = attrs.into_inner();
        if !attrs.is_empty() {
            println!("   Topology attributes:");
            for (key, value) in &attrs {
                println!("     {}: {:?}", key, value);
            }
        }
    }

    // Demonstrate serialization
    let serialized = topology.serialize()?;
    println!(
        "   Serialized topology: {} bytes\n",
        serialized.bytes().len()
    );

    Ok(())
}

/// Runs the same computation on each addressable device independently.
fn demonstrate_per_device_execution(client: &Client) -> Result<()> {
    println!("3. Per-Device Execution");
    println!("   ---------------------");

    let devices = client.addressable_devices()?;
    let num_devices = devices.len();
    println!("   Running on {} addressable device(s):\n", num_devices);

    // Compile with one replica per device so the runtime maps each replica
    // to a different addressable device.
    let program = pjrt::Program::new(MLIR, CODE);
    let build_options = ExecutableBuildOptions::new()
        .num_replicas(num_devices as i64)
        .num_partitions(1);
    let compile_options = CompileOptions::new().executable_build_options(build_options);
    let loaded_exe = client.compile(&program, compile_options)?;

    // Create one input buffer per device, each with a different value.
    // The outer Vec corresponds to devices/replicas, inner Vec to arguments.
    let mut per_device_inputs: Vec<Vec<pjrt::Buffer>> = Vec::with_capacity(num_devices);
    let mut input_vals = Vec::with_capacity(num_devices);
    for (i, device) in devices.iter().enumerate() {
        let input_val = (i + 1) as f32 * 10.0;
        input_vals.push(input_val);
        let host_buf = HostBuffer::from_scalar(input_val);
        let device_buf = host_buf.to_sync(device).copy()?;
        per_device_inputs.push(vec![device_buf]);
    }

    // Execute across all devices simultaneously
    let results = loaded_exe.execution(per_device_inputs).run_sync()?;

    for (i, (device, outputs)) in devices.iter().zip(results.iter()).enumerate() {
        let output: HostBuffer = outputs[0].to_host_sync(None)?;
        println!(
            "     Device {} (kind: {}, id: {}): input={}, output={:?}",
            i,
            device.description()?.kind()?,
            device.description()?.id()?,
            input_vals[i],
            output,
        );
    }
    println!();

    Ok(())
}

/// Demonstrates transferring buffers between devices.
async fn demonstrate_device_transfers(client: &Client) -> Result<()> {
    println!("4. Device-to-Device Transfers");
    println!("   ---------------------------");

    let devices = client.addressable_devices()?;

    if devices.len() < 2 {
        println!("   Skipped (requires ≥2 addressable devices)\n");
        return Ok(());
    }

    let src_device = &devices[0];
    let dst_device = &devices[1];

    println!(
        "   Source: device {} (kind: {})",
        src_device.description()?.id()?,
        src_device.description()?.kind()?,
    );
    println!(
        "   Dest:   device {} (kind: {})\n",
        dst_device.description()?.id()?,
        dst_device.description()?.kind()?,
    );

    let data = vec![1.0f32, 2.0, 3.0, 4.0, 5.0, 6.0];
    let host_buf = HostBuffer::from_data(data, Some(vec![2, 3]), None);

    // Host → Device 0
    let buf_on_src = host_buf.to_sync(src_device).copy()?;
    println!(
        "   Host → Device {}: shape {:?}",
        src_device.description()?.id()?,
        buf_on_src.dims()?
    );

    // Device 0 → Device 1 (async)
    let buf_on_dst = buf_on_src.to_device(dst_device).await?;
    println!(
        "   Device {} → Device {}: shape {:?}",
        src_device.description()?.id()?,
        dst_device.description()?.id()?,
        buf_on_dst.dims()?,
    );

    // Device 1 → Host
    let result: HostBuffer = buf_on_dst.to_host(None).await?;
    println!(
        "   Device {} → Host: {:?}",
        dst_device.description()?.id()?,
        result
    );

    // Also demo sync transfers
    let buf_on_src = host_buf.to_sync(src_device).copy()?;
    let buf_on_dst_sync = buf_on_src.to_device_sync(dst_device).copy()?;
    let result_sync: HostBuffer = buf_on_dst_sync.to_host_sync(None)?;
    println!("   (sync version): {:?}\n", result_sync);

    Ok(())
}

/// Demonstrates querying device memory statistics.
fn demonstrate_memory_stats(client: &Client) -> Result<()> {
    println!("5. Device Memory Stats");
    println!("   --------------------");

    let devices = client.addressable_devices()?;

    for (i, device) in devices.iter().enumerate() {
        match device.memory_stats() {
            Ok(stats) => {
                println!("   Device {}:", i);
                println!("     bytes_in_use: {}", stats.bytes_in_use);

                if let Some(limit) = stats.bytes_limit {
                    println!("     bytes_limit:  {}", limit);
                }
                if let Some(peak) = stats.peak_bytes_in_use {
                    println!("     peak_bytes:   {}", peak);
                }
                if let Some(reserved) = stats.bytes_reserved {
                    println!("     reserved:     {}", reserved);
                }
                if let Some(peak_reserved) = stats.peak_bytes_reserved {
                    println!("     peak_reserved: {}", peak_reserved);
                }

                println!();
            }
            Err(e) => {
                println!("   Device {}: Memory stats unavailable ({})\n", i, e);
            }
        }
    }

    Ok(())
}
