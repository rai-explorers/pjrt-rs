//! Layouts Extension Example
//!
//! This example demonstrates the PJRT Layouts extension, which provides
//! access to buffer memory layouts, serialization, and default layout queries.
//! This is useful for:
//! - Inspecting how data is laid out in device memory
//! - Querying default layouts for a given type/shape combination
//! - Serializing and deserializing layouts for caching or transfer
//!
//! To run this example:
//! ```
//! export PJRT_PLUGIN_PATH=/path/to/pjrt_c_api_plugin.so
//! cargo run --example layouts_extension
//! ```

use pjrt::ProgramFormat::MLIR;
use pjrt::{self, Client, HostBuffer, LayoutsExtension, PrimitiveType, Result};

const CODE: &[u8] = include_bytes!("prog_f32.mlir");

fn main() -> Result<()> {
    let plugin_path = std::env::var("PJRT_PLUGIN_PATH")
        .expect("PJRT_PLUGIN_PATH environment variable must be set");
    let api = pjrt::plugin(&plugin_path).load()?;
    let client = Client::builder(&api).build()?;

    println!("Layouts Extension Example");
    println!("=========================\n");

    let layouts_ext = match api.get_extension::<LayoutsExtension>() {
        Some(ext) => {
            println!("Layouts extension: available\n");
            ext
        }
        None => {
            println!("Layouts extension is not available in this plugin.");
            println!("This extension is typically available on GPU/TPU backends.");
            println!("\nThe Layouts extension provides:");
            println!("  - buffer_memory_layout()     — get on-device layout of a buffer");
            println!("  - client_default_layout()    — query default layout for type/shape");
            println!("  - topology_default_layout()  — query layout for a topology");
            println!("  - executable_output_layouts() — get output layouts of an executable");
            println!("  - SerializedLayout.bytes()   — serialize layout to bytes for caching");
            return Ok(());
        }
    };

    // 1. Query default layout for a specific type and shape
    println!("1. Default Layouts");
    println!("   ---------------");
    let shapes: &[(&str, PrimitiveType, &[i64])] = &[
        ("scalar f32", PrimitiveType::F32, &[]),
        ("vector f32[8]", PrimitiveType::F32, &[8]),
        ("matrix f32[4,8]", PrimitiveType::F32, &[4, 8]),
        ("tensor f32[2,3,4]", PrimitiveType::F32, &[2, 3, 4]),
        ("matrix f16[4,8]", PrimitiveType::F16, &[4, 8]),
    ];
    for (label, ty, dims) in shapes {
        match layouts_ext.client_default_layout(&client, *ty, dims) {
            Ok(layout) => {
                let serialized = layout.serialize()?;
                println!(
                    "   {}: {} bytes serialized",
                    label,
                    serialized.bytes().len()
                );
            }
            Err(e) => println!("   {}: error — {}", label, e),
        }
    }

    // 2. Get layout of an actual buffer
    println!("\n2. Buffer Layouts");
    println!("   --------------");
    let devices = client.addressable_devices()?;
    let device = &devices[0];
    let host_buf = HostBuffer::from_data(
        vec![1.0f32, 2.0, 3.0, 4.0, 5.0, 6.0],
        Some(vec![2, 3]),
        None,
    );
    let device_buf = host_buf.to_sync(device).copy()?;
    match layouts_ext.buffer_memory_layout(&device_buf) {
        Ok(layout) => {
            let serialized = layout.serialize()?;
            println!(
                "   f32[2,3] buffer: {} bytes serialized",
                serialized.bytes().len()
            );
        }
        Err(e) => println!("   f32[2,3] buffer: error — {}", e),
    }

    // 3. Get layout from topology
    println!("\n3. Topology Layouts");
    println!("   ----------------");
    let topology = client.topology()?;
    match layouts_ext.topology_default_layout(&topology, PrimitiveType::F32, &[4, 8]) {
        Ok(layout) => {
            let serialized = layout.serialize()?;
            println!(
                "   f32[4,8] on {}: {} bytes serialized",
                topology.platform_name()?,
                serialized.bytes().len()
            );
        }
        Err(e) => println!("   f32[4,8]: error — {}", e),
    }

    // 4. Get executable output layouts
    println!("\n4. Executable Output Layouts");
    println!("   -------------------------");
    let program = pjrt::Program::new(MLIR, CODE);
    let loaded_exe = client.compile(&program, pjrt::CompileOptions::new())?;
    let executable = loaded_exe.executable()?;
    match layouts_ext.executable_output_layouts(&executable) {
        Ok(layouts) => {
            println!("   {} output layout(s):", layouts.len());
            for (i, layout) in layouts.iter().enumerate() {
                let serialized = layout.serialize()?;
                println!(
                    "     [{}]: {} bytes serialized",
                    i,
                    serialized.bytes().len()
                );
            }
        }
        Err(e) => println!("   error — {}", e),
    }

    Ok(())
}
