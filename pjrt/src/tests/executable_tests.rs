//! Unit Tests for Executable Module
//!
//! These tests verify the types in the executable and loaded_executable modules:
//! - `CompiledMemoryStats`: Memory usage statistics
//! - `ExecuteOptions`: Execution options FFI mapping
//! - `ExecutionInputs`: Input buffer structure
//!
//! Note: Tests for `Executable` and `LoadedExecutable` that require actual
//! PJRT plugin operations are in integration tests.
//!
//! Tests do not require a PJRT plugin to run.

#[cfg(test)]
mod compiled_memory_stats_tests {
    use pjrt_sys::PJRT_Executable_GetCompiledMemoryStats_Args;

    use crate::CompiledMemoryStats;

    #[test]
    fn test_compiled_memory_stats_from_args_all_zeros() {
        let mut args = PJRT_Executable_GetCompiledMemoryStats_Args::new();
        // All fields default to zero
        args.generated_code_size_in_bytes = 0;
        args.argument_size_in_bytes = 0;
        args.output_size_in_bytes = 0;
        args.alias_size_in_bytes = 0;
        args.temp_size_in_bytes = 0;
        args.host_generated_code_size_in_bytes = 0;
        args.host_argument_size_in_bytes = 0;
        args.host_output_size_in_bytes = 0;
        args.host_alias_size_in_bytes = 0;
        args.host_temp_size_in_bytes = 0;

        let stats = CompiledMemoryStats::from(args);

        assert_eq!(stats.generated_code_size_in_bytes, 0);
        assert_eq!(stats.argument_size_in_bytes, 0);
        assert_eq!(stats.output_size_in_bytes, 0);
        assert_eq!(stats.alias_size_in_bytes, 0);
        assert_eq!(stats.temp_size_in_bytes, 0);
        assert_eq!(stats.host_generated_code_size_in_bytes, 0);
        assert_eq!(stats.host_argument_size_in_bytes, 0);
        assert_eq!(stats.host_output_size_in_bytes, 0);
        assert_eq!(stats.host_alias_size_in_bytes, 0);
        assert_eq!(stats.host_temp_size_in_bytes, 0);
    }

    #[test]
    fn test_compiled_memory_stats_from_args_with_values() {
        let mut args = PJRT_Executable_GetCompiledMemoryStats_Args::new();
        args.generated_code_size_in_bytes = 1024;
        args.argument_size_in_bytes = 2048;
        args.output_size_in_bytes = 4096;
        args.alias_size_in_bytes = 512;
        args.temp_size_in_bytes = 8192;
        args.host_generated_code_size_in_bytes = 256;
        args.host_argument_size_in_bytes = 128;
        args.host_output_size_in_bytes = 64;
        args.host_alias_size_in_bytes = 32;
        args.host_temp_size_in_bytes = 16;

        let stats = CompiledMemoryStats::from(args);

        assert_eq!(stats.generated_code_size_in_bytes, 1024);
        assert_eq!(stats.argument_size_in_bytes, 2048);
        assert_eq!(stats.output_size_in_bytes, 4096);
        assert_eq!(stats.alias_size_in_bytes, 512);
        assert_eq!(stats.temp_size_in_bytes, 8192);
        assert_eq!(stats.host_generated_code_size_in_bytes, 256);
        assert_eq!(stats.host_argument_size_in_bytes, 128);
        assert_eq!(stats.host_output_size_in_bytes, 64);
        assert_eq!(stats.host_alias_size_in_bytes, 32);
        assert_eq!(stats.host_temp_size_in_bytes, 16);
    }

    #[test]
    fn test_compiled_memory_stats_from_args_large_values() {
        let mut args = PJRT_Executable_GetCompiledMemoryStats_Args::new();
        // Test with large memory sizes (multi-GB)
        args.generated_code_size_in_bytes = 10 * 1024 * 1024 * 1024; // 10 GB
        args.argument_size_in_bytes = 5 * 1024 * 1024 * 1024; // 5 GB
        args.output_size_in_bytes = 2 * 1024 * 1024 * 1024; // 2 GB
        args.temp_size_in_bytes = 1024 * 1024 * 1024; // 1 GB

        let stats = CompiledMemoryStats::from(args);

        assert_eq!(stats.generated_code_size_in_bytes, 10 * 1024 * 1024 * 1024);
        assert_eq!(stats.argument_size_in_bytes, 5 * 1024 * 1024 * 1024);
        assert_eq!(stats.output_size_in_bytes, 2 * 1024 * 1024 * 1024);
        assert_eq!(stats.temp_size_in_bytes, 1024 * 1024 * 1024);
    }

    #[test]
    fn test_compiled_memory_stats_total_device_memory() {
        let mut args = PJRT_Executable_GetCompiledMemoryStats_Args::new();
        args.generated_code_size_in_bytes = 1000;
        args.argument_size_in_bytes = 2000;
        args.output_size_in_bytes = 3000;
        args.alias_size_in_bytes = 500;
        args.temp_size_in_bytes = 1500;

        let stats = CompiledMemoryStats::from(args);

        // Calculate total device memory (excluding alias as it's shared)
        let total_device = stats.generated_code_size_in_bytes
            + stats.argument_size_in_bytes
            + stats.output_size_in_bytes
            + stats.temp_size_in_bytes;
        assert_eq!(total_device, 7500);
    }

    #[test]
    fn test_compiled_memory_stats_total_host_memory() {
        let mut args = PJRT_Executable_GetCompiledMemoryStats_Args::new();
        args.host_generated_code_size_in_bytes = 100;
        args.host_argument_size_in_bytes = 200;
        args.host_output_size_in_bytes = 300;
        args.host_alias_size_in_bytes = 50;
        args.host_temp_size_in_bytes = 150;

        let stats = CompiledMemoryStats::from(args);

        // Calculate total host memory (excluding alias as it's shared)
        let total_host = stats.host_generated_code_size_in_bytes
            + stats.host_argument_size_in_bytes
            + stats.host_output_size_in_bytes
            + stats.host_temp_size_in_bytes;
        assert_eq!(total_host, 750);
    }

    #[test]
    fn test_compiled_memory_stats_negative_values() {
        // While negative values don't make semantic sense for memory,
        // the conversion should handle them without panicking
        let mut args = PJRT_Executable_GetCompiledMemoryStats_Args::new();
        args.generated_code_size_in_bytes = -1;
        args.argument_size_in_bytes = -100;

        let stats = CompiledMemoryStats::from(args);

        assert_eq!(stats.generated_code_size_in_bytes, -1);
        assert_eq!(stats.argument_size_in_bytes, -100);
    }

    #[test]
    fn test_compiled_memory_stats_max_i64_values() {
        let mut args = PJRT_Executable_GetCompiledMemoryStats_Args::new();
        args.generated_code_size_in_bytes = i64::MAX;
        args.output_size_in_bytes = i64::MAX;

        let stats = CompiledMemoryStats::from(args);

        assert_eq!(stats.generated_code_size_in_bytes, i64::MAX);
        assert_eq!(stats.output_size_in_bytes, i64::MAX);
    }

    #[test]
    fn test_compiled_memory_stats_field_independence() {
        // Verify that each field is independent and correctly mapped
        for i in 0..10 {
            let mut args = PJRT_Executable_GetCompiledMemoryStats_Args::new();
            match i {
                0 => args.generated_code_size_in_bytes = 42,
                1 => args.argument_size_in_bytes = 42,
                2 => args.output_size_in_bytes = 42,
                3 => args.alias_size_in_bytes = 42,
                4 => args.temp_size_in_bytes = 42,
                5 => args.host_generated_code_size_in_bytes = 42,
                6 => args.host_argument_size_in_bytes = 42,
                7 => args.host_output_size_in_bytes = 42,
                8 => args.host_alias_size_in_bytes = 42,
                9 => args.host_temp_size_in_bytes = 42,
                _ => unreachable!(),
            }
            let stats = CompiledMemoryStats::from(args);

            // Count how many fields are 42
            let count = [
                stats.generated_code_size_in_bytes,
                stats.argument_size_in_bytes,
                stats.output_size_in_bytes,
                stats.alias_size_in_bytes,
                stats.temp_size_in_bytes,
                stats.host_generated_code_size_in_bytes,
                stats.host_argument_size_in_bytes,
                stats.host_output_size_in_bytes,
                stats.host_alias_size_in_bytes,
                stats.host_temp_size_in_bytes,
            ]
            .iter()
            .filter(|&&x| x == 42)
            .count();
            assert_eq!(count, 1, "Field {} should be the only one set to 42", i);
        }
    }
}

#[cfg(test)]
mod loaded_executable_types_tests {
    // Test types related to LoadedExecutable that don't require a plugin

    use pjrt_sys::PJRT_ExecuteOptions;

    use crate::ExecuteOptions;

    #[test]
    fn test_execute_options_for_loaded_executable() {
        let options = ExecuteOptions::new();
        let raw: PJRT_ExecuteOptions = (&options).into();

        // Verify default state
        assert_eq!(raw.launch_id, 0);
        assert_eq!(raw.num_non_donatable_input_indices, 0);
    }

    #[test]
    fn test_execute_options_with_launch_id() {
        let options = ExecuteOptions::new().launch_id(42);
        let raw: PJRT_ExecuteOptions = (&options).into();

        assert_eq!(raw.launch_id, 42);
    }

    #[test]
    fn test_execute_options_preserves_non_donatable_indices() {
        let options = ExecuteOptions::new().non_donatable_input_indices(vec![0, 1, 2, 3]);
        let raw: PJRT_ExecuteOptions = (&options).into();

        assert_eq!(raw.num_non_donatable_input_indices, 4);
        assert!(!raw.non_donatable_input_indices.is_null());
    }
}

#[cfg(test)]
mod execution_inputs_tests {
    use crate::ExecutionInputs;

    #[test]
    fn test_execution_inputs_empty_tuple() {
        let inputs: () = ();
        let ptrs = inputs.buffer_ptrs();

        // Empty inputs should have one device with empty buffer list
        assert_eq!(ptrs.len(), 1);
        assert!(ptrs[0].is_empty());
    }

    #[test]
    fn test_execution_inputs_empty_non_donatable() {
        let inputs: () = ();
        let indices = inputs.non_donatable_input_indices();
        assert!(indices.is_empty());
    }

    // Note: Tests with actual Buffer instances require a PJRT plugin
    // and are covered in integration tests
}
