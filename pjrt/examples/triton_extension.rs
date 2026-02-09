//! Triton Extension Example
//!
//! This example demonstrates the PJRT Triton extension, which enables
//! compilation of Triton GPU kernels through the PJRT interface.
//! Triton is an open-source language and compiler for GPU programming
//! that simplifies writing custom high-performance kernels.
//!
//! To run this example:
//! ```
//! export PJRT_PLUGIN_PATH=/path/to/pjrt_c_api_gpu_plugin.so
//! cargo run --example triton_extension
//! ```

use pjrt::{self, Client, Result, TritonExtension};

fn main() -> Result<()> {
    let plugin_path = std::env::var("PJRT_PLUGIN_PATH")
        .expect("PJRT_PLUGIN_PATH environment variable must be set");
    let api = pjrt::plugin(&plugin_path).load()?;
    let _client = Client::builder(&api).build()?;

    println!("Triton Extension Example");
    println!("========================\n");

    match api.get_extension::<TritonExtension>() {
        Some(_ext) => {
            println!("Triton extension: available\n");

            // The Triton extension provides a compile method:
            //   ext.compile(module, arch_name, num_warps, num_ctas,
            //               num_stages, enable_fp_fusion,
            //               enable_warp_specialization, serialized_metadata)
            //
            // Parameters:
            //   module:     The Triton IR module as a string
            //   arch_name:  GPU architecture name (e.g., "sm_80", "sm_90")
            //   num_warps:  Number of warps per CTA (e.g., 4)
            //   num_ctas:   Number of CTAs in a cluster (e.g., 1)
            //   num_stages: Number of pipeline stages (e.g., 3)
            //   enable_fp_fusion: Enable floating-point fusion optimizations
            //   enable_warp_specialization: Enable warp specialization
            //   serialized_metadata: Additional metadata as bytes
            //
            // Returns:
            //   TritonCompileResult { name, asm, shared_mem_bytes }

            println!("Triton extension is available but compiling kernels requires");
            println!("valid Triton IR input. Example compile call:");
            println!();
            println!("  let result = ext.compile(");
            println!("      &triton_module,    // Triton IR source");
            println!("      \"sm_80\",            // GPU architecture");
            println!("      4,                 // num_warps");
            println!("      1,                 // num_ctas");
            println!("      3,                 // num_stages");
            println!("      true,              // enable_fp_fusion");
            println!("      false,             // enable_warp_specialization");
            println!("      &[],               // serialized_metadata");
            println!("  )?;");
            println!();
            println!("  println!(\"Kernel name: {{}}\", result.name);");
            println!("  println!(\"Shared memory: {{}} bytes\", result.shared_mem_bytes);");
            println!("  println!(\"Assembly size: {{}} bytes\", result.asm.len());");
        }
        None => {
            println!("Triton extension is not available in this plugin.");
            println!("This extension is typically available on GPU plugins.\n");
            println!("The Triton extension provides:");
            println!("  - compile(module, arch_name, num_warps, num_ctas,");
            println!("           num_stages, enable_fp_fusion,");
            println!("           enable_warp_specialization, serialized_metadata)");
            println!("      → TritonCompileResult {{ name, asm, shared_mem_bytes }}");
            println!();
            println!("Use this extension to compile Triton kernels for GPU execution");
            println!("directly through the PJRT API.");
        }
    }

    Ok(())
}
