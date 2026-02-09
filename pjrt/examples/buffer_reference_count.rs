//! Buffer External Reference Counting Example
//!
//! This example demonstrates how to use the buffer external reference counting APIs
//! for interoperability with external frameworks:
//! 1. Getting raw pointers to buffer data
//! 2. Managing external reference counts
//! 3. Safely accessing buffer memory
//!
//! WARNING: This example uses unsafe operations that can cause memory corruption
//! if used incorrectly. Only use these APIs when you need to interoperate with
//! external frameworks like NumPy, PyTorch, or other GPU libraries.
//!
//! To run this example:
//! ```
//! export PJRT_PLUGIN_PATH=/path/to/pjrt_c_api_cpu_plugin.so
//! cargo run --example buffer_reference_count
//! ```

use pjrt::{self, Client, HostBuffer, Result};

fn main() -> Result<()> {
    let plugin_path = std::env::var("PJRT_PLUGIN_PATH")
        .expect("PJRT_PLUGIN_PATH environment variable must be set");
    let api = pjrt::plugin(&plugin_path).load()?;

    let client = Client::builder(&api).build()?;

    // Create a buffer with some data
    let host_buffer = HostBuffer::from_data(vec![1.0f32, 2.0, 3.0, 4.0], Some(vec![2, 2]), None);
    let device_buffer = host_buffer.to_sync(&client).copy()?;

    println!("Created device buffer: {:?}", device_buffer);

    // Example: Interoperating with an external framework
    // This is a common pattern when using PJRT with other ML frameworks

    // SAFETY: This block demonstrates unsafe operations properly
    unsafe {
        // 1. Increase external reference count before getting pointers
        // This prevents the buffer from being freed while external code uses it
        device_buffer.increase_external_ref_count()?;
        println!("Increased external reference count");

        // 2. Get the unsafe pointer to the buffer data
        // This can be passed to external frameworks
        let buffer_ptr = device_buffer.unsafe_pointer()?;
        println!("Buffer pointer: {:p}", buffer_ptr as *const ());
        assert!(buffer_ptr != 0, "Buffer pointer should not be null");

        // 3. Get the opaque device memory pointer
        // This might be needed for some external frameworks
        let device_mem_ptr = device_buffer.opaque_device_memory_pointer()?;
        println!("Device memory pointer: {:p}", device_mem_ptr);
        assert!(
            !device_mem_ptr.is_null(),
            "Device memory pointer should not be null"
        );

        // Simulate external framework using the buffer
        println!("Simulating external framework operations...");
        // Read data through the raw pointer (like an external framework would)
        let num_elements = 4;
        let slice = std::slice::from_raw_parts(device_mem_ptr as *const f32, num_elements);
        println!("  Data read via raw pointer: {:?}", slice);

        // 4. When the external framework is done, decrease the reference count
        // This allows PJRT to free the buffer when all references are released
        device_buffer.decrease_external_ref_count()?;
        println!("Decreased external reference count");
    }

    // The buffer can still be used normally after external reference counting
    let result = device_buffer.to_host_sync(None)?;
    let data = result.read_f32()?;
    println!("Buffer data after external operations: {:?}", data);

    println!("Note: In a real application, ensure that the external framework");
    println!("doesn't use the buffer after you decrease the reference count.");

    Ok(())
}

/// Reads the contents of a device buffer through its raw pointer.
///
/// # Safety
///
/// Caller must ensure the buffer has an active external reference
/// (via `increase_external_ref_count`) for the duration of this call.
#[allow(dead_code)]
unsafe fn read_buffer_via_raw_pointer(ptr: *const std::ffi::c_void, len: usize) -> Vec<f32> {
    std::slice::from_raw_parts(ptr as *const f32, len).to_vec()
}
