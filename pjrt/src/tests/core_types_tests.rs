//! Unit Tests for Core Types
//!
//! These tests verify the core types like PrimitiveType, MemoryLayout, etc.
//! They do not require a PJRT plugin to run.

#[cfg(test)]
mod unit_tests {
    use crate::{MemoryLayout, PrimitiveType};

    #[test]
    fn test_primitive_type_values() {
        // Test that primitive types have correct relationships
        // Invalid should be 0
        assert_eq!(PrimitiveType::Invalid as i32, 0);
        // Different types should have different values
        assert_ne!(PrimitiveType::F32 as i32, PrimitiveType::F64 as i32);
        assert_ne!(PrimitiveType::S32 as i32, PrimitiveType::S64 as i32);
    }

    #[test]
    fn test_primitive_type_debug() {
        let ty = PrimitiveType::F32;
        let debug_str = format!("{:?}", ty);
        assert!(debug_str.contains("F32"));
    }

    #[test]
    fn test_primitive_type_clone_and_copy() {
        let ty = PrimitiveType::F64;
        let cloned = ty;
        let copied = ty;
        assert_eq!(ty, cloned);
        assert_eq!(ty, copied);
    }

    #[test]
    fn test_memory_layout_from_strides() {
        let strides = vec![8, 4];
        let layout = MemoryLayout::from_strides(strides.clone());
        let debug_str = format!("{:?}", layout);
        // Verify layout was created successfully
        assert!(debug_str.contains("MemoryLayout"));
    }

    #[test]
    fn test_memory_layout_empty_strides() {
        // Scalar layout (empty strides)
        let layout = MemoryLayout::from_strides(vec![]);
        let debug_str = format!("{:?}", layout);
        assert!(debug_str.contains("MemoryLayout"));
    }

    #[test]
    fn test_memory_layout_clone() {
        let layout = MemoryLayout::from_strides(vec![16, 8, 4]);
        let cloned = layout.clone();
        // Both should produce same debug output
        assert_eq!(format!("{:?}", layout), format!("{:?}", cloned));
    }
}

#[cfg(test)]
mod error_tests {
    use crate::{Error, ErrorCode};

    #[test]
    fn test_error_code_values() {
        // Test known error codes
        assert_eq!(ErrorCode::Cancel as i32, 1);
        assert_eq!(ErrorCode::Unknown as i32, 2);
        assert_eq!(ErrorCode::InvalidArgument as i32, 3);
    }

    #[test]
    fn test_error_code_debug() {
        let code = ErrorCode::NotFound;
        let debug_str = format!("{:?}", code);
        assert!(debug_str.contains("NotFound"));
    }

    #[test]
    fn test_error_display() {
        let err = Error::InvalidArgument("test argument".to_string());
        let display_str = format!("{}", err);
        assert!(display_str.contains("invalid argument"));
        assert!(display_str.contains("test argument"));
    }

    #[test]
    fn test_error_null_pointer() {
        let err = Error::NullPointer;
        let display_str = format!("{}", err);
        assert!(display_str.contains("null pointer"));
    }

    #[test]
    fn test_error_no_addressable_device() {
        let err = Error::NoAddressableDevice;
        let display_str = format!("{}", err);
        assert!(display_str.contains("addressable device"));
    }

    #[test]
    fn test_error_plugin_not_found() {
        let err = Error::PluginNotFound("test_plugin.so".to_string());
        let display_str = format!("{}", err);
        assert!(display_str.contains("plugin not found"));
        assert!(display_str.contains("test_plugin.so"));
    }
}

#[cfg(test)]
mod host_buffer_tests {
    use crate::{HostBuffer, TypedHostBuffer, F32, F64, I32};

    #[test]
    fn test_typed_host_buffer_from_scalar() {
        let buffer = TypedHostBuffer::<F32>::from_scalar(42.0f32);
        assert_eq!(buffer.data().len(), 1);
        assert_eq!(buffer.data()[0], 42.0f32);
        assert!(buffer.dims().is_empty()); // Scalar has empty dims
    }

    #[test]
    fn test_typed_host_buffer_from_data() {
        let data = vec![1.0f32, 2.0, 3.0, 4.0];
        let buffer = TypedHostBuffer::<F32>::from_data(data.clone(), Some(vec![2, 2]), None);
        assert_eq!(buffer.data().len(), 4);
        assert_eq!(buffer.dims(), &[2, 2]);
    }

    #[test]
    fn test_typed_host_buffer_inferred_dims() {
        let data = vec![1.0f32, 2.0, 3.0, 4.0, 5.0, 6.0];
        let buffer = TypedHostBuffer::<F32>::from_data(data.clone(), None, None);
        // When dims not specified, should default to 1D
        assert_eq!(buffer.dims(), &[6]);
    }

    #[test]
    fn test_typed_host_buffer_debug() {
        let buffer = TypedHostBuffer::<F64>::from_scalar(1.5);
        let debug_str = format!("{:?}", buffer);
        assert!(debug_str.contains("TypedHostBuffer"));
    }

    #[test]
    fn test_host_buffer_from_typed() {
        let typed = TypedHostBuffer::<I32>::from_scalar(123);
        let buffer: HostBuffer = typed.into();
        let debug_str = format!("{:?}", buffer);
        assert!(debug_str.contains("I32"));
    }

    #[test]
    fn test_host_buffer_f32_variant() {
        let data = vec![1.0f32, 2.0];
        let typed = TypedHostBuffer::<F32>::from_data(data, None, None);
        let buffer: HostBuffer = typed.into();
        let debug_str = format!("{:?}", buffer);
        assert!(debug_str.contains("F32"));
    }
}

#[cfg(test)]
mod compile_options_tests {
    use crate::{CompileOptions, ExecutableBuildOptions};

    #[test]
    fn test_compile_options_default() {
        let options = CompileOptions::default();
        // Should create without panic
        let _proto = options.proto();
    }

    #[test]
    fn test_compile_options_new() {
        let options = CompileOptions::new();
        let debug_str = format!("{:?}", options);
        assert!(debug_str.contains("CompileOptions"));
    }

    #[test]
    fn test_executable_build_options_default() {
        let options = ExecutableBuildOptions::new();
        let proto = options.proto();
        assert_eq!(proto.device_ordinal, -1);
        assert_eq!(proto.num_partitions, 1);
        assert_eq!(proto.num_replicas, 1);
    }

    #[test]
    fn test_executable_build_options_num_replicas() {
        let options = ExecutableBuildOptions::new().num_replicas(4);
        assert_eq!(options.proto().num_replicas, 4);
    }

    #[test]
    fn test_executable_build_options_num_partitions() {
        let options = ExecutableBuildOptions::new().num_partitions(2);
        assert_eq!(options.proto().num_partitions, 2);
    }

    #[test]
    fn test_executable_build_options_device_ordinal() {
        let options = ExecutableBuildOptions::new().device_ordinal(1);
        assert_eq!(options.proto().device_ordinal, 1);
    }

    #[test]
    fn test_executable_build_options_use_spmd_partitioning() {
        let options = ExecutableBuildOptions::new().use_spmd_partitioning(true);
        assert!(options.proto().use_spmd_partitioning);
    }

    #[test]
    fn test_compile_options_with_build_options() {
        let build_opts = ExecutableBuildOptions::new()
            .num_replicas(2)
            .num_partitions(4);
        let options = CompileOptions::new().executable_build_options(build_opts);
        let proto = options.proto();
        let build = proto.executable_build_options.as_ref().unwrap();
        assert_eq!(build.num_replicas, 2);
        assert_eq!(build.num_partitions, 4);
    }

    #[test]
    fn test_compile_options_encode() {
        let options = CompileOptions::new();
        let encoded = options.encode();
        // Should produce non-empty bytes
        assert!(!encoded.is_empty());
    }
}

#[cfg(test)]
mod execute_options_tests {
    use crate::{CallLocation, ExecuteOptions};

    #[test]
    fn test_execute_options_default() {
        let options = ExecuteOptions::new();
        assert_eq!(options.get_launch_id(), 0);
        assert!(options.get_non_donatable_input_indices().is_empty());
        assert!(options.get_call_location().is_none());
    }

    #[test]
    fn test_execute_options_launch_id() {
        let options = ExecuteOptions::new().launch_id(42);
        assert_eq!(options.get_launch_id(), 42);
    }

    #[test]
    fn test_execute_options_non_donatable_indices() {
        let options = ExecuteOptions::new().non_donatable_input_indices(vec![0, 2, 5]);
        assert_eq!(options.get_non_donatable_input_indices(), &[0, 2, 5]);
    }

    #[test]
    fn test_call_location_creation() {
        let location = CallLocation::new("my_function", "test.py", 42);
        assert_eq!(location.function_name(), Some("my_function"));
        assert_eq!(location.file_name(), Some("test.py"));
        assert_eq!(location.line_number(), Some(42));
    }

    #[test]
    fn test_call_location_debug() {
        let location = CallLocation::new("func", "file.rs", 100);
        let debug_str = format!("{:?}", location);
        assert!(debug_str.contains("CallLocation"));
    }

    #[test]
    fn test_execute_options_with_call_location() {
        let location = CallLocation::new("train", "model.py", 150);
        let options = ExecuteOptions::new().call_location(location);
        let loc = options.get_call_location().unwrap();
        assert_eq!(loc.function_name(), Some("train"));
    }

    #[test]
    fn test_execute_options_chaining() {
        let options = ExecuteOptions::new()
            .launch_id(5)
            .non_donatable_input_indices(vec![1, 3])
            .call_location(CallLocation::new("f", "x.py", 1));

        assert_eq!(options.get_launch_id(), 5);
        assert_eq!(options.get_non_donatable_input_indices(), &[1, 3]);
        assert!(options.get_call_location().is_some());
    }
}

#[cfg(test)]
mod named_value_tests {
    use crate::NamedValue;

    #[test]
    fn test_named_value_string() {
        let nv = NamedValue::string("key", "value");
        assert_eq!(nv.name, "key");
    }

    #[test]
    fn test_named_value_i64() {
        let nv = NamedValue::i64("count", 42);
        assert_eq!(nv.name, "count");
    }

    #[test]
    fn test_named_value_i64_list() {
        let nv = NamedValue::i64_list("dims", vec![1, 2, 3, 4]);
        assert_eq!(nv.name, "dims");
    }

    #[test]
    fn test_named_value_f32() {
        let nv = NamedValue::f32("rate", 0.01);
        assert_eq!(nv.name, "rate");
    }

    #[test]
    fn test_named_value_bool() {
        let nv = NamedValue::bool("enabled", true);
        assert_eq!(nv.name, "enabled");
    }
}

#[cfg(test)]
mod program_tests {
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
    fn test_program_creation_mlir() {
        let code = b"module {}";
        let program = Program::new(ProgramFormat::MLIR, code);
        assert_eq!(program.format(), ProgramFormat::MLIR);
        assert_eq!(program.code(), code);
    }

    #[test]
    fn test_program_creation_hlo() {
        let code = b"HloModule test";
        let program = Program::new(ProgramFormat::HLO, code);
        assert_eq!(program.format(), ProgramFormat::HLO);
    }
}

#[cfg(test)]
mod device_assignment_tests {
    use crate::DeviceAssignment;

    #[test]
    fn test_device_assignment_creation() {
        let assignment = DeviceAssignment::new(2, 2, vec![0, 1, 2, 3]);
        assert_eq!(assignment.num_replicas(), 2);
        assert_eq!(assignment.num_partitions(), 2);
    }

    #[test]
    fn test_device_assignment_multi_device() {
        let assignment = DeviceAssignment::new(4, 2, vec![0, 1, 2, 3, 4, 5, 6, 7]);
        assert_eq!(assignment.num_replicas(), 4);
        assert_eq!(assignment.num_partitions(), 2);
    }

    #[test]
    fn test_device_assignment_single() {
        let assignment = DeviceAssignment::new(1, 1, vec![0]);
        assert_eq!(assignment.num_replicas(), 1);
        assert_eq!(assignment.num_partitions(), 1);
    }

    #[test]
    fn test_device_assignment_lookup_logical_id() {
        let assignment = DeviceAssignment::new(2, 2, vec![0, 1, 2, 3]);
        let logical_id = assignment.lookup_logical_id(2).unwrap();
        assert_eq!(logical_id.replica_id, 1);
        assert_eq!(logical_id.partition_id, 0);
    }
}

#[cfg(test)]
mod buffer_shape_tests {
    use crate::{BufferShape, MemoryLayout, PrimitiveType};

    #[test]
    fn test_buffer_shape_creation() {
        let shape = BufferShape::new(vec![2, 3, 4], PrimitiveType::F32);
        assert_eq!(shape.dims(), &[2, 3, 4]);
        assert_eq!(shape.element_type(), PrimitiveType::F32);
        assert!(shape.layout().is_none());
    }

    #[test]
    fn test_buffer_shape_with_layout() {
        let layout = MemoryLayout::from_strides(vec![48, 16, 4]);
        let shape = BufferShape::new(vec![2, 3, 4], PrimitiveType::F32).with_layout(layout);
        assert!(shape.layout().is_some());
    }

    #[test]
    fn test_buffer_shape_debug() {
        let shape = BufferShape::new(vec![10, 20], PrimitiveType::F64);
        let debug_str = format!("{:?}", shape);
        assert!(debug_str.contains("BufferShape"));
        assert!(debug_str.contains("F64"));
    }

    #[test]
    fn test_buffer_shape_scalar() {
        let shape = BufferShape::new(vec![], PrimitiveType::S32);
        assert!(shape.dims().is_empty());
    }
}
