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

use pjrt::{self, Client, Result};

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
            demonstrate_tpu_slice_callback(&callback_ext)?;

            // Register custom handlers
            register_custom_callbacks(&callback_ext)?;
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
fn demonstrate_tpu_slice_callback(_callback_ext: &pjrt::CallbackExtension) -> Result<()> {
    println!("\nTPU Slice Failure Callback Example:");

    // In a real implementation, you would register a callback like this:
    /*
    callback_ext.register_callback(
        pjrt::CallbackType::TpuSliceBuilder,
        Box::new(|args, user_data| {
            // Handle TPU slice builder callback
            // This is called when a slice fails to build

            let failure_type = args.failure_type;
            match failure_type {
                pjrt::TpuSliceFailureType::InitError => {
                    println!("Received TPU slice initialization error");
                    // Handle initialization error
                }
                pjrt::TpuSliceFailureType::WorkerUnavailable => {
                    println!("Worker became unavailable");
                    // Handle worker failure
                }
                pjrt::TpuSliceFailureType::FlappingTaskError => {
                    println!("Task is flapping (restarting too frequently)");
                    // Handle flapping task
                }
                pjrt::TpuSliceFailureType::ChipDriverError => {
                    println!("Chip driver error detected");
                    // Handle driver error
                }
                pjrt::TpuSliceFailureType::SoftwareInjectedError => {
                    println!("Software injected error (testing)");
                    // Handle test error
                }
                pjrt::TpuSliceFailureType::Unknown => {
                    println!("Unknown TPU slice failure");
                    // Handle unknown error
                }
            }

            // Return PJRT_SUCCESS
            std::ptr::null_mut()
        }),
        // User data pointer (often null or a pointer to application state)
        std::ptr::null_mut()
    )?;
    */

    // For this example, we'll show the enum values
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

/// Demonstrates registering various custom callbacks
fn register_custom_callbacks(_callback_ext: &pjrt::CallbackExtension) -> Result<()> {
    println!("\nRegistering Custom Callbacks:");

    // In a real implementation, you might register multiple callbacks:

    // 1. Pre-fatal error callback
    println!("  1. Pre-fatal error callback:");
    println!("     Called before PJRT terminates due to a fatal error");
    println!("     Allows cleanup of application state");

    // 2. Memory pressure callback
    println!("  2. Memory pressure callback:");
    println!("     Called when device memory is running low");
    println!("     Allows application to free buffers or adjust memory usage");

    // 3. Progress reporting callback
    println!("  3. Progress reporting callback:");
    println!("     Called for long-running operations");
    println!("     Allows updating UI or logging progress");

    // 4. Custom user callback
    println!("  4. Custom application callback:");
    println!("     User-defined callback for application-specific events");

    println!("\n  Note: Actual callback registration requires:");
    println!("  - Boxed closure conforming to the callback signature");
    println!("  - User data pointer (optional)");
    println!("  - Proper error handling within the callback");

    Ok(())
}

/// Example error handling strategy for callback failures
mod callback_error_handler {
    use pjrt::{Result, TpuSliceFailureType};

    /// Handles TPU slice failure based on the type of error
    pub fn handle_tpu_slice_failure(
        failure_type: TpuSliceFailureType,
        context: &str,
    ) -> Result<()> {
        match failure_type {
            TpuSliceFailureType::InitError => {
                // For initialization errors, we might want to retry with different parameters
                println!("TPU slice initialization error in {}: retrying", context);
                // Implementation would include retry logic
                Ok(())
            }
            TpuSliceFailureType::WorkerUnavailable => {
                // Worker unavailability might require reconfiguring the job
                println!("Worker unavailable in {}: reconfiguring", context);
                // Implementation would include reconfiguration logic
                Ok(())
            }
            TpuSliceFailureType::FlappingTaskError => {
                // Flapping tasks might need to be temporarily disabled
                println!("Flapping task in {}: pausing retries", context);
                // Implementation would include pause/resume logic
                Ok(())
            }
            TpuSliceFailureType::ChipDriverError => {
                // Driver errors often require restarting the job
                println!("Chip driver error in {}: scheduling restart", context);
                // Implementation would include restart logic
                Ok(())
            }
            TpuSliceFailureType::SoftwareInjectedError => {
                // Test errors can be safely ignored or logged
                println!("Software injected error in {}: ignoring (test)", context);
                Ok(())
            }
            TpuSliceFailureType::Unknown => {
                // Unknown errors should be logged and potentially escalated
                println!("Unknown error in {}: escalating", context);
                Err(pjrt::Error::InvalidArgument(
                    "Unknown TPU slice failure".to_string(),
                ))
            }
        }
    }
}
