//! Device Assignment Example
//!
//! This example demonstrates the `DeviceAssignment` API for mapping logical
//! computation replicas and partitions to physical devices:
//!
//! 1. Creating device assignments manually and from client defaults
//! 2. Looking up logical IDs (replica, partition) for a given device
//! 3. Building a lookup map for fast device → logical ID resolution
//! 4. Error handling for mismatched dimensions and missing devices
//! 5. Using device assignments with compilation
//!
//! ## Concepts
//!
//! - **Replicas**: Independent copies of the same computation on different devices.
//!   Each replica processes different data (data parallelism).
//! - **Partitions**: A single computation split across multiple devices, each handling
//!   a slice of the model (model parallelism / SPMD).
//! - **`DeviceAssignment`**: A 2D grid mapping (replica, partition) → global device ID.
//!
//! To run this example:
//! ```
//! export PJRT_PLUGIN_PATH=/path/to/pjrt_c_api_cpu_plugin.so
//! cargo run --example device_assignment
//! ```

use pjrt::ProgramFormat::MLIR;
use pjrt::{
    self, Client, CompileOptions, DeviceAssignment, ExecutableBuildOptions, HostBuffer, Result,
};

const CODE: &[u8] = include_bytes!("prog_f32.mlir");

fn main() -> Result<()> {
    let plugin_path = std::env::var("PJRT_PLUGIN_PATH")
        .expect("PJRT_PLUGIN_PATH environment variable must be set");
    let api = pjrt::plugin(&plugin_path).load()?;
    let client = Client::builder(&api).build()?;

    println!("Device Assignment Example");
    println!("=========================\n");

    demonstrate_manual_assignment()?;
    demonstrate_client_default_assignment(&client)?;
    demonstrate_lookup_map()?;
    demonstrate_error_handling()?;
    demonstrate_compile_with_assignment(&client)?;

    Ok(())
}

/// Creates device assignments manually and inspects them.
fn demonstrate_manual_assignment() -> Result<()> {
    println!("1. Manual Device Assignment");
    println!("   -------------------------");

    // A simple 2-replica, 1-partition assignment:
    //   Replica 0 → Device 0
    //   Replica 1 → Device 1
    let da = DeviceAssignment::new(2, 1, vec![0, 1])?;
    println!("   2 replicas x 1 partition:");
    println!("     replicas  = {}", da.num_replicas());
    println!("     partitions = {}", da.num_partitions());

    // Look up the logical ID for device 1
    let logical = da.lookup_logical_id(1)?;
    println!(
        "     Device 1 → replica {}, partition {}",
        logical.replica_id, logical.partition_id
    );

    // A 2-replica, 2-partition assignment (4 devices):
    //   Replica 0: [Device 0, Device 1]
    //   Replica 1: [Device 2, Device 3]
    let da = DeviceAssignment::new(2, 2, vec![0, 1, 2, 3])?;
    println!("\n   2 replicas x 2 partitions:");

    for device_id in 0..4 {
        let logical = da.lookup_logical_id(device_id)?;
        println!(
            "     Device {} → replica {}, partition {}",
            device_id, logical.replica_id, logical.partition_id
        );
    }
    println!();

    Ok(())
}

/// Demonstrates using the client to create a default device assignment
/// based on the available devices.
fn demonstrate_client_default_assignment(client: &Client) -> Result<()> {
    println!("2. Client Default Device Assignment");
    println!("   ---------------------------------");

    let devices = client.addressable_devices();
    let num_devices = devices.len();
    println!("   Addressable devices: {}", num_devices);

    // Request as many replicas as we have devices, 1 partition each
    let num_replicas = num_devices;
    let num_partitions = 1;

    let da = client.default_device_assignment(num_replicas, num_partitions)?;
    println!(
        "   Default assignment: {} replicas x {} partition",
        da.num_replicas(),
        da.num_partitions()
    );

    // Show the device-to-replica mapping
    for device in &devices {
        let global_id = device.description().id();
        match da.lookup_logical_id(global_id) {
            Ok(logical) => println!(
                "     Device {} (kind: {}) → replica {}",
                global_id,
                device.description().kind(),
                logical.replica_id
            ),
            Err(_) => println!("     Device {} not in assignment", global_id),
        }
    }
    println!();

    Ok(())
}

/// Demonstrates the bulk lookup map for efficient device-to-logical-ID resolution.
fn demonstrate_lookup_map() -> Result<()> {
    println!("3. Lookup Map (Bulk Resolution)");
    println!("   -----------------------------");

    let da = DeviceAssignment::new(3, 2, vec![10, 11, 20, 21, 30, 31])?;
    let map = da.get_lookup_map();

    println!("   3 replicas x 2 partitions (device IDs 10-31):");
    println!("   Map entries: {}", map.len());

    // Sort by device ID for deterministic output
    let mut entries: Vec<_> = map.iter().collect();
    entries.sort_by_key(|(id, _)| **id);

    for (device_id, logical) in &entries {
        println!(
            "     Device {} → replica {}, partition {}",
            device_id, logical.replica_id, logical.partition_id
        );
    }

    // Fast lookup example
    if let Some(logical) = map.get(&20) {
        println!(
            "\n   Fast lookup: device 20 → replica {}, partition {}",
            logical.replica_id, logical.partition_id
        );
    }
    println!();

    Ok(())
}

/// Demonstrates error handling for invalid device assignments.
fn demonstrate_error_handling() -> Result<()> {
    println!("4. Error Handling");
    println!("   ---------------");

    // Wrong number of device IDs
    let result = DeviceAssignment::new(2, 2, vec![0, 1, 2]);
    match &result {
        Err(e) => println!("   ✓ new(2, 2, [0,1,2]) → Error: {}", e),
        Ok(_) => println!("   ✗ Expected error for mismatched length"),
    }

    // Looking up a device that isn't in the assignment
    let da = DeviceAssignment::new(1, 1, vec![42])?;
    let result = da.lookup_logical_id(99);
    match &result {
        Err(e) => println!("   ✓ lookup_logical_id(99) → Error: {}", e),
        Ok(_) => println!("   ✗ Expected error for missing device"),
    }

    // Zero-size assignment is valid
    let da = DeviceAssignment::new(0, 0, vec![])?;
    println!(
        "   ✓ new(0, 0, []) → OK ({} replicas x {} partitions)",
        da.num_replicas(),
        da.num_partitions()
    );
    println!();

    Ok(())
}

/// Demonstrates using a device assignment with compilation via compile options.
fn demonstrate_compile_with_assignment(client: &Client) -> Result<()> {
    println!("5. Compilation with Device Assignment");
    println!("   ------------------------------------");

    let devices = client.addressable_devices();

    // Get the default assignment for 1 replica, 1 partition
    let da = client.default_device_assignment(1, 1)?;
    println!(
        "   Using assignment: {} replica x {} partition",
        da.num_replicas(),
        da.num_partitions()
    );

    // Build compile options with the default assignment parameters
    let build_options = ExecutableBuildOptions::new()
        .num_replicas(da.num_replicas() as i64)
        .num_partitions(da.num_partitions() as i64);

    let compile_options = CompileOptions::new().executable_build_options(build_options);

    let program = pjrt::Program::new(MLIR, CODE);
    let loaded_exe = client.compile(&program, compile_options)?;

    // Show executable info
    let exe = loaded_exe.executable();
    println!("   Executable: {}", exe.name());
    println!("   Replicas:   {}", exe.num_replicas());
    println!("   Partitions: {}", exe.num_partitions());

    // Execute on the first device
    let input = HostBuffer::from_scalar(42.0f32);
    let device_buf = input.to_sync(&devices[0]).copy()?;

    let result = loaded_exe.execution(device_buf).run_sync()?;
    let output: HostBuffer = result[0][0].to_host_sync(None)?;
    println!("   Input:  3.14");
    println!("   Output: {:?}\n", output);

    Ok(())
}
