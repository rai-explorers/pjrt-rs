//! Unit Tests for Executable Module
//!
//! These tests verify the types in the executable and loaded_executable modules:
//! - `CompiledMemoryStats`: Memory usage statistics
//! - `SerializedExecutable`: Serialized form of executables (internal)
//! - `SerializedCompileOptions`: Serialized compilation options (internal)
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
mod executable_builder_tests {
    // These tests verify the builder pattern setup without requiring a plugin

    use crate::CompileOptions;

    #[test]
    fn test_compile_options_for_executable_default() {
        let options = CompileOptions::default();
        let proto = options.proto();
        // Verify default executable build options are set
        assert!(proto.executable_build_options.is_some());
    }

    #[test]
    fn test_compile_options_for_executable_encode() {
        let options = CompileOptions::new();
        let encoded = options.encode();
        // Should produce non-empty bytes
        assert!(!encoded.is_empty());
    }

    #[test]
    fn test_compile_options_for_executable_clone() {
        let options = CompileOptions::new();
        let cloned = options.clone();

        // Both should produce the same encoding
        assert_eq!(options.encode(), cloned.encode());
    }

    #[test]
    fn test_compile_options_for_executable_debug() {
        let options = CompileOptions::new();
        let debug_str = format!("{:?}", options);
        assert!(debug_str.contains("CompileOptions"));
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
    use pjrt_sys::PJRT_Buffer;

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

    #[test]
    fn test_mock_buffer_ptr_structure() {
        // Verify the expected structure of buffer pointers
        // This test documents the expected layout without actual buffers

        // For a single device with 3 inputs:
        // buffer_ptrs should return: [[ptr1, ptr2, ptr3]]

        // For 2 devices with 3 inputs each:
        // buffer_ptrs should return: [[ptr1, ptr2, ptr3], [ptr4, ptr5, ptr6]]

        // The structure is: Vec<Vec<*mut PJRT_Buffer>>
        // - Outer Vec: one entry per device
        // - Inner Vec: one entry per input buffer for that device

        let _dummy: Vec<Vec<*mut PJRT_Buffer>> = vec![vec![]];
    }
}

#[cfg(test)]
mod program_format_tests {
    use crate::{Program, ProgramFormat};

    #[test]
    fn test_program_format_mlir() {
        let format = ProgramFormat::MLIR;
        let debug_str = format!("{:?}", format);
        assert!(debug_str.contains("MLIR"));
    }

    #[test]
    fn test_program_format_hlo() {
        let format = ProgramFormat::HLO;
        let debug_str = format!("{:?}", format);
        assert!(debug_str.contains("HLO"));
    }

    #[test]
    fn test_program_format_as_str() {
        assert_eq!(ProgramFormat::MLIR.as_str(), "mlir");
        assert_eq!(ProgramFormat::HLO.as_str(), "hlo");
    }

    #[test]
    fn test_program_format_as_bytes() {
        assert_eq!(ProgramFormat::MLIR.as_bytes(), b"mlir");
        assert_eq!(ProgramFormat::HLO.as_bytes(), b"hlo");
    }

    #[test]
    fn test_program_format_try_from_str() {
        assert_eq!(
            ProgramFormat::try_from("mlir").unwrap(),
            ProgramFormat::MLIR
        );
        assert_eq!(ProgramFormat::try_from("hlo").unwrap(), ProgramFormat::HLO);
    }

    #[test]
    fn test_program_format_try_from_str_invalid() {
        assert!(ProgramFormat::try_from("invalid").is_err());
        assert!(ProgramFormat::try_from("").is_err());
    }

    #[test]
    fn test_program_creation_mlir() {
        let mlir_code = b"module {}".to_vec();
        let program = Program::new(ProgramFormat::MLIR, mlir_code.clone());

        assert_eq!(program.format(), ProgramFormat::MLIR);
        assert_eq!(program.code(), &mlir_code);
    }

    #[test]
    fn test_program_creation_empty() {
        let program = Program::new(ProgramFormat::MLIR, vec![]);
        assert_eq!(program.format(), ProgramFormat::MLIR);
        assert!(program.code().is_empty());
    }

    #[test]
    fn test_program_format_clone() {
        let format = ProgramFormat::MLIR;
        let cloned = format;
        assert_eq!(format, cloned);
    }

    #[test]
    fn test_program_format_eq() {
        assert_eq!(ProgramFormat::MLIR, ProgramFormat::MLIR);
        assert_eq!(ProgramFormat::HLO, ProgramFormat::HLO);
        assert_ne!(ProgramFormat::MLIR, ProgramFormat::HLO);
    }

    #[test]
    fn test_program_format_ord() {
        // Ensure ordering is consistent (implementation specific)
        let formats = [ProgramFormat::MLIR, ProgramFormat::HLO];
        let mut sorted = formats;
        sorted.sort();
        // Just ensure sorting doesn't panic and is deterministic
        let mut sorted2 = formats;
        sorted2.sort();
        assert_eq!(sorted, sorted2);
    }

    #[test]
    fn test_program_format_hash() {
        use std::collections::HashSet;

        let mut set = HashSet::new();
        set.insert(ProgramFormat::MLIR);
        set.insert(ProgramFormat::HLO);

        assert_eq!(set.len(), 2);
        assert!(set.contains(&ProgramFormat::MLIR));
        assert!(set.contains(&ProgramFormat::HLO));

        // Inserting duplicate should not change size
        set.insert(ProgramFormat::MLIR);
        assert_eq!(set.len(), 2);
    }
}

#[cfg(test)]
#[allow(clippy::useless_vec)]
mod executable_metadata_tests {
    // Tests for data structures that would be returned by executable methods

    use crate::PrimitiveType;

    #[test]
    fn test_primitive_type_for_output_types() {
        // Executables return Vec<PrimitiveType> for output types
        let output_types = vec![PrimitiveType::F32, PrimitiveType::F64, PrimitiveType::S32];

        assert_eq!(output_types.len(), 3);
        assert_eq!(output_types[0], PrimitiveType::F32);
        assert_eq!(output_types[1], PrimitiveType::F64);
        assert_eq!(output_types[2], PrimitiveType::S32);
    }

    #[test]
    fn test_output_dims_structure() {
        // Executables return Vec<Vec<i64>> for output dimensions
        let output_dims: Vec<Vec<i64>> = vec![
            vec![2, 3, 4],    // First output: shape [2, 3, 4]
            vec![10],         // Second output: shape [10] (1D)
            vec![],           // Third output: scalar (empty dims)
            vec![5, 5, 5, 5], // Fourth output: 4D tensor
        ];

        assert_eq!(output_dims.len(), 4);
        assert_eq!(output_dims[0], vec![2, 3, 4]);
        assert_eq!(output_dims[1], vec![10]);
        assert!(output_dims[2].is_empty());
        assert_eq!(output_dims[3].len(), 4);
    }

    #[test]
    fn test_output_memory_kinds_structure() {
        // Executables return Vec<Cow<'_, str>> for memory kinds
        let memory_kinds: Vec<String> = vec![
            "device".to_string(),
            "host".to_string(),
            "pinned".to_string(),
        ];

        assert_eq!(memory_kinds.len(), 3);
        assert!(memory_kinds.iter().any(|k| k == "device"));
        assert!(memory_kinds.iter().any(|k| k == "host"));
    }

    #[test]
    fn test_cost_analysis_structure() {
        use crate::NamedValueMap;

        // Executables return NamedValueMap for cost analysis
        let cost_analysis = NamedValueMap::new();
        // Cost analysis typically contains entries like:
        // - "flops": number of floating point operations
        // - "transcendentals": number of transcendental operations
        // - "bytes_accessed": memory traffic

        // Verify we can work with the structure
        assert!(cost_analysis.get("non_existent").is_none());
    }
}

#[cfg(test)]
#[allow(clippy::approx_constant)]
mod named_value_tests {
    use crate::named_value::Value;
    use crate::{NamedValue, NamedValueMap};

    #[test]
    fn test_named_value_i64() {
        let nv = NamedValue::i64("test", 42);
        assert_eq!(nv.name, "test");
        assert!(matches!(nv.value, Value::I64(42)));
    }

    #[test]
    fn test_named_value_f32() {
        let nv = NamedValue::f32("pi", 3.14);
        assert_eq!(nv.name, "pi");
        if let Value::F32(v) = nv.value {
            assert!((v - 3.14).abs() < 0.001);
        } else {
            panic!("Expected F32 value");
        }
    }

    #[test]
    fn test_named_value_string() {
        let nv = NamedValue::string("key", "value");
        assert_eq!(nv.name, "key");
        assert!(matches!(nv.value, Value::String(ref s) if s == "value"));
    }

    #[test]
    fn test_named_value_bool() {
        let nv = NamedValue::bool("flag", true);
        assert_eq!(nv.name, "flag");
        assert!(matches!(nv.value, Value::Bool(true)));
    }

    #[test]
    fn test_named_value_i64_list() {
        let nv = NamedValue::i64_list("dims", vec![1, 2, 3]);
        assert_eq!(nv.name, "dims");
        assert!(matches!(nv.value, Value::I64List(ref v) if v == &[1, 2, 3]));
    }

    #[test]
    fn test_named_value_new() {
        let nv = NamedValue::new("custom", Value::I64(100));
        assert_eq!(nv.name, "custom");
        assert!(matches!(nv.value, Value::I64(100)));
    }

    #[test]
    fn test_named_value_debug() {
        let nv = NamedValue::i64("test", 42);
        let debug_str = format!("{:?}", nv);
        assert!(debug_str.contains("test"));
        assert!(debug_str.contains("42"));
    }

    #[test]
    fn test_named_value_clone() {
        let nv = NamedValue::string("key", "value");
        let cloned = nv.clone();
        assert_eq!(nv, cloned);
    }

    #[test]
    fn test_named_value_map_new() {
        let map = NamedValueMap::new();
        assert!(map.get("any").is_none());
    }

    #[test]
    fn test_named_value_map_default() {
        let map = NamedValueMap::default();
        assert!(map.get("any").is_none());
    }

    #[test]
    fn test_named_value_map_from_vec() {
        let values = vec![NamedValue::i64("a", 1), NamedValue::string("b", "two")];
        let map = NamedValueMap::from(values);

        assert!(matches!(map.get("a"), Some(Value::I64(1))));
        assert!(matches!(map.get("b"), Some(Value::String(ref s)) if s == "two"));
        assert!(map.get("c").is_none());
    }

    #[test]
    fn test_named_value_map_from_array() {
        let map = NamedValueMap::from([NamedValue::i64("x", 10), NamedValue::i64("y", 20)]);

        assert!(matches!(map.get("x"), Some(Value::I64(10))));
        assert!(matches!(map.get("y"), Some(Value::I64(20))));
    }

    #[test]
    fn test_named_value_map_into_inner() {
        let values = vec![NamedValue::i64("count", 42)];
        let map = NamedValueMap::from(values);
        let inner = map.into_inner();

        assert_eq!(inner.len(), 1);
        assert!(matches!(inner.get("count"), Some(Value::I64(42))));
    }

    #[test]
    fn test_named_value_map_into_vec() {
        let map = NamedValueMap::from([NamedValue::bool("flag", true)]);
        let vec = map.into_vec();

        assert_eq!(vec.len(), 1);
        assert_eq!(vec[0].name, "flag");
        assert!(matches!(vec[0].value, Value::Bool(true)));
    }

    #[test]
    fn test_named_value_map_clone() {
        let map = NamedValueMap::from([NamedValue::i64("key", 123)]);
        let cloned = map.clone();

        assert!(matches!(cloned.get("key"), Some(Value::I64(123))));
    }
}

#[cfg(test)]
mod executable_debug_format_tests {
    // Tests for Debug trait implementations
    // These verify that Debug formatting works without a plugin

    use crate::{CompileOptions, ExecutableBuildOptions};

    #[test]
    fn test_compile_options_debug_format() {
        let options = CompileOptions::new();
        let debug_str = format!("{:?}", options);

        // Should contain type name
        assert!(debug_str.contains("CompileOptions"));
    }

    #[test]
    fn test_executable_build_options_debug_format() {
        let options = ExecutableBuildOptions::new()
            .num_replicas(4)
            .num_partitions(2);
        let debug_str = format!("{:?}", options);

        // Should contain type name
        assert!(debug_str.contains("ExecutableBuildOptions"));
    }
}

#[cfg(test)]
mod executable_build_options_tests {
    use crate::ExecutableBuildOptions;

    #[test]
    fn test_default_values() {
        let options = ExecutableBuildOptions::new();
        let proto = options.proto();

        assert_eq!(proto.device_ordinal, -1); // -1 means not set
        assert_eq!(proto.num_replicas, 1);
        assert_eq!(proto.num_partitions, 1);
        assert!(!proto.use_spmd_partitioning);
    }

    #[test]
    fn test_device_ordinal() {
        let options = ExecutableBuildOptions::new().device_ordinal(2);
        assert_eq!(options.proto().device_ordinal, 2);
    }

    #[test]
    fn test_device_ordinal_zero() {
        let options = ExecutableBuildOptions::new().device_ordinal(0);
        assert_eq!(options.proto().device_ordinal, 0);
    }

    #[test]
    fn test_num_replicas() {
        let options = ExecutableBuildOptions::new().num_replicas(8);
        assert_eq!(options.proto().num_replicas, 8);
    }

    #[test]
    fn test_num_partitions() {
        let options = ExecutableBuildOptions::new().num_partitions(4);
        assert_eq!(options.proto().num_partitions, 4);
    }

    #[test]
    fn test_use_spmd_partitioning_true() {
        let options = ExecutableBuildOptions::new().use_spmd_partitioning(true);
        assert!(options.proto().use_spmd_partitioning);
    }

    #[test]
    fn test_use_spmd_partitioning_false() {
        let options = ExecutableBuildOptions::new().use_spmd_partitioning(false);
        assert!(!options.proto().use_spmd_partitioning);
    }

    #[test]
    fn test_chained_builder() {
        let options = ExecutableBuildOptions::new()
            .device_ordinal(1)
            .num_replicas(4)
            .num_partitions(2)
            .use_spmd_partitioning(true);

        let proto = options.proto();
        assert_eq!(proto.device_ordinal, 1);
        assert_eq!(proto.num_replicas, 4);
        assert_eq!(proto.num_partitions, 2);
        assert!(proto.use_spmd_partitioning);
    }

    #[test]
    fn test_proto_mut() {
        let mut options = ExecutableBuildOptions::new();
        options.proto_mut().num_replicas = 16;

        assert_eq!(options.proto().num_replicas, 16);
    }

    #[test]
    fn test_large_replica_count() {
        let options = ExecutableBuildOptions::new().num_replicas(1024);
        assert_eq!(options.proto().num_replicas, 1024);
    }

    #[test]
    fn test_large_partition_count() {
        let options = ExecutableBuildOptions::new().num_partitions(256);
        assert_eq!(options.proto().num_partitions, 256);
    }

    #[test]
    fn test_combined_replica_partition() {
        // Common SPMD configuration: 8 replicas x 4 partitions = 32 devices
        let options = ExecutableBuildOptions::new()
            .num_replicas(8)
            .num_partitions(4)
            .use_spmd_partitioning(true);

        let proto = options.proto();
        assert_eq!(proto.num_replicas * proto.num_partitions, 32);
        assert!(proto.use_spmd_partitioning);
    }
}

#[cfg(test)]
mod compile_options_tests {
    use crate::{CompileOptions, ExecutableBuildOptions};

    #[test]
    fn test_default() {
        let options = CompileOptions::default();
        assert!(options.proto().executable_build_options.is_some());
    }

    #[test]
    fn test_new() {
        let options = CompileOptions::new();
        assert!(options.proto().executable_build_options.is_some());
    }

    #[test]
    fn test_with_executable_build_options() {
        let build_options = ExecutableBuildOptions::new().num_replicas(4);
        let options = CompileOptions::new().executable_build_options(build_options);

        let proto = options.proto();
        assert!(proto.executable_build_options.is_some());
        assert_eq!(
            proto
                .executable_build_options
                .as_ref()
                .unwrap()
                .num_replicas,
            4
        );
    }

    #[test]
    fn test_with_none_executable_build_options() {
        let options = CompileOptions::new().executable_build_options(None);

        let proto = options.proto();
        assert!(proto.executable_build_options.is_none());
    }

    #[test]
    fn test_encode_produces_bytes() {
        let options = CompileOptions::new();
        let encoded = options.encode();

        // Encoded protobuf should have some content
        assert!(!encoded.is_empty());
    }

    #[test]
    fn test_encode_deterministic() {
        let options1 = CompileOptions::new();
        let options2 = CompileOptions::new();

        // Same options should produce same encoding
        assert_eq!(options1.encode(), options2.encode());
    }

    #[test]
    fn test_encode_different_for_different_options() {
        let options1 = CompileOptions::new()
            .executable_build_options(ExecutableBuildOptions::new().num_replicas(1));
        let options2 = CompileOptions::new()
            .executable_build_options(ExecutableBuildOptions::new().num_replicas(8));

        // Different options should produce different encodings
        assert_ne!(options1.encode(), options2.encode());
    }

    #[test]
    fn test_clone() {
        let original = CompileOptions::new().executable_build_options(
            ExecutableBuildOptions::new()
                .num_replicas(4)
                .num_partitions(2),
        );
        let cloned = original.clone();

        // Cloned should produce same encoding
        assert_eq!(original.encode(), cloned.encode());
    }

    #[test]
    fn test_proto_mut() {
        let mut options = CompileOptions::new();

        // Directly modify the proto
        if let Some(ref mut build_opts) = options.proto_mut().executable_build_options {
            build_opts.num_replicas = 42;
        }

        assert_eq!(
            options
                .proto()
                .executable_build_options
                .as_ref()
                .unwrap()
                .num_replicas,
            42
        );
    }
}

#[cfg(test)]
#[allow(clippy::approx_constant)]
mod value_type_tests {
    use crate::named_value::Value;

    #[test]
    fn test_value_i64() {
        let v = Value::I64(42);
        assert!(matches!(v, Value::I64(42)));
    }

    #[test]
    fn test_value_f32() {
        let v = Value::F32(3.14);
        if let Value::F32(f) = v {
            assert!((f - 3.14).abs() < 0.001);
        } else {
            panic!("Expected F32");
        }
    }
    #[test]
    fn test_value_bool() {
        let v_true = Value::Bool(true);
        let v_false = Value::Bool(false);
        assert!(matches!(v_true, Value::Bool(true)));
        assert!(matches!(v_false, Value::Bool(false)));
    }

    #[test]
    fn test_value_string() {
        let v = Value::String("hello".to_string());
        assert!(matches!(v, Value::String(ref s) if s == "hello"));
    }

    #[test]
    fn test_value_i64_list() {
        let v = Value::I64List(vec![1, 2, 3, 4, 5]);
        assert!(matches!(v, Value::I64List(ref l) if l.len() == 5));
    }

    #[test]
    fn test_value_i64_list_empty() {
        let v = Value::I64List(vec![]);
        assert!(matches!(v, Value::I64List(ref l) if l.is_empty()));
    }

    #[test]
    fn test_value_clone() {
        let v = Value::String("test".to_string());
        let cloned = v.clone();
        assert_eq!(v, cloned);
    }

    #[test]
    fn test_value_debug() {
        let v = Value::I64(123);
        let debug_str = format!("{:?}", v);
        assert!(debug_str.contains("123"));
    }

    #[test]
    fn test_value_partial_eq() {
        assert_eq!(Value::I64(1), Value::I64(1));
        assert_ne!(Value::I64(1), Value::I64(2));
        assert_ne!(Value::I64(1), Value::F32(1.0));
    }

    #[test]
    fn test_value_partial_ord() {
        // Same type values should be comparable
        assert!(Value::I64(1) < Value::I64(2));
        assert!(Value::F32(1.0) < Value::F32(2.0));
    }
}
