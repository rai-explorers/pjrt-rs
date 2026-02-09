//! Custom Partitioner Extension Example
//!
//! This example demonstrates the PJRT Custom Partitioner extension, which
//! allows registering custom SPMD partitioning strategies for operations that
//! the default XLA partitioner cannot handle optimally.
//!
//! Custom partitioners are useful for:
//! - Operations with non-standard data distributions
//! - Domain-specific partitioning strategies
//! - Optimizing communication patterns in distributed training
//!
//! To run this example:
//! ```
//! export PJRT_PLUGIN_PATH=/path/to/pjrt_c_api_plugin.so
//! cargo run --example custom_partitioner_extension
//! ```

use pjrt::{self, Client, CustomPartitionerExtension, Result};

fn main() -> Result<()> {
    let plugin_path = std::env::var("PJRT_PLUGIN_PATH")
        .expect("PJRT_PLUGIN_PATH environment variable must be set");
    let api = pjrt::plugin(&plugin_path).load()?;
    let _client = Client::builder(&api).build()?;

    println!("Custom Partitioner Extension Example");
    println!("====================================\n");

    match api.get_extension::<CustomPartitionerExtension>() {
        Some(_ext) => {
            println!("Custom Partitioner extension: available\n");

            // The Custom Partitioner extension provides two methods:
            //
            // 1. register_custom_partitioner(name, callbacks, can_side_effecting_have_replicated_sharding)
            //    - name: unique operation name for the custom partitioner
            //    - callbacks: raw callbacks pointer for the partitioner implementation
            //    - can_side_effecting_have_replicated_sharding: whether side-effecting ops
            //      can use replicated sharding
            //
            // 2. register_batch_partitionable(names)
            //    - names: list of operation names that support batch partitioning
            //
            // These are typically used by framework developers building custom
            // SPMD-aware operations.

            println!("Custom Partitioner extension is available.");
            println!("Registration requires implementing the C callback interface.");
            println!();
            println!("Example usage:");
            println!("  // Register a custom partitioner for a specific op");
            println!("  ext.register_custom_partitioner(");
            println!("      \"my_custom_op\",");
            println!("      callbacks_ptr,");
            println!("      false,  // can_side_effecting_have_replicated_sharding");
            println!("  )?;");
            println!();
            println!("  // Register operations that support batch partitioning");
            println!("  ext.register_batch_partitionable(&[\"op1\", \"op2\"])?;");
        }
        None => {
            println!("Custom Partitioner extension is not available in this plugin.\n");
            println!("The Custom Partitioner extension provides:");
            println!("  - register_custom_partitioner(name, callbacks, can_side_effecting)");
            println!("      Registers a custom SPMD partitioning strategy for an operation");
            println!();
            println!("  - register_batch_partitionable(names)");
            println!("      Marks operations as supporting batch partitioning");
            println!();
            println!("This extension is used by framework developers who need");
            println!("non-standard SPMD partitioning behavior.");
        }
    }

    Ok(())
}
