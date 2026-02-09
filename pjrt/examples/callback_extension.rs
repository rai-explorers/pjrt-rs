//! Callback Extension Example
//!
//! This example demonstrates how to use the PJRT Callback Extension:
//! 1. Accessing the callback extension from a client
//! 2. Registering custom callbacks for various events
//! 3. Handling TPU slice builder callbacks
//!
//! Callbacks are useful for custom error handling and monitoring
//! in distributed training scenarios.
//!
//! To run this example:
//! ```
//! export PJRT_PLUGIN_PATH=/path/to/pjrt_c_api_plugin_with_callback_extension.so
//! cargo run --example callback_extension
//! ```

use pjrt::{self, CallbackExt, Client, Result};

fn main() -> Result<()> {
    let plugin_path = std::env::var("PJRT_PLUGIN_PATH")
        .expect("PJRT_PLUGIN_PATH environment variable must be set");
    let api = pjrt::plugin(&plugin_path).load()?;

    let client = Client::builder(&api).build()?;

    println!("Callback Extension Example");
    println!("========================");

    // Try to access the callback extension
    // Note: This depends on the PJRT plugin having the callback extension
    match client.callback_extension() {
        Some(callback_ext) => {
            println!("Callback extension is available!");

            // Demonstrate TPU slice failure handling
            demonstrate_tpu_slice_callback(&callback_ext, &client)?;

            // Register custom handlers
            register_custom_callbacks(&callback_ext, &client)?;
        }
        None => {
            println!("Callback extension is not available in this plugin.");
            println!("This is normal for CPU-only plugins or plugins without callback support.");
        }
    }

    println!("\nNote: In a real application:");
    println!("1. Check if the callback extension is available before use");
    println!("2. Register callbacks early in the application lifecycle");
    println!("3. Handle callback errors appropriately");
    println!("4. Clean up callbacks when the application exits");

    Ok(())
}

/// Demonstrates handling TPU slice failure callbacks
fn demonstrate_tpu_slice_callback(
    callback_ext: &pjrt::CallbackExtension,
    client: &Client,
) -> Result<()> {
    println!("\nTPU Slice Failure Callback Example:");

    // Define an extern "C" callback that handles TPU slice builder events.
    // The `args` pointer is a PJRT_Callback_Tpu_SliceBuilderArgs; `user_arg`
    // is an application-supplied context pointer (unused here).
    unsafe extern "C" fn tpu_slice_callback(
        _args: *mut std::ffi::c_void,
        _user_arg: *mut std::ffi::c_void,
    ) {
        println!("  [callback] TPU slice builder event received");
    }

    // Register the callback for TPU slice builder events.
    unsafe {
        callback_ext.register_callback(
            client,
            pjrt::CallbackType::TpuSliceBuilder,
            tpu_slice_callback,
            std::ptr::null_mut(),
        )?;
    }
    println!("  Registered TPU slice builder callback");

    // Show available failure types for reference
    println!("  Available TPU slice failure types:");

    let failure_types = vec![
        (pjrt::TpuSliceFailureType::Unknown, "Unknown error"),
        (pjrt::TpuSliceFailureType::InitError, "Initialization error"),
        (
            pjrt::TpuSliceFailureType::WorkerUnavailable,
            "Worker unavailable",
        ),
        (
            pjrt::TpuSliceFailureType::FlappingTaskError,
            "Flapping task",
        ),
        (
            pjrt::TpuSliceFailureType::SoftwareInjectedError,
            "Software injected test error",
        ),
        (
            pjrt::TpuSliceFailureType::ChipDriverError,
            "Chip driver error",
        ),
    ];

    for (failure_type, description) in failure_types {
        println!("    {:?}: {}", failure_type, description);
    }

    Ok(())
}

/// Demonstrates registering a pre-fatal error callback
fn register_custom_callbacks(
    callback_ext: &pjrt::CallbackExtension,
    client: &Client,
) -> Result<()> {
    println!("\nRegistering Pre-Fatal Callback:");

    // Define a pre-fatal callback that logs before the runtime terminates.
    unsafe extern "C" fn prefatal_callback(
        _args: *mut std::ffi::c_void,
        _user_arg: *mut std::ffi::c_void,
    ) {
        eprintln!("[prefatal] PJRT is about to terminate — cleaning up");
    }

    unsafe {
        callback_ext.register_callback(
            client,
            pjrt::CallbackType::Prefatal,
            prefatal_callback,
            std::ptr::null_mut(),
        )?;
    }
    println!("  Registered pre-fatal callback successfully");

    Ok(())
}

/// Example error handling strategy for callback failures
#[allow(dead_code)]
mod callback_error_handler {
    use pjrt::{Result, TpuSliceFailureType};

    /// Handles TPU slice failure based on the type of error
    pub fn handle_tpu_slice_failure(
        failure_type: TpuSliceFailureType,
        context: &str,
    ) -> Result<()> {
        match failure_type {
            TpuSliceFailureType::InitError => {
                println!("TPU slice initialization error in {}: retrying", context);
                Ok(())
            }
            TpuSliceFailureType::WorkerUnavailable => {
                println!("Worker unavailable in {}: reconfiguring", context);
                Ok(())
            }
            TpuSliceFailureType::FlappingTaskError => {
                println!("Flapping task in {}: pausing retries", context);
                Ok(())
            }
            TpuSliceFailureType::ChipDriverError => {
                println!("Chip driver error in {}: scheduling restart", context);
                Ok(())
            }
            TpuSliceFailureType::SoftwareInjectedError => {
                println!("Software injected error in {}: ignoring (test)", context);
                Ok(())
            }
            TpuSliceFailureType::Unknown => {
                println!("Unknown error in {}: escalating", context);
                Err(pjrt::Error::InvalidArgument(
                    "Unknown TPU slice failure".to_string(),
                ))
            }
        }
    }
}
