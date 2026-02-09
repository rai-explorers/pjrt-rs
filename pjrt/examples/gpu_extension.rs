//! GPU Extension Example
//!
//! This example demonstrates the PJRT GPU extension, which provides
//! registration of GPU custom call handlers. This is useful for:
//! - Implementing custom CUDA/ROCm kernels callable from XLA programs
//! - Registering lifecycle handlers (prepare, initialize, execute)
//! - Using both typed and untyped custom call APIs
//!
//! This extension is only available on GPU backends (CUDA/ROCm).
//!
//! To run this example:
//! ```
//! export PJRT_PLUGIN_PATH=/path/to/pjrt_c_api_gpu_plugin.so
//! cargo run --example gpu_extension
//! ```

use pjrt::{self, Client, CustomCallApiVersion, GpuExtension, Result};

fn main() -> Result<()> {
    let plugin_path = std::env::var("PJRT_PLUGIN_PATH")
        .expect("PJRT_PLUGIN_PATH environment variable must be set");
    let api = pjrt::plugin(&plugin_path).load()?;
    let client = Client::builder(&api).build()?;

    println!("GPU Extension Example");
    println!("=====================\n");

    let gpu_ext = match api.get_extension::<GpuExtension>() {
        Some(ext) => {
            println!("GPU extension: available\n");
            ext
        }
        None => {
            println!("GPU extension is not available in this plugin.");
            println!(
                "Platform: {} (version: {})",
                client.platform_name()?,
                client.platform_version()?
            );
            println!("This extension is only available on GPU backends (CUDA/ROCm).\n");
            println!("The GPU extension provides:");
            println!("  - register_custom_call(name, api_version, ...)");
            println!("      Register a GPU custom call with lifecycle handlers:\n");
            println!("      API versions:");
            println!(
                "        {:?} — raw void* interface",
                CustomCallApiVersion::Untyped
            );
            println!(
                "        {:?} — XLA FFI typed interface",
                CustomCallApiVersion::Typed
            );
            println!();
            println!("      Lifecycle stages:");
            println!("        instantiate — called once per executable");
            println!("        prepare     — called before first execution");
            println!("        initialize  — called once for initialization");
            println!("        execute     — called for each invocation");
            println!();
            println!("Usage pattern:");
            println!("  let gpu_ext = api.get_extension::<GpuExtension>().unwrap();");
            println!("  unsafe {{");
            println!(
                "      gpu_ext.register_custom_call(\"my_kernel\", CustomCallApiVersion::Typed,"
            );
            println!("          None, None, None, Some(execute_handler))?;");
            println!("  }}");
            return Ok(());
        }
    };

    // When available, demonstrate custom call registration
    println!("GPU Custom Call Registration");
    println!("----------------------------");
    println!(
        "Platform: {} (version: {})",
        client.platform_name()?,
        client.platform_version()?
    );

    // In a real application, you would register handlers like:
    //
    // unsafe {
    //     gpu_ext.register_custom_call(
    //         "my_custom_matmul",
    //         CustomCallApiVersion::Typed,
    //         None,    // instantiate
    //         None,    // prepare
    //         None,    // initialize
    //         Some(my_execute_handler as CustomCallHandler),
    //     )?;
    // }
    //
    // The execute handler would be a function pointer matching the XLA custom call ABI.

    println!("GPU extension is ready for custom call registration.");
    println!("See the PJRT GPU extension API for handler function signatures.");
    let _ = gpu_ext; // keep the extension alive for demonstration

    Ok(())
}
