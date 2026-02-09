//! Specialized Extensions Example
//!
//! This example demonstrates the PJRT extensions that provide
//! hardware-specific and distributed-computing features. These extensions
//! have full high-level Rust wrappers but typically require specialised
//! plugins (TPU, GPU-Megascale, etc.) rather than the generic CPU plugin.
//!
//! **Fully wrapped extensions** (demonstrated here):
//!
//! - **HostAllocator**: Custom host-side memory allocation (experimental)
//! - **TpuTopology**: TPU-specific topology queries (chip counts, coordinates, etc.)
//! - **TpuExecutable**: TPU-specific executable introspection
//! - **Megascale**: Large-scale distributed training (client contexts, multi-slice)
//! - **Example**: Reference extension for the PJRT extension architecture
//!
//! **Stub-only extensions** (raw pointer access only):
//!
//! - **CrossHostTransfers**: Multi-host buffer transfer protocols
//! - **ExecutableMetadata**: Metadata about compiled executables
//!
//! To run this example:
//! ```
//! export PJRT_PLUGIN_PATH=/path/to/pjrt_c_api_plugin.so
//! cargo run --example stub_extensions
//! ```

use pjrt::{
    self, Client, CrossHostTransfersExtension, ExampleExtension, ExecutableMetadataExtension,
    HostAllocatorExtension, MegascaleExtension, Result, TpuExecutableExtension,
    TpuTopologyExtension,
};

fn main() -> Result<()> {
    let plugin_path = std::env::var("PJRT_PLUGIN_PATH")
        .expect("PJRT_PLUGIN_PATH environment variable must be set");
    let api = pjrt::plugin(&plugin_path).load()?;
    let client = Client::builder(&api).build()?;

    println!("Specialized Extensions Example");
    println!("==============================\n");
    println!(
        "Plugin: {}\n",
        std::path::Path::new(&plugin_path)
            .file_name()
            .unwrap()
            .to_string_lossy()
    );

    // ── HostAllocator (experimental) ────────────────────────────────
    demonstrate_host_allocator(&api, &client);

    // ── TpuTopology ─────────────────────────────────────────────────
    demonstrate_tpu_topology(&api);

    // ── TpuExecutable ───────────────────────────────────────────────
    demonstrate_tpu_executable(&api);

    // ── Megascale ───────────────────────────────────────────────────
    demonstrate_megascale(&api);

    // ── Example (reference extension) ───────────────────────────────
    demonstrate_example(&api);

    // ── Stub-only extensions ────────────────────────────────────────
    demonstrate_stub_extensions(&api);

    Ok(())
}

/// Demonstrates the HostAllocator extension for custom host-side memory allocation.
fn demonstrate_host_allocator(api: &pjrt::Api, client: &Client) {
    println!("HostAllocator Extension (experimental)");
    println!("--------------------------------------");

    match api.get_extension::<HostAllocatorExtension>() {
        Some(ext) => {
            println!("  Available: yes");
            println!("  is_experimental: {}", ext.is_experimental());

            // Query the preferred alignment for host allocations.
            match ext.get_preferred_alignment(client) {
                Ok(alignment) => println!("  preferred alignment: {} bytes", alignment),
                Err(e) => println!("  get_preferred_alignment error: {e}"),
            }

            // Allocate a small host buffer, write to it, then free.
            let size = 1024;
            let alignment = 64;
            match unsafe { ext.allocate(client, size, alignment) } {
                Ok(ptr) => {
                    println!(
                        "  allocated {} bytes at {:p} (alignment={})",
                        size, ptr, alignment
                    );
                    // Write a sentinel value to verify the allocation is usable.
                    unsafe {
                        std::ptr::write_bytes(ptr as *mut u8, 0xAB, size);
                    }
                    match unsafe { ext.free(client, ptr) } {
                        Ok(()) => println!("  freed successfully"),
                        Err(e) => println!("  free error: {e}"),
                    }
                }
                Err(e) => println!("  allocate error: {e}"),
            }
        }
        None => println!("  Not available in this plugin."),
    }
    println!();
}

/// Demonstrates the TpuTopology extension for querying TPU topology information.
fn demonstrate_tpu_topology(api: &pjrt::Api) {
    println!("TpuTopology Extension");
    println!("---------------------");

    match api.get_extension::<TpuTopologyExtension>() {
        Some(_ext) => {
            println!("  Available: yes");

            // TpuTopology methods require a TopologyDescription, which is only
            // available from a TPU plugin. Show the API surface that would be used:
            println!("  API methods available:");
            println!("    Subslice ops:   subslice(), is_subslice_topology(),");
            println!("                    subslice_device_id_from_full_device_id(),");
            println!("                    replace_host_bounds()");
            println!("    Boolean:        is_enhanced_barrier_enabled(),");
            println!("                    has_limited_ici_connectivity(),");
            println!("                    is_reachable_over_limited_ici()");
            println!("    Counts:         process_count(), chips_per_process(),");
            println!("                    core_count_per_chip(), chip_count(),");
            println!("                    core_count(), logical_device_count(),");
            println!("                    logical_device_count_per_process(),");
            println!("                    logical_device_count_per_chip(),");
            println!("                    core_count_per_process()");
            println!("    IDs:            process_ids(), logical_device_ids_on_process(),");
            println!("                    proc_id_and_idx_on_proc_for_chip(),");
            println!("                    proc_id_and_idx_on_proc_for_logi_device()");
            println!("    Coordinates:    process_coord_from_id(), chip_id_from_coord(),");
            println!("                    logical_device_id_from_chip_coord_and_idx(),");
            println!("                    chip_coord_and_idx_for_logi_device()");
            println!("    Bounds:         chips_per_process_bounds(), chip_bounds(),");
            println!("                    process_bounds()");
            println!("    Config:         get_routing_strategy(), get_slice_config(),");
            println!("                    get_slice_configs(), get_default_platform_config()");

            // Usage snippet (requires a real TPU TopologyDescription):
            // ```
            // let topology = api.topology_description()?;
            // let count = ext.process_count(&topology)?;
            // let chips = ext.chip_count(&topology)?;
            // let strategy = ext.get_routing_strategy(&topology, 256)?;
            // ```
        }
        None => println!("  Not available in this plugin (requires TPU)."),
    }
    println!();
}

/// Demonstrates the TpuExecutable extension for TPU-specific executable introspection.
fn demonstrate_tpu_executable(api: &pjrt::Api) {
    println!("TpuExecutable Extension");
    println!("-----------------------");

    match api.get_extension::<TpuExecutableExtension>() {
        Some(_ext) => {
            println!("  Available: yes");
            println!("  API methods available:");
            println!("    get_target_arguments(serialized_exec) -> OwnedTargetArguments");
            println!(
                "    get_core_program_abi_version(serialized_exec) -> OwnedCoreProgramAbiVersion"
            );
            println!("    get_hlo_module_with_config(serialized_exec) -> OwnedHloModuleWithConfig");
            println!();
            println!("  Return types use RAII — owned data is freed on Drop:");
            println!("    OwnedTargetArguments::as_bytes() -> &[u8]");
            println!("    OwnedCoreProgramAbiVersion::as_bytes() -> &[u8]");
            println!("    OwnedHloModuleWithConfig::as_bytes() -> &[u8]");

            // Usage snippet (requires a real TPU serialized executable):
            // ```
            // let serialized: &[u8] = /* serialized TPU executable */;
            // let args = ext.get_target_arguments(serialized)?;
            // println!("target args: {} bytes", args.as_bytes().len());
            //
            // let version = ext.get_core_program_abi_version(serialized)?;
            // println!("ABI version: {} bytes", version.as_bytes().len());
            // ```
        }
        None => println!("  Not available in this plugin (requires TPU)."),
    }
    println!();
}

/// Demonstrates the Megascale extension for large-scale distributed training.
fn demonstrate_megascale(api: &pjrt::Api) {
    println!("Megascale Extension");
    println!("-------------------");

    match api.get_extension::<MegascaleExtension>() {
        Some(_ext) => {
            println!("  Available: yes");
            println!("  API surface:");
            println!();
            println!("  Client context management:");
            println!(
                "    create_client_context_from_pjrt_client(client) -> MegascaleClientContext"
            );
            println!("    create_default_client_context()               -> MegascaleClientContext");
            println!("    delete_client_context(ctx)");
            println!();
            println!("  MegascaleClientContext methods:");
            println!("    initialize()");
            println!("    unblock_pending_work(launch_id, expire_after_ms)");
            println!("    megascale_port() -> i32");
            println!();
            println!("  Multi-slice configuration:");
            println!("    create_aot_config(topology, num_slices) -> MegascaleMultiSliceConfig");
            println!("    create_multi_slice_config(...)           -> MegascaleMultiSliceConfig");
            println!();
            println!("  MegascaleMultiSliceConfig methods:");
            println!("    num_slices() -> i32");
            println!("    slice_id() -> i32");
            println!("    get_num_devices_per_slice() -> Vec<(i32, i32)>");
            println!("    serialize() -> Vec<u8>");
            println!();
            println!("  RAII: MegascaleClientContext and MegascaleMultiSliceConfig");
            println!("  are automatically destroyed on Drop.");

            // Usage snippet (requires a Megascale-capable plugin):
            // ```
            // let ctx = ext.create_default_client_context()?;
            // ctx.initialize()?;
            // let port = ctx.megascale_port()?;
            // println!("megascale port: {port}");
            // // ctx is automatically destroyed when dropped
            // ```
        }
        None => println!("  Not available in this plugin (requires Megascale-capable hardware)."),
    }
    println!();
}

/// Demonstrates the Example extension (reference implementation for testing).
fn demonstrate_example(api: &pjrt::Api) {
    println!("Example Extension (reference implementation)");
    println!("--------------------------------------------");

    match api.get_extension::<ExampleExtension>() {
        Some(ext) => {
            println!("  Available: yes");
            println!("  type_id: {:?}", ext.type_id());

            // Create an ExampleExtensionCpp handle, call a method, then destroy.
            match ext.create() {
                Ok(mut cpp) => {
                    println!("  Created ExampleExtensionCpp handle");
                    match ext.example_method(&mut cpp, 42) {
                        Ok(()) => println!("  example_method(42) succeeded"),
                        Err(e) => println!("  example_method error: {e}"),
                    }
                    match ext.destroy(cpp) {
                        Ok(()) => println!("  Destroyed handle"),
                        Err(e) => println!("  destroy error: {e}"),
                    }
                }
                Err(e) => println!("  create error: {e}"),
            }
        }
        None => println!("  Not available in this plugin."),
    }
    println!();
}

/// Lists the remaining stub-only extensions that expose only raw_ptr() access.
fn demonstrate_stub_extensions(api: &pjrt::Api) {
    println!("Stub-only Extensions (raw pointer access)");
    println!("------------------------------------------");

    let status = if api.has_extension::<CrossHostTransfersExtension>() {
        "available"
    } else {
        "not available"
    };
    println!(
        "  CrossHostTransfers: {} — multi-host buffer transfer protocols",
        status
    );
    if let Some(ext) = api.get_extension::<CrossHostTransfersExtension>() {
        println!("    raw_ptr: {:p}", ext.raw_ptr());
    }

    let status = if api.has_extension::<ExecutableMetadataExtension>() {
        "available"
    } else {
        "not available"
    };
    println!(
        "  ExecutableMetadata: {} — executable metadata (fingerprints, cost analysis)",
        status
    );
    if let Some(ext) = api.get_extension::<ExecutableMetadataExtension>() {
        println!("    raw_ptr: {:p}", ext.raw_ptr());
    }

    println!();
    println!("Note: CrossHostTransfers and ExecutableMetadata currently expose");
    println!("only raw_ptr() for direct C API access. Higher-level wrappers");
    println!("will be added when the underlying C headers stabilise.");
}
