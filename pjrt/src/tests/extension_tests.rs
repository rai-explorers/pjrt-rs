//! Integration Tests for PJRT Extensions
//!
//! These tests verify that extensions can be queried and used correctly.
//! Most tests require a PJRT plugin to be loaded via the `PJRT_PLUGIN_PATH`
//! environment variable.

#[cfg(all(test, feature = "integration-tests"))]
mod integration_tests {
    use crate::{Client, ExtensionType, Result};

    fn setup_test_client() -> Result<Client> {
        let plugin_path = std::env::var("PJRT_PLUGIN_PATH")
            .expect("PJRT_PLUGIN_PATH environment variable must be set for integration tests");
        let api = crate::plugin(&plugin_path).load()?;
        Client::builder(&api).build()
    }

    #[test]
    fn test_extension_types_are_defined() {
        // Verify all extension types are properly defined
        let types = [
            ExtensionType::GpuCustomCall,
            ExtensionType::Profiler,
            ExtensionType::CustomPartitioner,
            ExtensionType::Stream,
            ExtensionType::Layouts,
            ExtensionType::Ffi,
            ExtensionType::MemoryDescriptions,
            ExtensionType::Triton,
            ExtensionType::RawBuffer,
            ExtensionType::CrossHostTransfers,
            ExtensionType::ExecutableMetadata,
            ExtensionType::Callback,
            ExtensionType::HostAllocator,
            ExtensionType::TpuTopology,
            ExtensionType::TpuExecutable,
            ExtensionType::Megascale,
            ExtensionType::PhaseCompile,
        ];

        // Verify each type has a raw value
        for ext_type in types {
            let _raw = ext_type.to_raw();
        }
    }

    #[test]
    fn test_stream_extension_query() -> Result<()> {
        let client = setup_test_client()?;
        let api = client.api();

        // Stream extension may or may not be available depending on the plugin
        // This test just verifies the query mechanism works
        let stream_ext = api.get_extension::<crate::StreamExtension>();

        if let Some(ext) = stream_ext {
            println!("Stream extension available: {:?}", ext);
        } else {
            println!("Stream extension not available (this is OK for CPU plugin)");
        }

        Ok(())
    }

    #[test]
    fn test_layouts_extension_query() -> Result<()> {
        let client = setup_test_client()?;
        let api = client.api();

        let layouts_ext = api.get_extension::<crate::LayoutsExtension>();

        if let Some(ext) = layouts_ext {
            println!("Layouts extension available: {:?}", ext);
        } else {
            println!("Layouts extension not available");
        }

        Ok(())
    }

    #[test]
    fn test_profiler_extension_query() -> Result<()> {
        let client = setup_test_client()?;
        let api = client.api();

        let profiler_ext = api.get_extension::<crate::ProfilerExtension>();

        if let Some(ext) = profiler_ext {
            println!("Profiler extension available: {:?}", ext);
            println!("Has profiler API: {}", ext.has_profiler_api());
        } else {
            println!("Profiler extension not available");
        }

        Ok(())
    }

    #[test]
    fn test_callback_extension_query() -> Result<()> {
        let client = setup_test_client()?;
        let api = client.api();

        let callback_ext = api.get_extension::<crate::CallbackExtension>();

        if let Some(ext) = callback_ext {
            println!("Callback extension available: {:?}", ext);
        } else {
            println!("Callback extension not available");
        }

        Ok(())
    }

    #[test]
    fn test_memory_descriptions_extension_query() -> Result<()> {
        let client = setup_test_client()?;
        let api = client.api();

        let mem_desc_ext = api.get_extension::<crate::MemoryDescriptionsExtension>();

        if let Some(ext) = mem_desc_ext {
            println!("Memory descriptions extension available: {:?}", ext);
        } else {
            println!("Memory descriptions extension not available");
        }

        Ok(())
    }

    #[test]
    fn test_gpu_extension_query() -> Result<()> {
        let client = setup_test_client()?;
        let api = client.api();

        let gpu_ext = api.get_extension::<crate::GpuExtension>();

        if let Some(ext) = gpu_ext {
            println!("GPU extension available: {:?}", ext);
        } else {
            println!("GPU extension not available (expected for CPU plugin)");
        }

        Ok(())
    }

    #[test]
    fn test_new_extension_types() -> Result<()> {
        let client = setup_test_client()?;
        let api = client.api();

        // Test the new extension types we added
        let cross_host = api.get_extension::<crate::CrossHostTransfersExtension>();
        if let Some(ext) = cross_host {
            println!("CrossHostTransfers extension available: {:?}", ext);
        } else {
            println!("CrossHostTransfers extension not available");
        }

        let exec_meta = api.get_extension::<crate::ExecutableMetadataExtension>();
        if let Some(ext) = exec_meta {
            println!("ExecutableMetadata extension available: {:?}", ext);
        } else {
            println!("ExecutableMetadata extension not available");
        }

        let host_alloc = api.get_extension::<crate::HostAllocatorExtension>();
        if let Some(ext) = host_alloc {
            println!("HostAllocator extension available: {:?}", ext);
        } else {
            println!("HostAllocator extension not available");
        }

        let tpu_topo = api.get_extension::<crate::TpuTopologyExtension>();
        if let Some(ext) = tpu_topo {
            println!("TpuTopology extension available: {:?}", ext);
        } else {
            println!("TpuTopology extension not available (expected for non-TPU)");
        }

        let tpu_exec = api.get_extension::<crate::TpuExecutableExtension>();
        if let Some(ext) = tpu_exec {
            println!("TpuExecutable extension available: {:?}", ext);
        } else {
            println!("TpuExecutable extension not available (expected for non-TPU)");
        }

        let megascale = api.get_extension::<crate::MegascaleExtension>();
        if let Some(ext) = megascale {
            println!("Megascale extension available: {:?}", ext);
        } else {
            println!("Megascale extension not available");
        }

        Ok(())
    }
}

#[cfg(test)]
mod unit_tests {
    use crate::ExtensionType;

    #[test]
    fn test_extension_type_to_raw_roundtrip() {
        // Test that all extension types have valid raw values
        let types = vec![
            ExtensionType::GpuCustomCall,
            ExtensionType::Profiler,
            ExtensionType::CustomPartitioner,
            ExtensionType::Stream,
            ExtensionType::Layouts,
            ExtensionType::Ffi,
            ExtensionType::MemoryDescriptions,
            ExtensionType::Triton,
            ExtensionType::RawBuffer,
            ExtensionType::CrossHostTransfers,
            ExtensionType::ExecutableMetadata,
            ExtensionType::Callback,
            ExtensionType::HostAllocator,
            ExtensionType::TpuTopology,
            ExtensionType::TpuExecutable,
            ExtensionType::Megascale,
            ExtensionType::PhaseCompile,
            ExtensionType::Example,
        ];

        for ext_type in types {
            // Just verify it doesn't panic and produces a valid value
            let _ = ext_type.to_raw();
        }
    }

    #[test]
    fn test_extension_type_debug() {
        // Verify Debug implementation works
        let ext = ExtensionType::Stream;
        let debug_str = format!("{:?}", ext);
        assert!(debug_str.contains("Stream"));
    }

    #[test]
    fn test_extension_type_equality() {
        // Verify equality works
        assert_eq!(ExtensionType::Stream, ExtensionType::Stream);
        assert_ne!(ExtensionType::Stream, ExtensionType::Layouts);
    }

    #[test]
    fn test_extension_type_clone() {
        // Verify Clone works
        let ext = ExtensionType::Profiler;
        let cloned = ext;
        assert_eq!(ext, cloned);
    }

    #[test]
    fn test_extension_type_all_variants_unique_raw_values() {
        // Test that all extension types have unique raw values
        use std::collections::HashSet;

        let types = [
            ExtensionType::GpuCustomCall,
            ExtensionType::Profiler,
            ExtensionType::CustomPartitioner,
            ExtensionType::Stream,
            ExtensionType::Layouts,
            ExtensionType::Ffi,
            ExtensionType::MemoryDescriptions,
            ExtensionType::Triton,
            ExtensionType::RawBuffer,
            ExtensionType::CrossHostTransfers,
            ExtensionType::ExecutableMetadata,
            ExtensionType::Callback,
            ExtensionType::HostAllocator,
            ExtensionType::TpuTopology,
            ExtensionType::TpuExecutable,
            ExtensionType::Megascale,
            ExtensionType::PhaseCompile,
            ExtensionType::Example,
        ];

        let mut raw_values = HashSet::new();
        for ext_type in types {
            let raw = ext_type.to_raw();
            assert!(
                raw_values.insert(raw),
                "Duplicate raw value for {:?}",
                ext_type
            );
        }
    }

    #[test]
    fn test_extension_type_copy_semantics() {
        // Verify Copy semantics work correctly
        let ext = ExtensionType::Layouts;
        let copied = ext;
        let also_copied = ext; // ext is still usable because it's Copy
        assert_eq!(ext, copied);
        assert_eq!(ext, also_copied);
    }

    #[test]
    fn test_extension_type_format_all_variants() {
        // Ensure all variants have reasonable debug output
        let types_and_expected = [
            (ExtensionType::GpuCustomCall, "GpuCustomCall"),
            (ExtensionType::Profiler, "Profiler"),
            (ExtensionType::CustomPartitioner, "CustomPartitioner"),
            (ExtensionType::Stream, "Stream"),
            (ExtensionType::Layouts, "Layouts"),
            (ExtensionType::Ffi, "Ffi"),
            (ExtensionType::MemoryDescriptions, "MemoryDescriptions"),
            (ExtensionType::Triton, "Triton"),
            (ExtensionType::RawBuffer, "RawBuffer"),
            (ExtensionType::CrossHostTransfers, "CrossHostTransfers"),
            (ExtensionType::ExecutableMetadata, "ExecutableMetadata"),
            (ExtensionType::Callback, "Callback"),
            (ExtensionType::HostAllocator, "HostAllocator"),
            (ExtensionType::TpuTopology, "TpuTopology"),
            (ExtensionType::TpuExecutable, "TpuExecutable"),
            (ExtensionType::Megascale, "Megascale"),
            (ExtensionType::PhaseCompile, "PhaseCompile"),
            (ExtensionType::Example, "Example"),
        ];

        for (ext_type, expected_substring) in types_and_expected {
            let debug_str = format!("{:?}", ext_type);
            assert!(
                debug_str.contains(expected_substring),
                "Debug output '{}' should contain '{}'",
                debug_str,
                expected_substring
            );
        }
    }
}
