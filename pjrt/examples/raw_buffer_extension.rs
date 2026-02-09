//! Raw Buffer Extension Example
//!
//! This example demonstrates the PJRT Raw Buffer extension (experimental),
//! which provides direct memory access to device buffers. This is useful for:
//! - Zero-copy integration with external frameworks
//! - Direct host-to-device and device-to-host memory operations
//! - Accessing device memory pointers for interop
//!
//! **WARNING**: This extension is experimental and may change.
//!
//! To run this example:
//! ```
//! export PJRT_PLUGIN_PATH=/path/to/pjrt_c_api_plugin.so
//! cargo run --example raw_buffer_extension
//! ```

use pjrt::{self, Client, HostBuffer, RawBufferExtension, Result};

fn main() -> Result<()> {
    let plugin_path = std::env::var("PJRT_PLUGIN_PATH")
        .expect("PJRT_PLUGIN_PATH environment variable must be set");
    let api = pjrt::plugin(&plugin_path).load()?;
    let client = Client::builder(&api).build()?;

    println!("Raw Buffer Extension Example");
    println!("============================\n");

    let raw_ext = match api.get_extension::<RawBufferExtension>() {
        Some(ext) => {
            println!("Raw Buffer extension: available (experimental)\n");
            ext
        }
        None => {
            println!("Raw Buffer extension is not available in this plugin.");
            println!("This extension provides direct memory access to device buffers.\n");
            println!("The Raw Buffer extension provides:");
            println!("  - create_raw_alias(buffer)");
            println!("      Creates a raw alias of a PJRT buffer for direct access.");
            println!("      The alias shares the underlying storage.\n");
            println!("  - RawBuffer methods:");
            println!("    - get_host_pointer()           — get host-visible base pointer (unsafe)");
            println!("    - on_device_size()             — size in bytes");
            println!("    - memory_space()               — memory space of the buffer");
            println!("    - copy_raw_host_to_device(src, offset)  — write bytes to device");
            println!("    - copy_raw_device_to_host(dst, offset)  — read bytes from device");
            println!();
            println!("Usage pattern:");
            println!("  let raw_ext = api.get_extension::<RawBufferExtension>().unwrap();");
            println!("  let raw = raw_ext.create_raw_alias(&buffer)?;");
            println!("  let size = raw.on_device_size()?;");
            println!("  let mut data = vec![0u8; size];");
            println!("  let event = raw.copy_raw_device_to_host(&mut data, 0)?;");
            println!("  event.wait()?;");
            return Ok(());
        }
    };

    // When available, demonstrate raw buffer operations
    let device = &client.addressable_devices()?[0];

    // 1. Create a buffer on device
    println!("1. Create Device Buffer");
    println!("   --------------------");
    let data = vec![1.0f32, 2.0, 3.0, 4.0, 5.0, 6.0];
    let host_buf = HostBuffer::from_data(data, Some(vec![2, 3]), None);
    let device_buf = host_buf.to_sync(device).copy()?;
    println!(
        "   Created f32[2,3] buffer on device {}",
        device.description()?.id()?
    );

    // 2. Create raw alias
    println!("\n2. Raw Buffer Alias");
    println!("   -----------------");
    let raw = raw_ext.create_raw_alias(&device_buf)?;
    let size = raw.on_device_size()?;
    let memory = raw.memory_space()?;
    println!("   On-device size: {} bytes", size);
    println!("   Memory space: {} (id: {})", memory.kind()?, memory.id()?);

    // 3. Get host pointer (if host-visible)
    println!("\n3. Host Pointer Access");
    println!("   --------------------");
    match unsafe { raw.get_host_pointer() } {
        Ok(ptr) => println!("   Host pointer: {:?}", ptr),
        Err(e) => println!("   Not host-visible: {}", e),
    }

    // 4. Raw device-to-host copy
    println!("\n4. Raw Device→Host Copy");
    println!("   ----------------------");
    let mut output = vec![0.0f32; 6];
    let event = unsafe { raw.copy_raw_device_to_host(&mut output, 0)? };
    event.wait()?;
    println!("   Copied: {:?}", output);

    // 5. Raw host-to-device copy (modify and write back)
    println!("\n5. Raw Host→Device Copy");
    println!("   ----------------------");
    let modified = [10.0f32, 20.0, 30.0, 40.0, 50.0, 60.0];
    let event = unsafe { raw.copy_raw_host_to_device(&modified, 0)? };
    event.wait()?;
    println!("   Wrote: {:?}", modified);

    // Verify by reading back
    let mut verify = vec![0.0f32; 6];
    let event = unsafe { raw.copy_raw_device_to_host(&mut verify, 0)? };
    event.wait()?;
    println!("   Verified: {:?}", verify);

    Ok(())
}
