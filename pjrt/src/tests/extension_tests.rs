//! Unit Tests for PJRT Extensions
//!
//! These tests verify the extension types and their basic behavior.
//! They do not require a PJRT plugin to be loaded.

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
