//! Async Transfer Manager Example
//!
//! This example demonstrates how to use the AsyncHostToDeviceTransferManager:
//! 1. Single-shot raw byte transfer via transfer_all_sync
//! 2. Chunked transfers with manual progress tracking
//!
//! Async transfers are useful when working with datasets larger than device memory
//! or when you want to overlap data transfers with computation.
//!
//! To run this example:
//! ```
//! export PJRT_PLUGIN_PATH=/path/to/pjrt_c_api_cpu_plugin.so
//! cargo run --example async_transfer
//! ```

use pjrt::{self, BufferShape, Client, Device, PrimitiveType, Result};

fn main() -> Result<()> {
    let plugin_path = std::env::var("PJRT_PLUGIN_PATH")
        .expect("PJRT_PLUGIN_PATH environment variable must be set");
    let api = pjrt::plugin(&plugin_path).load()?;
    let client = Client::builder(&api).build()?;

    println!("Async Transfer Manager Example");
    println!("================================");

    let device = client
        .addressable_devices()?
        .into_iter()
        .next()
        .expect("No addressable device");

    single_shot_transfer(&client, &device)?;
    chunked_transfer_with_progress(&client, &device)?;

    println!("\nAll examples completed successfully!");
    Ok(())
}

/// Demonstrates a single-shot transfer using transfer_all_sync.
fn single_shot_transfer(client: &Client, device: &Device) -> Result<()> {
    println!("\n1. Single-Shot Raw Transfer");
    println!("----------------------------");

    let data: Vec<f32> = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0];
    let data_bytes: &[u8] =
        unsafe { std::slice::from_raw_parts(data.as_ptr() as *const u8, data.len() * 4) };

    let shapes = vec![BufferShape::new(vec![2, 3], PrimitiveType::F32)];
    let memory = device.default_memory()?;
    let manager = client.create_buffers_for_async_host_to_device(&shapes, &memory)?;

    manager.transfer_all_sync(0, data_bytes)?;
    let buffer = manager.retrieve_buffer(0)?;

    println!(
        "  Transferred {}-element {:?} tensor to device",
        data.len(),
        buffer.primitive_type()?
    );
    println!("  On-device size: {} bytes", buffer.on_device_size()?);

    let host = buffer.to_host_sync(None)?;
    let result = host.read_f32()?;
    println!("  Readback: {:?}", result);
    assert_eq!(result, data);

    Ok(())
}

/// Demonstrates chunked transfers with manual progress tracking via the transfer manager.
fn chunked_transfer_with_progress(client: &Client, device: &Device) -> Result<()> {
    println!("\n2. Chunked Transfer with Progress");
    println!("----------------------------------");

    let num_elements: usize = 4096;
    let data: Vec<f32> = (0..num_elements).map(|i| i as f32).collect();
    let data_bytes: &[u8] =
        unsafe { std::slice::from_raw_parts(data.as_ptr() as *const u8, data.len() * 4) };

    let shapes = vec![BufferShape::new(
        vec![num_elements as i64],
        PrimitiveType::F32,
    )];
    let memory = device.default_memory()?;
    let manager = client.create_buffers_for_async_host_to_device(&shapes, &memory)?;

    println!(
        "  Transfer manager: {} buffer(s), {} bytes each",
        manager.buffer_count()?,
        manager.buffer_size(0)?
    );

    let chunk_size = 4096; // 4 KB per chunk
    let total = data_bytes.len();
    let mut transferred = 0;

    for chunk in data_bytes.chunks(chunk_size) {
        let is_last = transferred + chunk.len() >= total;
        let event = manager
            .transfer_data(0)
            .data(chunk)
            .offset(transferred as i64)
            .is_last_transfer(is_last)
            .transfer()?;
        event.wait()?;

        transferred += chunk.len();
        let pct = 100.0 * transferred as f64 / total as f64;
        println!("  Progress: {transferred}/{total} bytes ({pct:.0}%)");
    }

    let buffer = manager.retrieve_buffer(0)?;
    println!(
        "  Retrieved buffer: {} bytes on device",
        buffer.on_device_size()?
    );

    let host = buffer.to_host_sync(None)?;
    let result = host.read_f32()?;
    assert_eq!(result[0], 0.0);
    assert_eq!(result[100], 100.0);
    assert_eq!(result[num_elements - 1], (num_elements - 1) as f32);
    println!(
        "  Verification passed (first={}, last={})",
        result[0],
        result[num_elements - 1]
    );

    Ok(())
}
