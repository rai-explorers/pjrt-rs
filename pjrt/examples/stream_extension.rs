//! Stream Extension Example
//!
//! This example demonstrates the PJRT Stream extension, which provides
//! platform-specific stream handles for synchronizing with external libraries.
//! This is useful for:
//! - Integrating with CUDA streams for GPU interop
//! - Synchronizing external buffer readiness with PJRT operations
//! - Advanced GPU stream management
//!
//! This extension is typically available only on GPU backends (CUDA/ROCm).
//!
//! To run this example:
//! ```
//! export PJRT_PLUGIN_PATH=/path/to/pjrt_c_api_gpu_plugin.so
//! cargo run --example stream_extension
//! ```

use pjrt::{self, Client, HostBuffer, Result, StreamExtension};

fn main() -> Result<()> {
    let plugin_path = std::env::var("PJRT_PLUGIN_PATH")
        .expect("PJRT_PLUGIN_PATH environment variable must be set");
    let api = pjrt::plugin(&plugin_path).load()?;
    let client = Client::builder(&api).build()?;

    println!("Stream Extension Example");
    println!("========================\n");

    let stream_ext = match api.get_extension::<StreamExtension>() {
        Some(ext) => {
            println!("Stream extension: available\n");
            ext
        }
        None => {
            println!("Stream extension is not available in this plugin.");
            println!("This extension is typically available on GPU backends (CUDA/ROCm).\n");
            println!("The Stream extension provides:");
            println!("  - stream_for_external_ready_events(device)");
            println!("      Returns a DeviceStream handle (e.g., CUDA stream) for a device.");
            println!("      External libraries can signal buffer readiness on this stream.");
            println!();
            println!("  - DeviceStream::wait_until_buffer_ready(buffer)");
            println!("      Blocks until the given buffer is ready on the stream.");
            println!("      This synchronizes PJRT buffers with external GPU operations.");
            println!();
            println!("Usage pattern (GPU):");
            println!("  let stream_ext = api.stream_extension().unwrap();");
            println!("  let device = &client.addressable_devices()?[0];");
            println!("  let stream = stream_ext.stream_for_external_ready_events(device)?;");
            println!("  // ... perform external GPU work ...");
            println!("  stream.wait_until_buffer_ready(&buffer)?;");
            return Ok(());
        }
    };

    // When available, demonstrate stream usage
    let devices = client.addressable_devices()?;
    let device = &devices[0];

    // 1. Get a stream for the device
    println!("1. Device Stream");
    println!("   -------------");
    let stream = stream_ext.stream_for_external_ready_events(device)?;
    println!(
        "   Got stream for device {} ({})",
        device.description()?.id()?,
        device.description()?.kind()?
    );

    // 2. Create a buffer and wait for it on the stream
    println!("\n2. Buffer Synchronization");
    println!("   ----------------------");
    let host_buf = HostBuffer::from_data(vec![1.0f32, 2.0, 3.0, 4.0], Some(vec![2, 2]), None);
    let device_buf = host_buf.to_sync(device).copy()?;
    stream.wait_until_buffer_ready(&device_buf)?;
    println!("   Buffer is ready on the stream");

    Ok(())
}
