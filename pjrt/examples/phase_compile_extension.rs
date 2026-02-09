//! Phase Compile Extension Example
//!
//! This example demonstrates the PJRT Phase Compile extension, which provides
//! access to individual compilation phases. This is useful for:
//! - Caching intermediate compilation artifacts
//! - Debugging compilation issues at specific phases
//! - Splitting compilation across machines or processes
//!
//! The CPU plugin supports this extension.
//!
//! To run this example:
//! ```
//! export PJRT_PLUGIN_PATH=/path/to/pjrt_c_api_plugin.so
//! cargo run --example phase_compile_extension
//! ```

use pjrt::{self, Client, CompileOptions, PhaseCompileExtension, Result};

fn main() -> Result<()> {
    let plugin_path = std::env::var("PJRT_PLUGIN_PATH")
        .expect("PJRT_PLUGIN_PATH environment variable must be set");
    let api = pjrt::plugin(&plugin_path).load()?;
    let client = Client::builder(&api).build()?;

    println!("Phase Compile Extension Example");
    println!("===============================\n");

    // 1. Check for and obtain the extension
    let phase_ext = match api.get_extension::<PhaseCompileExtension>() {
        Some(ext) => {
            println!("Phase Compile extension: available");
            ext
        }
        None => {
            println!("Phase Compile extension is not available in this plugin.");
            return Ok(());
        }
    };

    // 2. Create a phase compiler
    let compiler = match phase_ext.get_compiler() {
        Ok(c) => {
            println!("Phase compiler: created\n");
            c
        }
        Err(e) => {
            println!("Could not create phase compiler: {}\n", e);
            println!("The PhaseCompile extension is registered but the");
            println!("get_compiler function is not implemented by this plugin.\n");
            println!("PhaseCompile API overview:");
            println!("  ext.get_compiler()           → PhaseCompiler");
            println!("  compiler.get_phase_names()   → Vec<String>");
            println!("  compiler.run_phases(");
            println!("      input_programs,           // serialized PjRtPartialProgramProto");
            println!("      phase_names,              // which phases to run");
            println!("      compile_options,           // CompileOptions");
            println!("      topology,                  // TopologyDescription");
            println!("  ) → PhaseCompileOutput {{ output_programs }}");
            return Ok(());
        }
    };

    // 3. List all registered compilation phases
    let phase_names = compiler.get_phase_names()?;
    println!("Registered phases ({} total):", phase_names.len());
    for (i, name) in phase_names.iter().enumerate() {
        println!("  [{}] {}", i, name);
    }

    // 4. Demonstrate running specific phases (requires a topology)
    let topology = client.topology()?;
    let compile_options = CompileOptions::new();

    println!(
        "\nRunning phases with topology: {}",
        topology.platform_name()?
    );

    // Run each phase individually to see what they do
    // Note: phases expect serialized PjRtPartialProgramProto input,
    // which is an internal format. For demonstration, we show the API
    // pattern even though we can't create valid input without the XLA compiler
    // producing it first.
    if !phase_names.is_empty() {
        println!(
            "\nAttempting to run first phase ('{}') with empty input...",
            phase_names[0]
        );
        match compiler.run_phases(&[], &[phase_names[0].clone()], &compile_options, &topology) {
            Ok(output) => {
                println!(
                    "  Phase completed: {} output program(s)",
                    output.output_programs.len()
                );
                for (i, prog) in output.output_programs.iter().enumerate() {
                    println!("    Output [{}]: {} bytes", i, prog.len());
                }
            }
            Err(e) => {
                println!("  Phase failed (expected with empty input): {}", e);
            }
        }
    }

    println!("\nUsage notes:");
    println!("  - Phase compilation is typically used for caching intermediate artifacts");
    println!("  - Input programs must be serialized PjRtPartialProgramProto");
    println!("  - Phases run in the order specified by the phase names list");
    println!("  - Output programs can be fed as input to subsequent phases");

    Ok(())
}
