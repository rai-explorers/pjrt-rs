//! Unit Tests for PJRT Extensions
//!
//! These tests verify extension type invariants.

#[cfg(test)]
mod unit_tests {
    use std::collections::HashSet;

    use crate::ExtensionType;

    #[test]
    fn test_extension_type_all_variants_unique_raw_values() {
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
    fn test_extension_type_format_all_variants() {
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
