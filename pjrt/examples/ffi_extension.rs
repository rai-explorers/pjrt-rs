//! FFI Extension Example
//!
//! This example demonstrates the PJRT FFI (Foreign Function Interface) extension,
//! which allows registering custom operations that can be called from compiled
//! programs. This is useful for:
//! - Implementing custom kernels in native code
//! - Integrating third-party libraries into XLA computations
//! - Registering custom type handlers
//!
//! To run this example:
//! ```
//! export PJRT_PLUGIN_PATH=/path/to/pjrt_c_api_plugin.so
//! cargo run --example ffi_extension
//! ```

use pjrt::{self, Client, FfiExtension, FfiTypeInfo, Result};

fn main() -> Result<()> {
    let plugin_path = std::env::var("PJRT_PLUGIN_PATH")
        .expect("PJRT_PLUGIN_PATH environment variable must be set");
    let api = pjrt::plugin(&plugin_path).load()?;
    let client = Client::builder(&api).build()?;

    println!("FFI Extension Example");
    println!("=====================\n");

    let ffi_ext = match api.get_extension::<FfiExtension>() {
        Some(ext) => {
            println!("FFI extension: available\n");
            ext
        }
        None => {
            println!("FFI extension is not available in this plugin.");
            println!("This extension is available on backends supporting custom calls.");
            println!("\nThe FFI extension provides:");
            println!("  - register_type()    — register a custom type in the FFI type registry");
            println!("  - register_handler() — register a custom FFI call handler");
            println!("  - add_user_data()    — attach user data to an execution context");
            return Ok(());
        }
    };

    // 1. Register a custom type
    println!("1. Custom Type Registration");
    println!("   -------------------------");

    let type_info = FfiTypeInfo {
        deleter: None,
        _serialize: std::marker::PhantomData,
        _deserialize: std::marker::PhantomData,
    };

    match ffi_ext.register_type("my_custom_type", &type_info, 0) {
        Ok(type_id) => println!("   Registered type 'my_custom_type' → ID {}", type_id),
        Err(e) => println!("   Failed to register type: {}", e),
    }

    // 2. Demonstrate handler registration (requires a valid XLA_FFI_Handler)
    println!("\n2. FFI Handler Registration");
    println!("   -------------------------");
    println!("   Platform: {}", client.platform_name()?);
    println!("   To register a handler, you need:");
    println!("   - A target name (the custom call target in the HLO)");
    println!("   - A platform name (e.g., \"Host\" for CPU, \"CUDA\" for GPU)");
    println!("   - An XLA_FFI_Handler pointer (from xla::ffi::Ffi::Bind())");
    println!("   - Handler traits (e.g., command_buffer_compatible)");
    println!();
    println!("   Example usage pattern:");
    println!("     let handler: FfiHandler = /* from XLA FFI binding */;");
    println!("     let mut traits = FfiHandlerTraits::empty();");
    println!("     traits.set_command_buffer_compatible(true);");
    println!("     ffi_ext.register_handler(\"my_op\", \"Host\", handler, traits)?;");

    // 3. Demonstrate execution context user data
    println!("\n3. Execution Context User Data");
    println!("   ----------------------------");
    println!("   User data allows passing custom state to FFI handlers during execution:");
    println!("     let ctx = client.create_execute_context()?;");
    println!("     ffi_ext.add_user_data(&ctx, type_id, data_ptr)?;");
    println!("   The handler can then retrieve this data during execution.");

    Ok(())
}
