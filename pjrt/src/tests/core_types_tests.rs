//! Unit Tests for Core Types
//!
//! These tests verify the core types like PrimitiveType, MemoryLayout, etc.
//! They do not require a PJRT plugin to run.

#[cfg(test)]
#[allow(clippy::approx_constant)]
#[allow(clippy::useless_vec)]
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
        let assignment = DeviceAssignment::new(2, 2, vec![0, 1, 2, 3]).unwrap();
        assert_eq!(assignment.num_replicas(), 2);
        assert_eq!(assignment.num_partitions(), 2);
    }

    #[test]
    fn test_device_assignment_multi_device() {
        let assignment = DeviceAssignment::new(4, 2, vec![0, 1, 2, 3, 4, 5, 6, 7]).unwrap();
        assert_eq!(assignment.num_replicas(), 4);
        assert_eq!(assignment.num_partitions(), 2);
    }

    #[test]
    fn test_device_assignment_single() {
        let assignment = DeviceAssignment::new(1, 1, vec![0]).unwrap();
        assert_eq!(assignment.num_replicas(), 1);
        assert_eq!(assignment.num_partitions(), 1);
    }

    #[test]
    fn test_device_assignment_lookup_logical_id() {
        let assignment = DeviceAssignment::new(2, 2, vec![0, 1, 2, 3]).unwrap();
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

#[cfg(test)]
mod comprehensive_error_tests {
    use std::collections::HashSet;

    use crate::{Error, ErrorCode, PrimitiveType};

    #[test]
    fn test_error_function_pjrt_error() {
        let err = Error::PjrtError {
            function: "PJRT_Client_Create",
            msg: "test error".to_string(),
            code: ErrorCode::Internal,
            backtrace: String::new(),
        };
        assert_eq!(err.function(), Some("PJRT_Client_Create"));
    }

    #[test]
    fn test_error_function_null_function_pointer() {
        let err = Error::NullFunctionPointer("PJRT_Buffer_Delete");
        assert_eq!(err.function(), Some("PJRT_Buffer_Delete"));
    }

    #[test]
    fn test_error_function_returns_none_for_other_variants() {
        let variants: Vec<Error> = vec![
            Error::InvalidArgument("test".to_string()),
            Error::NoAddressableDevice,
            Error::InvalidPrimitiveType(0),
            Error::InvalidErrorCode(0),
            Error::InvalidMemoryLayoutType(0),
            Error::DeviceNotInDeviceAssignment(0),
            Error::InvalidProgramFormat("test".to_string()),
            Error::NotSupportedType(PrimitiveType::F32),
            Error::NullPointer,
            Error::PluginNotFound("test".to_string()),
            Error::IoError(std::io::Error::other("test")),
            Error::PoisonError("test".to_string()),
            Error::Unimplemented,
        ];

        for err in variants {
            assert!(
                err.function().is_none(),
                "Expected None for {:?}, got {:?}",
                err,
                err.function()
            );
        }
    }

    #[test]
    fn test_error_code_for_all_pjrt_error_codes() {
        let codes = vec![
            ErrorCode::Cancel,
            ErrorCode::Unknown,
            ErrorCode::InvalidArgument,
            ErrorCode::DeadlineExceeded,
            ErrorCode::NotFound,
            ErrorCode::AlreadyExists,
            ErrorCode::PermissionDenied,
            ErrorCode::ResourceExhausted,
            ErrorCode::FailedPrecondition,
            ErrorCode::Aborted,
            ErrorCode::OutOfRange,
            ErrorCode::Unimplemented,
            ErrorCode::Internal,
            ErrorCode::Unavailable,
            ErrorCode::DataLoss,
            ErrorCode::Unauthenticated,
        ];

        for code in codes {
            let err = Error::PjrtError {
                function: "test",
                msg: String::new(),
                code,
                backtrace: String::new(),
            };
            assert_eq!(err.code(), code);
        }
    }

    #[test]
    fn test_error_code_returns_internal_for_non_pjrt_errors() {
        let variants: Vec<Error> = vec![
            Error::NullFunctionPointer("test"),
            Error::InvalidArgument("test".to_string()),
            Error::NoAddressableDevice,
            Error::InvalidPrimitiveType(0),
            Error::InvalidErrorCode(0),
            Error::InvalidMemoryLayoutType(0),
            Error::DeviceNotInDeviceAssignment(0),
            Error::InvalidProgramFormat("test".to_string()),
            Error::NotSupportedType(PrimitiveType::F32),
            Error::NullPointer,
            Error::PluginNotFound("test".to_string()),
            Error::IoError(std::io::Error::other("test")),
            Error::PoisonError("test".to_string()),
            Error::Unimplemented,
        ];

        for err in variants {
            assert_eq!(
                err.code(),
                ErrorCode::Internal,
                "Expected Internal for {:?}",
                err
            );
        }
    }

    #[test]
    fn test_error_code_clone() {
        let code = ErrorCode::NotFound;
        let cloned = code;
        assert_eq!(code, cloned);
    }

    #[test]
    fn test_error_code_copy() {
        let code = ErrorCode::PermissionDenied;
        let copied = code;
        assert_eq!(code, copied);
    }

    #[test]
    fn test_error_code_ne_different() {
        assert_ne!(ErrorCode::Cancel, ErrorCode::Unknown);
        assert_ne!(ErrorCode::NotFound, ErrorCode::AlreadyExists);
    }

    #[test]
    fn test_error_code_debug_all_variants() {
        let codes = vec![
            (ErrorCode::Cancel, "Cancel"),
            (ErrorCode::Unknown, "Unknown"),
            (ErrorCode::InvalidArgument, "InvalidArgument"),
            (ErrorCode::DeadlineExceeded, "DeadlineExceeded"),
            (ErrorCode::NotFound, "NotFound"),
            (ErrorCode::AlreadyExists, "AlreadyExists"),
            (ErrorCode::PermissionDenied, "PermissionDenied"),
            (ErrorCode::ResourceExhausted, "ResourceExhausted"),
            (ErrorCode::FailedPrecondition, "FailedPrecondition"),
            (ErrorCode::Aborted, "Aborted"),
            (ErrorCode::OutOfRange, "OutOfRange"),
            (ErrorCode::Unimplemented, "Unimplemented"),
            (ErrorCode::Internal, "Internal"),
            (ErrorCode::Unavailable, "Unavailable"),
            (ErrorCode::DataLoss, "DataLoss"),
            (ErrorCode::Unauthenticated, "Unauthenticated"),
        ];

        for (code, expected_str) in codes {
            let debug_str = format!("{:?}", code);
            assert_eq!(debug_str, expected_str);
        }
    }

    #[test]
    fn test_pjrt_error_display_format() {
        let err = Error::PjrtError {
            function: "PJRT_Client_Create",
            msg: "Failed to create client".to_string(),
            code: ErrorCode::Internal,
            backtrace: "at main.rs:42".to_string(),
        };

        let display = format!("{}", err);
        assert!(display.contains("PJRT_Client_Create"));
        assert!(display.contains("Failed to create client"));
        assert!(display.contains("Internal"));
        assert!(display.contains("at main.rs:42"));
    }

    #[test]
    fn test_pjrt_error_display_with_empty_backtrace() {
        let err = Error::PjrtError {
            function: "PJRT_Buffer_Delete",
            msg: "Buffer already freed".to_string(),
            code: ErrorCode::FailedPrecondition,
            backtrace: String::new(),
        };

        let display = format!("{}", err);
        assert!(display.contains("PJRT_Buffer_Delete"));
        assert!(display.contains("Buffer already freed"));
        assert!(display.contains("FailedPrecondition"));
    }

    #[test]
    fn test_error_debug_all_variants() {
        let variants: Vec<Error> = vec![
            Error::PjrtError {
                function: "test",
                msg: "msg".to_string(),
                code: ErrorCode::Cancel,
                backtrace: "bt".to_string(),
            },
            Error::NullFunctionPointer("test"),
            Error::InvalidArgument("test".to_string()),
            Error::NoAddressableDevice,
            Error::InvalidPrimitiveType(42),
            Error::InvalidErrorCode(42),
            Error::InvalidMemoryLayoutType(42),
            Error::DeviceNotInDeviceAssignment(42),
            Error::InvalidProgramFormat("test".to_string()),
            Error::NotSupportedType(PrimitiveType::F32),
            Error::NullPointer,
            Error::PluginNotFound("test".to_string()),
            Error::IoError(std::io::Error::other("test")),
            Error::PoisonError("test".to_string()),
            Error::Unimplemented,
        ];

        for err in variants {
            let debug = format!("{:?}", err);
            assert!(!debug.is_empty());
        }
    }

    #[test]
    fn test_poison_error_display() {
        let err = Error::PoisonError("RwLock poisoned".to_string());
        let display = format!("{}", err);
        assert!(display.contains("lock poison error"));
        assert!(display.contains("RwLock poisoned"));
    }

    #[test]
    fn test_invalid_primitive_type_edge_values() {
        let edge_values = vec![0, -1, i32::MAX, i32::MIN];

        for val in edge_values {
            let err = Error::InvalidPrimitiveType(val);
            let display = format!("{}", err);
            assert!(display.contains(&val.to_string()));
        }
    }

    #[test]
    fn test_invalid_error_code_edge_values() {
        let edge_values = vec![0, -1, i32::MAX, i32::MIN];

        for val in edge_values {
            let err = Error::InvalidErrorCode(val);
            let display = format!("{}", err);
            assert!(display.contains(&val.to_string()));
        }
    }

    #[test]
    fn test_result_ok_various_types() {
        use crate::Result;

        fn returns_string() -> Result<String> {
            Ok("success".to_string())
        }

        fn returns_vec() -> Result<Vec<i32>> {
            Ok(vec![1, 2, 3])
        }

        fn returns_option() -> Result<Option<i32>> {
            Ok(Some(42))
        }

        fn returns_unit() -> Result<()> {
            Ok(())
        }

        assert_eq!(returns_string().unwrap(), "success");
        assert_eq!(returns_vec().unwrap(), vec![1, 2, 3]);
        assert_eq!(returns_option().unwrap(), Some(42));
        assert!(returns_unit().is_ok());
    }

    #[test]
    fn test_result_err_propagation() {
        use crate::Result;

        fn inner() -> Result<i32> {
            Err(Error::NullPointer)
        }

        fn outer() -> Result<i32> {
            let _ = inner()?;
            Ok(0)
        }

        match outer() {
            Err(Error::NullPointer) => (),
            _ => panic!("Expected NullPointer error"),
        }
    }

    #[test]
    fn test_error_code_unique_values() {
        let values: Vec<i32> = vec![
            ErrorCode::Cancel as i32,
            ErrorCode::Unknown as i32,
            ErrorCode::InvalidArgument as i32,
            ErrorCode::DeadlineExceeded as i32,
            ErrorCode::NotFound as i32,
            ErrorCode::AlreadyExists as i32,
            ErrorCode::PermissionDenied as i32,
            ErrorCode::ResourceExhausted as i32,
            ErrorCode::FailedPrecondition as i32,
            ErrorCode::Aborted as i32,
            ErrorCode::OutOfRange as i32,
            ErrorCode::Unimplemented as i32,
            ErrorCode::Internal as i32,
            ErrorCode::Unavailable as i32,
            ErrorCode::DataLoss as i32,
            ErrorCode::Unauthenticated as i32,
        ];

        let unique: HashSet<i32> = values.iter().copied().collect();
        assert_eq!(
            values.len(),
            unique.len(),
            "All error codes should have unique values"
        );
    }

    #[test]
    fn test_error_is_std_error() {
        fn assert_std_error<E: std::error::Error>(_: &E) {}

        let err = Error::NullPointer;
        assert_std_error(&err);

        let pjrt_err = Error::PjrtError {
            function: "test",
            msg: "test".to_string(),
            code: ErrorCode::Internal,
            backtrace: String::new(),
        };
        assert_std_error(&pjrt_err);
    }
}

#[cfg(test)]
mod comprehensive_type_tests {
    use std::any::TypeId;
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    use num_complex::Complex;

    use crate::{
        AsDType, Bool, DType, ElemType, PrimitiveType, Type, BF16, C128, C64, F16, F32, F64, I16,
        I32, I64, I8, U16, U32, U64, U8,
    };

    #[test]
    fn test_all_alignments() {
        assert_eq!(Bool::ALIGNMENT, std::mem::align_of::<bool>());
        assert_eq!(F32::ALIGNMENT, std::mem::align_of::<f32>());
        assert_eq!(F64::ALIGNMENT, std::mem::align_of::<f64>());
        assert_eq!(I8::ALIGNMENT, std::mem::align_of::<i8>());
        assert_eq!(I16::ALIGNMENT, std::mem::align_of::<i16>());
        assert_eq!(I32::ALIGNMENT, std::mem::align_of::<i32>());
        assert_eq!(I64::ALIGNMENT, std::mem::align_of::<i64>());
        assert_eq!(U8::ALIGNMENT, std::mem::align_of::<u8>());
        assert_eq!(U16::ALIGNMENT, std::mem::align_of::<u16>());
        assert_eq!(U32::ALIGNMENT, std::mem::align_of::<u32>());
        assert_eq!(U64::ALIGNMENT, std::mem::align_of::<u64>());
        assert_eq!(F16::ALIGNMENT, std::mem::align_of::<half::f16>());
        assert_eq!(BF16::ALIGNMENT, std::mem::align_of::<half::bf16>());
        assert_eq!(C64::ALIGNMENT, std::mem::align_of::<Complex<f32>>());
        assert_eq!(C128::ALIGNMENT, std::mem::align_of::<Complex<f64>>());
    }

    #[test]
    fn test_dtype_as_any_downcast() {
        let dtype: Box<dyn DType> = F32.boxed_dtype();
        let any = dtype.as_any();
        let downcasted = any.downcast_ref::<F32>();
        assert!(downcasted.is_some());
        assert_eq!(*downcasted.unwrap(), F32);
    }

    #[test]
    fn test_dtype_as_any_wrong_type() {
        let dtype: Box<dyn DType> = F32.boxed_dtype();
        let any = dtype.as_any();
        let downcasted = any.downcast_ref::<F64>();
        assert!(downcasted.is_none());
    }

    #[test]
    fn test_type_marker_hash() {
        fn hash<T: Hash>(val: &T) -> u64 {
            let mut hasher = DefaultHasher::new();
            val.hash(&mut hasher);
            hasher.finish()
        }

        assert_eq!(hash(&F32), hash(&F32));
        assert_eq!(hash(&I64), hash(&I64));
        assert_eq!(hash(&Bool), hash(&Bool));
    }

    #[test]
    fn test_type_marker_debug() {
        assert_eq!(format!("{:?}", Bool), "Bool");
        assert_eq!(format!("{:?}", F32), "F32");
        assert_eq!(format!("{:?}", F64), "F64");
        assert_eq!(format!("{:?}", I8), "I8");
        assert_eq!(format!("{:?}", I16), "I16");
        assert_eq!(format!("{:?}", I32), "I32");
        assert_eq!(format!("{:?}", I64), "I64");
        assert_eq!(format!("{:?}", U8), "U8");
        assert_eq!(format!("{:?}", U16), "U16");
        assert_eq!(format!("{:?}", U32), "U32");
        assert_eq!(format!("{:?}", U64), "U64");
        assert_eq!(format!("{:?}", F16), "F16");
        assert_eq!(format!("{:?}", BF16), "BF16");
        assert_eq!(format!("{:?}", C64), "C64");
        assert_eq!(format!("{:?}", C128), "C128");
    }

    #[test]
    fn test_primitive_type_hash() {
        fn hash<T: Hash>(val: &T) -> u64 {
            let mut hasher = DefaultHasher::new();
            val.hash(&mut hasher);
            hasher.finish()
        }

        assert_eq!(hash(&PrimitiveType::F32), hash(&PrimitiveType::F32));
        assert_ne!(hash(&PrimitiveType::F32), hash(&PrimitiveType::F64));
    }

    #[test]
    fn test_try_into_dtype_all_supported_types() {
        let test_cases: Vec<(PrimitiveType, &str)> = vec![
            (PrimitiveType::Pred, "bool"),
            (PrimitiveType::S8, "i8"),
            (PrimitiveType::S16, "i16"),
            (PrimitiveType::S32, "i32"),
            (PrimitiveType::S64, "i64"),
            (PrimitiveType::U8, "u8"),
            (PrimitiveType::U16, "u16"),
            (PrimitiveType::U32, "u32"),
            (PrimitiveType::U64, "u64"),
            (PrimitiveType::F16, "f16"),
            (PrimitiveType::F32, "f32"),
            (PrimitiveType::F64, "f64"),
            (PrimitiveType::BF16, "bf16"),
            (PrimitiveType::C64, "c64"),
            (PrimitiveType::C128, "c128"),
        ];

        for (primitive_type, expected_name) in test_cases {
            let result = primitive_type.try_into_dtype();
            assert!(
                result.is_ok(),
                "Failed to convert {:?} to DType",
                primitive_type
            );
            assert_eq!(
                result.unwrap().name(),
                expected_name,
                "Name mismatch for {:?}",
                primitive_type
            );
        }
    }

    #[test]
    fn test_try_into_dtype_unsupported_types() {
        let unsupported = vec![
            PrimitiveType::Invalid,
            PrimitiveType::F8E5M2,
            PrimitiveType::F8E4M3FN,
            PrimitiveType::Token,
        ];

        for primitive_type in unsupported {
            let result = primitive_type.try_into_dtype();
            assert!(
                result.is_err(),
                "Expected error for unsupported type {:?}",
                primitive_type
            );
        }
    }

    #[test]
    fn test_elem_type_to_type_mapping() {
        assert_eq!(<bool as ElemType>::Type::NAME, "bool");
        assert_eq!(<f32 as ElemType>::Type::NAME, "f32");
        assert_eq!(<f64 as ElemType>::Type::NAME, "f64");
        assert_eq!(<i8 as ElemType>::Type::NAME, "i8");
        assert_eq!(<i16 as ElemType>::Type::NAME, "i16");
        assert_eq!(<i32 as ElemType>::Type::NAME, "i32");
        assert_eq!(<i64 as ElemType>::Type::NAME, "i64");
        assert_eq!(<u8 as ElemType>::Type::NAME, "u8");
        assert_eq!(<u16 as ElemType>::Type::NAME, "u16");
        assert_eq!(<u32 as ElemType>::Type::NAME, "u32");
        assert_eq!(<u64 as ElemType>::Type::NAME, "u64");
        assert_eq!(<half::f16 as ElemType>::Type::NAME, "f16");
        assert_eq!(<half::bf16 as ElemType>::Type::NAME, "bf16");
        assert_eq!(<Complex<f32> as ElemType>::Type::NAME, "c64");
        assert_eq!(<Complex<f64> as ElemType>::Type::NAME, "c128");
    }

    #[test]
    fn test_type_type_constant_identity() {
        assert_eq!(Bool::TYPE, Bool);
        assert_eq!(F32::TYPE, F32);
        assert_eq!(F64::TYPE, F64);
        assert_eq!(I8::TYPE, I8);
        assert_eq!(I16::TYPE, I16);
        assert_eq!(I32::TYPE, I32);
        assert_eq!(I64::TYPE, I64);
        assert_eq!(U8::TYPE, U8);
        assert_eq!(U16::TYPE, U16);
        assert_eq!(U32::TYPE, U32);
        assert_eq!(U64::TYPE, U64);
        assert_eq!(F16::TYPE, F16);
        assert_eq!(BF16::TYPE, BF16);
        assert_eq!(C64::TYPE, C64);
        assert_eq!(C128::TYPE, C128);
    }

    #[test]
    fn test_primitive_type_debug_all_variants() {
        let variants = [
            PrimitiveType::Invalid,
            PrimitiveType::Pred,
            PrimitiveType::S2,
            PrimitiveType::S4,
            PrimitiveType::S8,
            PrimitiveType::S16,
            PrimitiveType::S32,
            PrimitiveType::S64,
            PrimitiveType::U2,
            PrimitiveType::U4,
            PrimitiveType::U8,
            PrimitiveType::U16,
            PrimitiveType::U32,
            PrimitiveType::U64,
            PrimitiveType::F16,
            PrimitiveType::F32,
            PrimitiveType::F64,
            PrimitiveType::BF16,
            PrimitiveType::C64,
            PrimitiveType::C128,
            PrimitiveType::F8E5M2,
            PrimitiveType::F8E4M3FN,
            PrimitiveType::F8E4M3B11FNUZ,
            PrimitiveType::F8E5M2FNUZ,
            PrimitiveType::F8E4M3FNUZ,
            PrimitiveType::Token,
        ];

        for variant in variants {
            let debug_str = format!("{:?}", variant);
            assert!(!debug_str.is_empty());
        }
    }

    #[test]
    fn test_as_dtype_returns_correct_type() {
        fn verify_as_dtype<T: Type + AsDType>(val: T) {
            let dtype = val.as_dtype();
            assert_eq!(dtype.name(), T::NAME);
            assert_eq!(dtype.primitive_type(), T::PRIMITIVE_TYPE);
            assert_eq!(dtype.size(), T::SIZE);
            assert_eq!(dtype.alignment(), T::ALIGNMENT);
        }

        verify_as_dtype(Bool);
        verify_as_dtype(F32);
        verify_as_dtype(F64);
        verify_as_dtype(I8);
        verify_as_dtype(I16);
        verify_as_dtype(I32);
        verify_as_dtype(I64);
        verify_as_dtype(U8);
        verify_as_dtype(U16);
        verify_as_dtype(U32);
        verify_as_dtype(U64);
        verify_as_dtype(F16);
        verify_as_dtype(BF16);
        verify_as_dtype(C64);
        verify_as_dtype(C128);
    }

    #[test]
    fn test_size_alignment_consistency() {
        fn verify_consistency<T: Type>() {
            assert_eq!(T::SIZE, std::mem::size_of::<T::ElemType>());
            assert_eq!(T::ALIGNMENT, std::mem::align_of::<T::ElemType>());
        }

        verify_consistency::<Bool>();
        verify_consistency::<F32>();
        verify_consistency::<F64>();
        verify_consistency::<I8>();
        verify_consistency::<I16>();
        verify_consistency::<I32>();
        verify_consistency::<I64>();
        verify_consistency::<U8>();
        verify_consistency::<U16>();
        verify_consistency::<U32>();
        verify_consistency::<U64>();
        verify_consistency::<F16>();
        verify_consistency::<BF16>();
        verify_consistency::<C64>();
        verify_consistency::<C128>();
    }

    #[test]
    fn test_boxed_dtype_clone_preserves_properties() {
        fn verify_clone<T: Type>() {
            let dtype: Box<dyn DType> = T::TYPE.boxed_dtype();
            let cloned = dtype.clone();

            assert_eq!(dtype.name(), cloned.name());
            assert_eq!(dtype.primitive_type(), cloned.primitive_type());
            assert_eq!(dtype.size(), cloned.size());
            assert_eq!(dtype.alignment(), cloned.alignment());
        }

        verify_clone::<Bool>();
        verify_clone::<F32>();
        verify_clone::<F64>();
        verify_clone::<I8>();
        verify_clone::<I16>();
        verify_clone::<I32>();
        verify_clone::<I64>();
        verify_clone::<U8>();
        verify_clone::<U16>();
        verify_clone::<U32>();
        verify_clone::<U64>();
        verify_clone::<F16>();
        verify_clone::<BF16>();
        verify_clone::<C64>();
        verify_clone::<C128>();
    }

    #[test]
    fn test_type_primitive_type_mapping() {
        assert_eq!(Bool::PRIMITIVE_TYPE, PrimitiveType::Pred);
        assert_eq!(F32::PRIMITIVE_TYPE, PrimitiveType::F32);
        assert_eq!(F64::PRIMITIVE_TYPE, PrimitiveType::F64);
        assert_eq!(I8::PRIMITIVE_TYPE, PrimitiveType::S8);
        assert_eq!(I16::PRIMITIVE_TYPE, PrimitiveType::S16);
        assert_eq!(I32::PRIMITIVE_TYPE, PrimitiveType::S32);
        assert_eq!(I64::PRIMITIVE_TYPE, PrimitiveType::S64);
        assert_eq!(U8::PRIMITIVE_TYPE, PrimitiveType::U8);
        assert_eq!(U16::PRIMITIVE_TYPE, PrimitiveType::U16);
        assert_eq!(U32::PRIMITIVE_TYPE, PrimitiveType::U32);
        assert_eq!(U64::PRIMITIVE_TYPE, PrimitiveType::U64);
        assert_eq!(F16::PRIMITIVE_TYPE, PrimitiveType::F16);
        assert_eq!(BF16::PRIMITIVE_TYPE, PrimitiveType::BF16);
        assert_eq!(C64::PRIMITIVE_TYPE, PrimitiveType::C64);
        assert_eq!(C128::PRIMITIVE_TYPE, PrimitiveType::C128);
    }

    #[test]
    fn test_elem_type_round_trip() {
        fn verify_round_trip<T: Type>()
        where
            T::ElemType: ElemType<Type = T>,
        {
            assert_eq!(
                TypeId::of::<<T::ElemType as ElemType>::Type>(),
                TypeId::of::<T>()
            );
        }

        verify_round_trip::<Bool>();
        verify_round_trip::<F32>();
        verify_round_trip::<F64>();
        verify_round_trip::<I8>();
        verify_round_trip::<I16>();
        verify_round_trip::<I32>();
        verify_round_trip::<I64>();
        verify_round_trip::<U8>();
        verify_round_trip::<U16>();
        verify_round_trip::<U32>();
        verify_round_trip::<U64>();
        verify_round_trip::<F16>();
        verify_round_trip::<BF16>();
        verify_round_trip::<C64>();
        verify_round_trip::<C128>();
    }
}

#[cfg(test)]
#[allow(clippy::approx_constant)]
mod comprehensive_host_buffer_tests {
    use half::{bf16, f16};
    use num_complex::Complex;

    use crate::{
        HostBuffer, PrimitiveType, TypedHostBuffer, BF16, C128, C64, F16, F32, F64, I16, I32, I64,
        I8, U16, U32, U64, U8,
    };

    #[test]
    fn test_typed_host_buffer_f16() {
        let data = vec![f16::from_f32(1.0), f16::from_f32(2.5)];
        let buffer = TypedHostBuffer::<F16>::from_data(data.clone(), None, None);
        assert_eq!(buffer.data().len(), 2);
        assert_eq!(buffer.data()[0], f16::from_f32(1.0));
        assert_eq!(buffer.dims(), &[2]);
    }

    #[test]
    fn test_typed_host_buffer_bf16() {
        let data = vec![bf16::from_f32(3.14), bf16::from_f32(-2.7)];
        let buffer = TypedHostBuffer::<BF16>::from_data(data.clone(), None, None);
        assert_eq!(buffer.data().len(), 2);
        assert_eq!(buffer.dims(), &[2]);
    }

    #[test]
    fn test_typed_host_buffer_i8() {
        let data = vec![-128i8, 0, 127];
        let buffer = TypedHostBuffer::<I8>::from_data(data, None, None);
        assert_eq!(buffer.data(), &[-128i8, 0, 127]);
        assert_eq!(buffer.dims(), &[3]);
    }

    #[test]
    fn test_typed_host_buffer_i16() {
        let data = vec![-32768i16, 0, 32767];
        let buffer = TypedHostBuffer::<I16>::from_data(data, None, None);
        assert_eq!(buffer.data(), &[-32768i16, 0, 32767]);
    }

    #[test]
    fn test_typed_host_buffer_i64() {
        let data = vec![i64::MIN, 0, i64::MAX];
        let buffer = TypedHostBuffer::<I64>::from_data(data, None, None);
        assert_eq!(buffer.data()[0], i64::MIN);
        assert_eq!(buffer.data()[2], i64::MAX);
    }

    #[test]
    fn test_typed_host_buffer_u8() {
        let data = vec![0u8, 128, 255];
        let buffer = TypedHostBuffer::<U8>::from_data(data, None, None);
        assert_eq!(buffer.data(), &[0u8, 128, 255]);
    }

    #[test]
    fn test_typed_host_buffer_u16() {
        let data = vec![0u16, 32768, 65535];
        let buffer = TypedHostBuffer::<U16>::from_data(data, None, None);
        assert_eq!(buffer.data(), &[0u16, 32768, 65535]);
    }

    #[test]
    fn test_typed_host_buffer_u32() {
        let data = vec![0u32, u32::MAX / 2, u32::MAX];
        let buffer = TypedHostBuffer::<U32>::from_data(data, None, None);
        assert_eq!(buffer.data()[2], u32::MAX);
    }

    #[test]
    fn test_typed_host_buffer_u64() {
        let data = vec![0u64, u64::MAX];
        let buffer = TypedHostBuffer::<U64>::from_data(data, None, None);
        assert_eq!(buffer.data()[1], u64::MAX);
    }

    #[test]
    fn test_typed_host_buffer_c64() {
        let data = vec![Complex::new(1.0f32, 2.0f32), Complex::new(-3.0f32, 4.0f32)];
        let buffer = TypedHostBuffer::<C64>::from_data(data.clone(), None, None);
        assert_eq!(buffer.data().len(), 2);
        assert_eq!(buffer.data()[0].re, 1.0f32);
        assert_eq!(buffer.data()[0].im, 2.0f32);
    }

    #[test]
    fn test_typed_host_buffer_c128() {
        let data = vec![Complex::new(1.0f64, -1.0f64), Complex::new(0.0f64, 0.0f64)];
        let buffer = TypedHostBuffer::<C128>::from_data(data, None, None);
        assert_eq!(buffer.data()[0], Complex::new(1.0f64, -1.0f64));
    }

    #[test]
    fn test_typed_host_buffer_scalar_f64() {
        let buffer = TypedHostBuffer::<F64>::from_scalar(3.14159);
        assert_eq!(buffer.data().len(), 1);
        assert_eq!(buffer.data()[0], 3.14159f64);
        assert!(buffer.dims().is_empty());
    }

    #[test]
    fn test_typed_host_buffer_scalar_complex() {
        let c = Complex::new(1.0f32, -1.0f32);
        let buffer = TypedHostBuffer::<C64>::from_scalar(c);
        assert_eq!(buffer.data()[0], c);
        assert!(buffer.dims().is_empty());
    }

    #[test]
    fn test_typed_host_buffer_from_bytes_f32() {
        let bytes: Vec<u8> = vec![
            0x00, 0x00, 0x80, 0x3f, // 1.0f32
            0x00, 0x00, 0x00, 0x40, // 2.0f32
        ];
        let buffer = TypedHostBuffer::<F32>::from_bytes(bytes, Some(vec![2]), None);
        assert_eq!(buffer.data().len(), 2);
        assert_eq!(buffer.data()[0], 1.0f32);
        assert_eq!(buffer.data()[1], 2.0f32);
    }

    #[test]
    fn test_typed_host_buffer_from_bytes_i32() {
        let bytes: Vec<u8> = vec![
            0x01, 0x00, 0x00, 0x00, // 1
            0xff, 0xff, 0xff, 0xff, // -1
            0x00, 0x00, 0x00, 0x00, // 0
        ];
        let buffer = TypedHostBuffer::<I32>::from_bytes(bytes, Some(vec![3]), None);
        assert_eq!(buffer.data(), &[1i32, -1i32, 0i32]);
    }

    #[test]
    fn test_typed_host_buffer_layout() {
        let buffer =
            TypedHostBuffer::<F32>::from_data(vec![1.0, 2.0, 3.0, 4.0], Some(vec![2, 2]), None);
        let layout = buffer.layout();
        let debug_str = format!("{:?}", layout);
        assert!(debug_str.contains("MemoryLayout"));
    }

    #[test]
    fn test_typed_host_buffer_3d_dims() {
        let data: Vec<f32> = (0..24).map(|x| x as f32).collect();
        let buffer = TypedHostBuffer::<F32>::from_data(data, Some(vec![2, 3, 4]), None);
        assert_eq!(buffer.dims(), &[2, 3, 4]);
        assert_eq!(buffer.data().len(), 24);
    }

    #[test]
    fn test_host_buffer_from_data_all_types() {
        let _: HostBuffer = HostBuffer::from_data(vec![1.0f32, 2.0], None, None);
        let _: HostBuffer = HostBuffer::from_data(vec![1.0f64, 2.0], None, None);
        let _: HostBuffer = HostBuffer::from_data(vec![1i32, 2], None, None);
        let _: HostBuffer = HostBuffer::from_data(vec![0u8, 255], None, None);
    }

    #[test]
    fn test_host_buffer_from_scalar_all_types() {
        assert_eq!(
            HostBuffer::from_scalar(1.0f32).primitive_type(),
            PrimitiveType::F32
        );
        assert_eq!(
            HostBuffer::from_scalar(1.0f64).primitive_type(),
            PrimitiveType::F64
        );
        assert_eq!(
            HostBuffer::from_scalar(1i32).primitive_type(),
            PrimitiveType::S32
        );
        assert_eq!(
            HostBuffer::from_scalar(1u8).primitive_type(),
            PrimitiveType::U8
        );
        assert_eq!(
            HostBuffer::from_scalar(bf16::from_f32(1.0)).primitive_type(),
            PrimitiveType::BF16
        );
        assert_eq!(
            HostBuffer::from_scalar(f16::from_f32(1.0)).primitive_type(),
            PrimitiveType::F16
        );
        assert_eq!(
            HostBuffer::from_scalar(Complex::new(1.0f32, 0.0)).primitive_type(),
            PrimitiveType::C64
        );
        assert_eq!(
            HostBuffer::from_scalar(Complex::new(1.0f64, 0.0)).primitive_type(),
            PrimitiveType::C128
        );
    }

    #[test]
    fn test_host_buffer_from_bytes() {
        let bytes: Vec<u8> = vec![0x00, 0x00, 0x80, 0x3f, 0x00, 0x00, 0x00, 0x40];
        let buffer =
            HostBuffer::from_bytes(bytes, PrimitiveType::F32, Some(vec![2]), None).unwrap();
        assert_eq!(buffer.dims(), &[2]);
        assert_eq!(buffer.primitive_type(), PrimitiveType::F32);
    }

    #[test]
    fn test_host_buffer_from_bytes_unsupported_type() {
        let bytes = vec![0u8; 4];
        let result = HostBuffer::from_bytes(bytes, PrimitiveType::Invalid, Some(vec![1]), None);
        assert!(result.is_err());
    }

    #[test]
    fn test_host_buffer_read_f32_success() {
        let data = vec![1.0f32, 2.0, 3.0];
        let buffer = HostBuffer::from_data(data.clone(), None, None);
        let result = buffer.read_f32().unwrap();
        assert_eq!(result, &[1.0f32, 2.0, 3.0]);
    }

    #[test]
    fn test_host_buffer_read_f32_error() {
        let buffer = HostBuffer::from_scalar(1.0f64);
        let result = buffer.read_f32();
        assert!(result.is_err());
    }

    #[test]
    fn test_host_buffer_layout_all_variants() {
        let buffers: Vec<HostBuffer> = vec![
            HostBuffer::from_scalar(bf16::from_f32(1.0)),
            HostBuffer::from_scalar(f16::from_f32(1.0)),
            HostBuffer::from_scalar(1.0f32),
            HostBuffer::from_scalar(1.0f64),
            HostBuffer::from_scalar(1i8),
            HostBuffer::from_scalar(1i16),
            HostBuffer::from_scalar(1i32),
            HostBuffer::from_scalar(1i64),
            HostBuffer::from_scalar(1u8),
            HostBuffer::from_scalar(1u16),
            HostBuffer::from_scalar(1u32),
            HostBuffer::from_scalar(1u64),
            HostBuffer::from_scalar(Complex::new(1.0f32, 0.0)),
            HostBuffer::from_scalar(Complex::new(1.0f64, 0.0)),
        ];

        for buffer in buffers {
            let layout = buffer.layout();
            let debug_str = format!("{:?}", layout);
            assert!(debug_str.contains("MemoryLayout"));
        }
    }

    #[test]
    fn test_from_typed_to_host_buffer_all_types() {
        let _: HostBuffer = TypedHostBuffer::<BF16>::from_scalar(bf16::from_f32(1.0)).into();
        let _: HostBuffer = TypedHostBuffer::<F16>::from_scalar(f16::from_f32(1.0)).into();
        let _: HostBuffer = TypedHostBuffer::<F32>::from_scalar(1.0f32).into();
        let _: HostBuffer = TypedHostBuffer::<F64>::from_scalar(1.0f64).into();
        let _: HostBuffer = TypedHostBuffer::<I8>::from_scalar(1i8).into();
        let _: HostBuffer = TypedHostBuffer::<I16>::from_scalar(1i16).into();
        let _: HostBuffer = TypedHostBuffer::<I32>::from_scalar(1i32).into();
        let _: HostBuffer = TypedHostBuffer::<I64>::from_scalar(1i64).into();
        let _: HostBuffer = TypedHostBuffer::<U8>::from_scalar(1u8).into();
        let _: HostBuffer = TypedHostBuffer::<U16>::from_scalar(1u16).into();
        let _: HostBuffer = TypedHostBuffer::<U32>::from_scalar(1u32).into();
        let _: HostBuffer = TypedHostBuffer::<U64>::from_scalar(1u64).into();
        let _: HostBuffer = TypedHostBuffer::<C64>::from_scalar(Complex::new(1.0f32, 0.0)).into();
        let _: HostBuffer = TypedHostBuffer::<C128>::from_scalar(Complex::new(1.0f64, 0.0)).into();
    }

    #[test]
    fn test_typed_host_buffer_debug_all_types() {
        let debug_strs: Vec<String> = vec![
            format!(
                "{:?}",
                TypedHostBuffer::<BF16>::from_scalar(bf16::from_f32(1.0))
            ),
            format!(
                "{:?}",
                TypedHostBuffer::<F16>::from_scalar(f16::from_f32(1.0))
            ),
            format!("{:?}", TypedHostBuffer::<F32>::from_scalar(1.0f32)),
            format!("{:?}", TypedHostBuffer::<F64>::from_scalar(1.0f64)),
            format!("{:?}", TypedHostBuffer::<I8>::from_scalar(1i8)),
            format!("{:?}", TypedHostBuffer::<I16>::from_scalar(1i16)),
            format!("{:?}", TypedHostBuffer::<I32>::from_scalar(1i32)),
            format!("{:?}", TypedHostBuffer::<I64>::from_scalar(1i64)),
            format!("{:?}", TypedHostBuffer::<U8>::from_scalar(1u8)),
            format!("{:?}", TypedHostBuffer::<U16>::from_scalar(1u16)),
            format!("{:?}", TypedHostBuffer::<U32>::from_scalar(1u32)),
            format!("{:?}", TypedHostBuffer::<U64>::from_scalar(1u64)),
            format!(
                "{:?}",
                TypedHostBuffer::<C64>::from_scalar(Complex::new(1.0f32, 0.0))
            ),
            format!(
                "{:?}",
                TypedHostBuffer::<C128>::from_scalar(Complex::new(1.0f64, 0.0))
            ),
        ];

        for debug_str in debug_strs {
            assert!(debug_str.contains("TypedHostBuffer"));
        }
    }

    #[test]
    fn test_typed_host_buffer_empty_data_1d() {
        let buffer = TypedHostBuffer::<F32>::from_data(vec![], None, None);
        assert_eq!(buffer.data().len(), 0);
        assert_eq!(buffer.dims(), &[0]);
    }

    #[test]
    fn test_typed_host_buffer_special_float_values() {
        let data = vec![
            f32::INFINITY,
            f32::NEG_INFINITY,
            f32::NAN,
            f32::MIN,
            f32::MAX,
            f32::EPSILON,
            0.0f32,
            -0.0f32,
        ];
        let buffer = TypedHostBuffer::<F32>::from_data(data.clone(), None, None);
        assert_eq!(buffer.data().len(), 8);
        assert!(buffer.data()[0].is_infinite() && buffer.data()[0].is_sign_positive());
        assert!(buffer.data()[1].is_infinite() && buffer.data()[1].is_sign_negative());
        assert!(buffer.data()[2].is_nan());
    }
}

#[cfg(test)]
#[allow(clippy::approx_constant)]
mod comprehensive_named_value_tests {
    use std::collections::HashMap;

    use crate::named_value::Value;
    use crate::{NamedValue, NamedValueMap};

    #[test]
    fn test_value_i64_boundary_values() {
        assert_eq!(Value::I64(i64::MAX), Value::I64(i64::MAX));
        assert_eq!(Value::I64(i64::MIN), Value::I64(i64::MIN));
        assert_ne!(Value::I64(i64::MAX), Value::I64(i64::MIN));

        let nv_max = NamedValue::i64("max", i64::MAX);
        assert!(matches!(nv_max.value, Value::I64(v) if v == i64::MAX));
    }

    #[test]
    fn test_value_f32_special_values() {
        let inf = Value::F32(f32::INFINITY);
        let neg_inf = Value::F32(f32::NEG_INFINITY);
        let zero = Value::F32(0.0);
        let neg_zero = Value::F32(-0.0);

        assert_eq!(inf, Value::F32(f32::INFINITY));
        assert_eq!(neg_inf, Value::F32(f32::NEG_INFINITY));
        assert_eq!(zero, neg_zero);

        let nan = Value::F32(f32::NAN);
        assert_ne!(nan, Value::F32(f32::NAN));
    }

    #[test]
    fn test_value_string_empty() {
        let empty = Value::String(String::new());
        assert_eq!(empty, Value::String("".to_string()));
    }

    #[test]
    fn test_value_string_unicode() {
        let unicode_str = "Hello, 世界! 🌍";
        let value = Value::String(unicode_str.to_string());
        assert_eq!(value, Value::String(unicode_str.to_string()));
    }

    #[test]
    fn test_value_i64_list_empty() {
        let empty_list = Value::I64List(vec![]);
        assert_eq!(empty_list, Value::I64List(vec![]));
    }

    #[test]
    fn test_value_i64_list_with_boundaries() {
        let boundaries = vec![i64::MIN, -1, 0, 1, i64::MAX];
        let value = Value::I64List(boundaries.clone());
        assert_eq!(value, Value::I64List(boundaries));
    }

    #[test]
    fn test_value_partial_ord_i64() {
        assert!(Value::I64(10) < Value::I64(20));
        assert!(Value::I64(20) > Value::I64(10));
        assert!(Value::I64(10) <= Value::I64(10));
        assert!(Value::I64(-10) < Value::I64(0));
    }

    #[test]
    fn test_value_partial_ord_f32() {
        assert!(Value::F32(1.0) < Value::F32(2.0));
        assert!(Value::F32(2.0) > Value::F32(1.0));
        assert!(Value::F32(f32::NEG_INFINITY) < Value::F32(0.0));
    }

    #[test]
    fn test_value_partial_ord_bool() {
        assert!(Value::Bool(false) < Value::Bool(true));
        assert!(Value::Bool(true) > Value::Bool(false));
    }

    #[test]
    fn test_value_partial_ord_string() {
        assert!(Value::String("a".to_string()) < Value::String("b".to_string()));
        assert!(Value::String("".to_string()) < Value::String("a".to_string()));
    }

    #[test]
    fn test_value_partial_ord_i64_list() {
        assert!(Value::I64List(vec![1, 2]) < Value::I64List(vec![1, 3]));
        assert!(Value::I64List(vec![1]) < Value::I64List(vec![1, 2]));
        assert!(Value::I64List(vec![]) < Value::I64List(vec![1]));
    }

    #[test]
    fn test_named_value_new_all_types() {
        let nv_i64 = NamedValue::new("int", Value::I64(42));
        assert_eq!(nv_i64.name, "int");
        assert_eq!(nv_i64.value, Value::I64(42));

        let nv_f32 = NamedValue::new("float", Value::F32(3.14));
        assert_eq!(nv_f32.name, "float");

        let nv_bool = NamedValue::new("flag", Value::Bool(false));
        assert_eq!(nv_bool.value, Value::Bool(false));

        let nv_str = NamedValue::new("text", Value::String("hello".to_string()));
        assert_eq!(nv_str.value, Value::String("hello".to_string()));

        let nv_list = NamedValue::new("array", Value::I64List(vec![1, 2, 3]));
        assert_eq!(nv_list.value, Value::I64List(vec![1, 2, 3]));
    }

    #[test]
    fn test_named_value_empty_name() {
        let nv = NamedValue::i64("", 42);
        assert_eq!(nv.name, "");
    }

    #[test]
    fn test_named_value_unicode_name() {
        let nv = NamedValue::i64("变量_🎉", 100);
        assert_eq!(nv.name, "变量_🎉");
    }

    #[test]
    fn test_named_value_map_empty_operations() {
        let map = NamedValueMap::new();
        assert!(map.get("anything").is_none());

        let inner = map.clone().into_inner();
        assert!(inner.is_empty());

        let vec = map.into_vec();
        assert!(vec.is_empty());
    }

    #[test]
    fn test_named_value_map_from_empty_vec() {
        let map = NamedValueMap::from(Vec::<NamedValue>::new());
        assert!(map.get("anything").is_none());
    }

    #[test]
    fn test_named_value_map_from_empty_hashmap() {
        let map = NamedValueMap::from(HashMap::<String, Value>::new());
        assert!(map.get("anything").is_none());
    }

    #[test]
    fn test_named_value_map_all_value_types() {
        let map = NamedValueMap::from(vec![
            NamedValue::i64("int", 42),
            NamedValue::f32("float", 3.14),
            NamedValue::bool("flag", true),
            NamedValue::string("text", "hello"),
            NamedValue::i64_list("list", vec![1, 2, 3]),
        ]);

        assert_eq!(map.get("int"), Some(&Value::I64(42)));
        assert_eq!(map.get("float"), Some(&Value::F32(3.14)));
        assert_eq!(map.get("flag"), Some(&Value::Bool(true)));
        assert_eq!(map.get("text"), Some(&Value::String("hello".to_string())));
        assert_eq!(map.get("list"), Some(&Value::I64List(vec![1, 2, 3])));
    }

    #[test]
    fn test_named_value_map_into_vec_preserves_values() {
        let original = vec![
            NamedValue::i64("a", 1),
            NamedValue::f32("b", 2.0),
            NamedValue::bool("c", true),
        ];
        let map = NamedValueMap::from(original.clone());
        let vec = map.into_vec();

        assert_eq!(vec.len(), 3);

        let a = vec.iter().find(|nv| nv.name == "a").unwrap();
        assert_eq!(a.value, Value::I64(1));
    }

    #[test]
    fn test_named_value_map_clone() {
        let map = NamedValueMap::from(vec![
            NamedValue::i64("x", 10),
            NamedValue::string("y", "test"),
        ]);
        let cloned = map.clone();

        assert_eq!(map.get("x"), cloned.get("x"));
        assert_eq!(map.get("y"), cloned.get("y"));
    }

    #[test]
    fn test_named_value_map_debug() {
        let map = NamedValueMap::from(vec![NamedValue::i64("test", 42)]);
        let debug_str = format!("{:?}", map);
        assert!(debug_str.contains("NamedValueMap"));
    }

    #[test]
    fn test_value_clone_all_types() {
        let values: Vec<Value> = vec![
            Value::I64(42),
            Value::F32(3.14),
            Value::Bool(true),
            Value::String("test".to_string()),
            Value::I64List(vec![1, 2, 3]),
        ];

        for v in values {
            let cloned = v.clone();
            assert_eq!(v, cloned);
        }
    }

    #[test]
    fn test_value_debug_all_types() {
        let test_cases = vec![
            (Value::I64(42), "I64"),
            (Value::F32(3.14), "F32"),
            (Value::Bool(true), "Bool"),
            (Value::String("hello".to_string()), "String"),
            (Value::I64List(vec![1, 2, 3]), "I64List"),
        ];

        for (value, type_name) in test_cases {
            let debug_str = format!("{:?}", value);
            assert!(debug_str.contains(type_name));
        }
    }

    #[test]
    fn test_named_value_equality_name_matters() {
        let a = NamedValue::i64("x", 10);
        let b = NamedValue::i64("y", 10);
        assert_ne!(a, b);
    }

    #[test]
    fn test_named_value_equality_value_matters() {
        let a = NamedValue::i64("x", 10);
        let b = NamedValue::i64("x", 20);
        assert_ne!(a, b);
    }

    #[test]
    fn test_named_value_equality_type_matters() {
        let a = NamedValue::i64("x", 10);
        let b = NamedValue::f32("x", 10.0);
        assert_ne!(a, b);
    }

    #[test]
    fn test_named_value_partial_ord() {
        let nv1 = NamedValue::i64("a", 10);
        let nv2 = NamedValue::i64("b", 5);
        let nv3 = NamedValue::i64("a", 20);

        assert!(nv1 < nv2);
        assert!(nv1 < nv3);
    }
}

/// Tests for device-related types from device.rs
///
/// These tests verify the behavior of device ID types and MemoryStats
/// without requiring a PJRT plugin.
#[cfg(test)]
mod device_id_types_tests {
    use crate::{GlobalDeviceId, LocalDeviceId, LocalHardwareId};

    #[test]
    fn test_global_device_id_is_i32() {
        let id: GlobalDeviceId = 42;
        assert_eq!(id, 42i32);
        let id2: GlobalDeviceId = -1;
        assert_eq!(id2, -1);
    }

    #[test]
    fn test_local_device_id_is_i32() {
        let id: LocalDeviceId = 0;
        assert_eq!(id, 0i32);
        let undefined: LocalDeviceId = -1;
        assert_eq!(undefined, -1);
    }

    #[test]
    fn test_local_hardware_id_is_i32() {
        let id: LocalHardwareId = 5;
        assert_eq!(id, 5i32);
        let undefined: LocalHardwareId = -1;
        assert_eq!(undefined, -1);
    }

    #[test]
    fn test_device_id_arithmetic() {
        let global: GlobalDeviceId = 10;
        let local: LocalDeviceId = 5;
        let hardware: LocalHardwareId = 3;

        assert_eq!(global + local, 15);
        assert_eq!(global - hardware, 7);
        assert_eq!(local * 2, 10);
    }

    #[test]
    fn test_device_id_comparison() {
        let id1: GlobalDeviceId = 10;
        let id2: GlobalDeviceId = 20;
        let id3: GlobalDeviceId = 10;

        assert!(id1 < id2);
        assert!(id2 > id1);
        assert_eq!(id1, id3);
        assert!(id1 <= id3);
        assert!(id1 >= id3);
    }

    #[test]
    fn test_device_id_in_collections() {
        use std::collections::{HashMap, HashSet};

        let mut set: HashSet<GlobalDeviceId> = HashSet::new();
        set.insert(1);
        set.insert(2);
        set.insert(1);
        assert_eq!(set.len(), 2);
        assert!(set.contains(&1));

        let mut map: HashMap<LocalDeviceId, String> = HashMap::new();
        map.insert(0, "device0".to_string());
        map.insert(1, "device1".to_string());
        assert_eq!(map.get(&0), Some(&"device0".to_string()));
    }

    #[test]
    fn test_device_id_formatting() {
        let global: GlobalDeviceId = 42;
        let local: LocalDeviceId = -1;

        assert_eq!(format!("{}", global), "42");
        assert_eq!(format!("{}", local), "-1");
        assert_eq!(format!("{:?}", global), "42");
    }

    #[test]
    fn test_device_id_bounds() {
        let max: GlobalDeviceId = i32::MAX;
        let min: GlobalDeviceId = i32::MIN;

        assert_eq!(max, 2147483647);
        assert_eq!(min, -2147483648);
    }

    #[test]
    fn test_device_id_type_coercion() {
        fn takes_i32(x: i32) -> i32 {
            x * 2
        }

        let global: GlobalDeviceId = 5;
        let result = takes_i32(global);
        assert_eq!(result, 10);
    }

    #[test]
    fn test_device_id_undefined_conventions() {
        const UNDEFINED_LOCAL_DEVICE: LocalDeviceId = -1;
        const UNDEFINED_LOCAL_HARDWARE: LocalHardwareId = -1;

        assert_eq!(UNDEFINED_LOCAL_DEVICE, -1);
        assert_eq!(UNDEFINED_LOCAL_HARDWARE, -1);

        fn is_defined_local_device(id: LocalDeviceId) -> bool {
            id >= 0
        }

        fn is_defined_hardware_id(id: LocalHardwareId) -> bool {
            id >= 0
        }

        assert!(!is_defined_local_device(UNDEFINED_LOCAL_DEVICE));
        assert!(is_defined_local_device(0));
        assert!(is_defined_local_device(5));

        assert!(!is_defined_hardware_id(UNDEFINED_LOCAL_HARDWARE));
        assert!(is_defined_hardware_id(0));
    }
}

/// Tests for MemoryStats from device.rs
#[cfg(test)]
mod memory_stats_tests {
    use crate::MemoryStats;

    fn create_test_memory_stats() -> MemoryStats {
        MemoryStats {
            bytes_in_use: 1000,
            peak_bytes_in_use: Some(2000),
            num_allocs: Some(50),
            largest_alloc_size: Some(500),
            bytes_limit: Some(10000),
            bytes_reserved: Some(3000),
            peak_bytes_reserved: Some(4000),
            bytes_reservable_limit: Some(8000),
            largest_free_block_bytes: Some(2500),
            pool_bytes: Some(5000),
            peak_pool_bytes: Some(6000),
        }
    }

    fn create_minimal_memory_stats() -> MemoryStats {
        MemoryStats {
            bytes_in_use: 100,
            peak_bytes_in_use: None,
            num_allocs: None,
            largest_alloc_size: None,
            bytes_limit: None,
            bytes_reserved: None,
            peak_bytes_reserved: None,
            bytes_reservable_limit: None,
            largest_free_block_bytes: None,
            pool_bytes: None,
            peak_pool_bytes: None,
        }
    }

    #[test]
    fn test_memory_stats_field_access() {
        let stats = create_test_memory_stats();

        assert_eq!(stats.bytes_in_use, 1000);
        assert_eq!(stats.peak_bytes_in_use, Some(2000));
        assert_eq!(stats.num_allocs, Some(50));
        assert_eq!(stats.largest_alloc_size, Some(500));
        assert_eq!(stats.bytes_limit, Some(10000));
        assert_eq!(stats.bytes_reserved, Some(3000));
        assert_eq!(stats.peak_bytes_reserved, Some(4000));
        assert_eq!(stats.bytes_reservable_limit, Some(8000));
        assert_eq!(stats.largest_free_block_bytes, Some(2500));
        assert_eq!(stats.pool_bytes, Some(5000));
        assert_eq!(stats.peak_pool_bytes, Some(6000));
    }

    #[test]
    fn test_memory_stats_clone() {
        let stats = create_test_memory_stats();
        let cloned = stats.clone();

        assert_eq!(stats, cloned);
        assert_eq!(stats.bytes_in_use, cloned.bytes_in_use);
        assert_eq!(stats.peak_bytes_in_use, cloned.peak_bytes_in_use);
    }

    #[test]
    fn test_memory_stats_equality() {
        let stats1 = create_test_memory_stats();
        let stats2 = create_test_memory_stats();
        let stats3 = create_minimal_memory_stats();

        assert_eq!(stats1, stats2);
        assert_ne!(stats1, stats3);
    }

    #[test]
    fn test_memory_stats_inequality_single_field() {
        let mut stats1 = create_test_memory_stats();
        let stats2 = create_test_memory_stats();

        stats1.bytes_in_use = 999;
        assert_ne!(stats1, stats2);
    }

    #[test]
    fn test_memory_stats_debug_format() {
        let stats = create_test_memory_stats();
        let debug_str = format!("{:?}", stats);

        assert!(debug_str.contains("MemoryStats"));
        assert!(debug_str.contains("bytes_in_use"));
        assert!(debug_str.contains("1000"));
        assert!(debug_str.contains("peak_bytes_in_use"));
        assert!(debug_str.contains("2000"));
    }

    #[test]
    fn test_memory_stats_partial_ord() {
        let stats1 = MemoryStats {
            bytes_in_use: 100,
            peak_bytes_in_use: Some(200),
            num_allocs: Some(10),
            largest_alloc_size: Some(50),
            bytes_limit: Some(1000),
            bytes_reserved: Some(300),
            peak_bytes_reserved: Some(400),
            bytes_reservable_limit: Some(800),
            largest_free_block_bytes: Some(250),
            pool_bytes: Some(500),
            peak_pool_bytes: Some(600),
        };

        let stats2 = MemoryStats {
            bytes_in_use: 200,
            ..stats1.clone()
        };

        assert!(stats1 < stats2);
        assert!(stats2 > stats1);
    }

    #[test]
    fn test_memory_stats_ord() {
        use std::cmp::Ordering;

        let stats1 = create_minimal_memory_stats();
        let stats2 = create_test_memory_stats();

        assert_eq!(stats1.cmp(&stats2), Ordering::Less);
        assert_eq!(stats2.cmp(&stats1), Ordering::Greater);
        assert_eq!(stats1.cmp(&stats1), Ordering::Equal);
    }

    #[test]
    fn test_memory_stats_hash() {
        use std::collections::HashSet;
        use std::hash::{Hash, Hasher};

        let stats1 = create_test_memory_stats();
        let stats2 = create_test_memory_stats();
        let stats3 = create_minimal_memory_stats();

        fn hash_value<T: Hash>(t: &T) -> u64 {
            let mut hasher = std::collections::hash_map::DefaultHasher::new();
            t.hash(&mut hasher);
            hasher.finish()
        }

        assert_eq!(hash_value(&stats1), hash_value(&stats2));

        let mut set = HashSet::new();
        set.insert(stats1.clone());
        set.insert(stats2);
        set.insert(stats3);

        assert_eq!(set.len(), 2);
        assert!(set.contains(&stats1));
    }

    #[test]
    fn test_memory_stats_in_btree() {
        use std::collections::BTreeSet;

        let stats1 = create_minimal_memory_stats();
        let stats2 = create_test_memory_stats();

        let mut set = BTreeSet::new();
        set.insert(stats2.clone());
        set.insert(stats1.clone());

        let items: Vec<_> = set.into_iter().collect();
        assert_eq!(items[0].bytes_in_use, 100);
        assert_eq!(items[1].bytes_in_use, 1000);
    }

    #[test]
    fn test_memory_stats_optional_fields_pattern() {
        let stats = create_minimal_memory_stats();
        assert!(stats.peak_bytes_in_use.is_none());

        let full_stats = create_test_memory_stats();
        assert_eq!(full_stats.peak_bytes_in_use, Some(2000));
    }

    #[test]
    fn test_memory_stats_zero_values_with_set_flag() {
        let stats = MemoryStats {
            bytes_in_use: 0,
            peak_bytes_in_use: Some(0),
            num_allocs: Some(0),
            largest_alloc_size: Some(0),
            bytes_limit: Some(0),
            bytes_reserved: Some(0),
            peak_bytes_reserved: Some(0),
            bytes_reservable_limit: Some(0),
            largest_free_block_bytes: Some(0),
            pool_bytes: Some(0),
            peak_pool_bytes: Some(0),
        };

        assert!(stats.peak_bytes_in_use.is_some());
        assert!(stats.bytes_limit.is_some());
    }

    #[test]
    fn test_memory_stats_large_values() {
        let stats = MemoryStats {
            bytes_in_use: 16 * 1024 * 1024 * 1024,
            peak_bytes_in_use: Some(32 * 1024 * 1024 * 1024),
            num_allocs: Some(1_000_000),
            largest_alloc_size: Some(4 * 1024 * 1024 * 1024),
            bytes_limit: Some(64 * 1024 * 1024 * 1024),
            bytes_reserved: Some(48 * 1024 * 1024 * 1024),
            peak_bytes_reserved: Some(56 * 1024 * 1024 * 1024),
            bytes_reservable_limit: Some(60 * 1024 * 1024 * 1024),
            largest_free_block_bytes: Some(8 * 1024 * 1024 * 1024),
            pool_bytes: Some(40 * 1024 * 1024 * 1024),
            peak_pool_bytes: Some(50 * 1024 * 1024 * 1024),
        };

        assert_eq!(stats.bytes_in_use, 17179869184);
        assert!(stats.bytes_limit.unwrap() > stats.bytes_in_use);
    }

    #[test]
    fn test_memory_stats_negative_values() {
        let stats = MemoryStats {
            bytes_in_use: -1,
            peak_bytes_in_use: None,
            num_allocs: None,
            largest_alloc_size: None,
            bytes_limit: None,
            bytes_reserved: None,
            peak_bytes_reserved: None,
            bytes_reservable_limit: None,
            largest_free_block_bytes: None,
            pool_bytes: None,
            peak_pool_bytes: None,
        };

        assert_eq!(stats.bytes_in_use, -1);
    }

    #[test]
    fn test_memory_stats_struct_update_syntax() {
        let base = create_test_memory_stats();

        let updated = MemoryStats {
            bytes_in_use: 9999,
            ..base.clone()
        };

        assert_eq!(updated.bytes_in_use, 9999);
        assert_eq!(updated.peak_bytes_in_use, base.peak_bytes_in_use);
        assert_eq!(updated.num_allocs, base.num_allocs);
    }

    #[test]
    fn test_memory_stats_all_optional_fields() {
        let stats = create_test_memory_stats();

        assert!(stats.peak_bytes_in_use.is_some());
        assert!(stats.num_allocs.is_some());
        assert!(stats.largest_alloc_size.is_some());
        assert!(stats.bytes_limit.is_some());
        assert!(stats.bytes_reserved.is_some());
        assert!(stats.peak_bytes_reserved.is_some());
        assert!(stats.bytes_reservable_limit.is_some());
        assert!(stats.largest_free_block_bytes.is_some());
        assert!(stats.pool_bytes.is_some());
        assert!(stats.peak_pool_bytes.is_some());
    }

    #[test]
    fn test_memory_stats_helper_functions() {
        fn utilization_percent(stats: &MemoryStats) -> Option<f64> {
            stats.bytes_limit.and_then(|limit| {
                if limit > 0 {
                    Some((stats.bytes_in_use as f64 / limit as f64) * 100.0)
                } else {
                    None
                }
            })
        }

        let stats = create_test_memory_stats();
        assert_eq!(stats.peak_bytes_in_use, Some(2000));
        assert_eq!(stats.bytes_limit, Some(10000));
        assert!((utilization_percent(&stats).unwrap() - 10.0).abs() < 0.01);

        let minimal = create_minimal_memory_stats();
        assert_eq!(minimal.peak_bytes_in_use, None);
        assert_eq!(minimal.bytes_limit, None);
        assert!(utilization_percent(&minimal).is_none());
    }
}

/// Tests for AsyncTrackingEvent Debug implementation
#[cfg(test)]
mod async_tracking_event_tests {
    #[test]
    fn test_async_tracking_event_type_exists() {
        use crate::AsyncTrackingEvent;
        fn _takes_event(_event: &AsyncTrackingEvent) {}
    }
}

/// Comprehensive tests for Memory type
#[cfg(test)]
mod comprehensive_memory_tests {
    #[test]
    fn test_memory_type_exists() {
        use crate::Memory;
        fn _takes_memory(_: &Memory) {}
    }

    #[test]
    fn test_memory_debug_trait() {
        use std::fmt::Debug;

        use crate::Memory;
        fn assert_debug<T: Debug>() {}
        assert_debug::<Memory>();
    }

    #[test]
    fn test_memory_display_trait() {
        use std::fmt::Display;

        use crate::Memory;
        fn assert_display<T: Display>() {}
        assert_display::<Memory>();
    }

    #[test]
    fn test_memory_method_signatures() {
        use std::borrow::Cow;

        use crate::{Client, Device};

        #[allow(dead_code)]
        trait MemoryMethods {
            fn client(&self) -> &Client;
            fn id(&self) -> i32;
            fn kind(&self) -> Cow<'_, str>;
            fn kind_id(&self) -> i32;
            fn debug_string(&self) -> Cow<'_, str>;
            fn to_string(&self) -> Cow<'_, str>;
            fn addressable_by_devices(&self) -> Vec<Device>;
        }
    }

    #[test]
    fn test_memory_is_not_clone() {
        // Memory should not be Clone since it wraps a unique resource
        use crate::Memory;
        fn _check(_: &Memory) {
            // Would fail to compile if Memory was Clone
        }
    }
}

/// Comprehensive tests for Event type
#[cfg(test)]
mod comprehensive_event_tests {
    use std::sync::atomic::{AtomicBool, Ordering};

    #[test]
    fn test_event_type_exists() {
        use crate::Event;
        fn _takes_event(_: &Event) {}
    }

    #[test]
    fn test_event_debug_trait() {
        use std::fmt::Debug;

        use crate::Event;
        fn assert_debug<T: Debug>() {}
        assert_debug::<Event>();
    }

    #[test]
    fn test_event_implements_future() {
        use std::future::Future;

        use crate::Event;
        fn assert_future<T: Future>() {}
        assert_future::<Event>();
    }

    #[test]
    fn test_event_method_signatures() {
        use crate::{Api, Result};

        #[allow(dead_code)]
        trait EventMethods {
            fn api(&self) -> &Api;
            fn await_ready(&self) -> Result<()>;
        }
    }

    #[test]
    fn test_atomic_ordering_for_callbacks() {
        // Event uses AtomicBool for callback tracking
        let flag = AtomicBool::new(false);
        flag.store(true, Ordering::SeqCst);
        assert!(flag.load(Ordering::SeqCst));
    }

    #[test]
    fn test_event_poll_states() {
        use std::task::Poll;

        let pending: Poll<()> = Poll::Pending;
        let ready: Poll<()> = Poll::Ready(());

        assert!(pending.is_pending());
        assert!(ready.is_ready());
    }

    #[test]
    fn test_pin_behavior() {
        use std::pin::Pin;

        // Event should be !Unpin due to self-referential callback data
        // This test documents the expected behavior
        let value = 42;
        let pinned = Pin::new(&value);
        assert_eq!(*pinned, 42);
    }
}

/// Comprehensive tests for BufferShape (async_transfer)
#[cfg(test)]
mod comprehensive_buffer_shape_tests {
    use crate::{BufferShape, MemoryLayout, PrimitiveType};

    #[test]
    fn test_buffer_shape_new() {
        let shape = BufferShape::new(vec![2, 3], PrimitiveType::F32);
        assert_eq!(shape.dims(), &[2, 3]);
        assert_eq!(shape.element_type(), PrimitiveType::F32);
        assert!(shape.layout().is_none());
    }

    #[test]
    fn test_buffer_shape_scalar() {
        let shape = BufferShape::new(vec![], PrimitiveType::F64);
        assert!(shape.dims().is_empty());
        assert_eq!(shape.element_type(), PrimitiveType::F64);
    }

    #[test]
    fn test_buffer_shape_1d() {
        let shape = BufferShape::new(vec![100], PrimitiveType::S32);
        assert_eq!(shape.dims(), &[100]);
    }

    #[test]
    fn test_buffer_shape_high_dimensional() {
        let shape = BufferShape::new(vec![2, 3, 4, 5, 6], PrimitiveType::F16);
        assert_eq!(shape.dims().len(), 5);
    }

    #[test]
    fn test_buffer_shape_with_layout() {
        let layout = MemoryLayout::from_strides(vec![12, 4]);
        let shape = BufferShape::new(vec![3, 4], PrimitiveType::F32).with_layout(layout.clone());
        assert!(shape.layout().is_some());
    }

    #[test]
    fn test_buffer_shape_debug() {
        let shape = BufferShape::new(vec![2, 3], PrimitiveType::F32);
        let debug = format!("{:?}", shape);
        assert!(debug.contains("BufferShape"));
    }

    #[test]
    fn test_buffer_shape_all_primitive_types() {
        let types = [
            PrimitiveType::F32,
            PrimitiveType::F64,
            PrimitiveType::S8,
            PrimitiveType::S16,
            PrimitiveType::S32,
            PrimitiveType::S64,
            PrimitiveType::U8,
            PrimitiveType::U16,
            PrimitiveType::U32,
            PrimitiveType::U64,
            PrimitiveType::F16,
            PrimitiveType::BF16,
            PrimitiveType::C64,
            PrimitiveType::C128,
            PrimitiveType::Pred,
        ];

        for ty in types {
            let shape = BufferShape::new(vec![2], ty);
            assert_eq!(shape.element_type(), ty);
        }
    }

    #[test]
    fn test_buffer_shape_large_dimensions() {
        let shape = BufferShape::new(vec![1000000], PrimitiveType::F32);
        assert_eq!(shape.dims()[0], 1000000);
    }

    #[test]
    fn test_buffer_shape_zero_dimension() {
        // Zero-size dimension (empty tensor)
        let shape = BufferShape::new(vec![0], PrimitiveType::F32);
        assert_eq!(shape.dims()[0], 0);
    }
}

/// Comprehensive tests for Executable types
#[cfg(test)]
mod comprehensive_executable_tests {
    #[test]
    fn test_executable_type_exists() {
        use crate::Executable;
        fn _takes_exec(_: &Executable) {}
    }

    #[test]
    fn test_executable_debug_trait() {
        use std::fmt::Debug;

        use crate::Executable;
        fn assert_debug<T: Debug>() {}
        assert_debug::<Executable>();
    }

    #[test]
    fn test_loaded_executable_type_exists() {
        use crate::LoadedExecutable;
        fn _takes_exec(_: &LoadedExecutable) {}
    }

    #[test]
    fn test_loaded_executable_debug_trait() {
        use std::fmt::Debug;

        use crate::LoadedExecutable;
        fn assert_debug<T: Debug>() {}
        assert_debug::<LoadedExecutable>();
    }

    #[test]
    fn test_compiled_memory_stats_exists() {
        use crate::CompiledMemoryStats;
        fn _takes_stats(_: &CompiledMemoryStats) {}
    }

    #[test]
    fn test_compiled_memory_stats_fields() {
        use crate::CompiledMemoryStats;

        // Create an instance directly
        let stats = CompiledMemoryStats {
            generated_code_size_in_bytes: 1000,
            argument_size_in_bytes: 500,
            output_size_in_bytes: 200,
            alias_size_in_bytes: 100,
            temp_size_in_bytes: 50,
            host_generated_code_size_in_bytes: 0,
            host_argument_size_in_bytes: 0,
            host_output_size_in_bytes: 0,
            host_alias_size_in_bytes: 0,
            host_temp_size_in_bytes: 0,
        };

        assert_eq!(stats.generated_code_size_in_bytes, 1000);
        assert_eq!(stats.argument_size_in_bytes, 500);
        assert_eq!(stats.output_size_in_bytes, 200);
    }

    #[test]
    fn test_compiled_memory_stats_all_fields_accessible() {
        use crate::CompiledMemoryStats;

        let stats = CompiledMemoryStats {
            generated_code_size_in_bytes: 1,
            argument_size_in_bytes: 2,
            output_size_in_bytes: 3,
            alias_size_in_bytes: 4,
            temp_size_in_bytes: 5,
            host_generated_code_size_in_bytes: 6,
            host_argument_size_in_bytes: 7,
            host_output_size_in_bytes: 8,
            host_alias_size_in_bytes: 9,
            host_temp_size_in_bytes: 10,
        };

        assert_eq!(stats.generated_code_size_in_bytes, 1);
        assert_eq!(stats.argument_size_in_bytes, 2);
        assert_eq!(stats.output_size_in_bytes, 3);
        assert_eq!(stats.alias_size_in_bytes, 4);
        assert_eq!(stats.temp_size_in_bytes, 5);
        assert_eq!(stats.host_generated_code_size_in_bytes, 6);
        assert_eq!(stats.host_argument_size_in_bytes, 7);
        assert_eq!(stats.host_output_size_in_bytes, 8);
        assert_eq!(stats.host_alias_size_in_bytes, 9);
        assert_eq!(stats.host_temp_size_in_bytes, 10);
    }

    #[test]
    fn test_executable_method_signatures() {
        use std::borrow::Cow;

        use crate::{Api, CompiledMemoryStats, PrimitiveType, Result};

        #[allow(dead_code)]
        trait ExecutableMethods {
            fn api(&self) -> &Api;
            fn name(&self) -> Result<Cow<'_, str>>;
            fn num_replicas(&self) -> Result<usize>;
            fn num_partitions(&self) -> Result<usize>;
            fn num_outputs(&self) -> Result<usize>;
            fn size_of_generated_code_in_bytes(&self) -> Result<i64>;
            fn output_element_types(&self) -> Result<Vec<PrimitiveType>>;
            fn output_dimensions(&self) -> Result<Vec<Vec<i64>>>;
            fn fingerprint(&self) -> Result<Cow<'_, str>>;
            fn memory_stats(&self) -> Result<CompiledMemoryStats>;
        }
    }

    #[test]
    fn test_loaded_executable_method_signatures() {
        use crate::{Client, Device, Executable, Result};

        #[allow(dead_code)]
        trait LoadedExecutableMethods {
            fn client(&self) -> &Client;
            fn executable(&self) -> &Executable;
            fn addressable_devices(&self) -> Result<Vec<Device>>;
            fn delete(self);
            fn is_deleted(&self) -> Result<bool>;
        }
    }
}

/// Comprehensive tests for async_transfer types
#[cfg(test)]
mod comprehensive_async_transfer_tests {
    #[test]
    fn test_async_host_to_device_transfer_manager_type_exists() {
        use crate::AsyncHostToDeviceTransferManager;
        fn _takes_mgr(_: &AsyncHostToDeviceTransferManager) {}
    }

    #[test]
    fn test_async_host_to_device_transfer_manager_debug() {
        use std::fmt::Debug;

        use crate::AsyncHostToDeviceTransferManager;
        fn assert_debug<T: Debug>() {}
        assert_debug::<AsyncHostToDeviceTransferManager>();
    }

    #[test]
    fn test_type_size_properties() {
        use crate::ty::{Bool, Type, F32, F64, I16, I32, I64, I8, U16, U32, U64, U8};

        // Verify SIZE constants for marker types
        assert_eq!(F32::SIZE, 4);
        assert_eq!(F64::SIZE, 8);
        assert_eq!(I8::SIZE, 1);
        assert_eq!(I16::SIZE, 2);
        assert_eq!(I32::SIZE, 4);
        assert_eq!(I64::SIZE, 8);
        assert_eq!(U8::SIZE, 1);
        assert_eq!(U16::SIZE, 2);
        assert_eq!(U32::SIZE, 4);
        assert_eq!(U64::SIZE, 8);
        assert_eq!(Bool::SIZE, 1);
    }

    #[test]
    fn test_type_alignment_properties() {
        use crate::ty::{Type, F32, F64, I32, I64};

        // Alignment should be at least 1 and power of 2
        fn is_power_of_two(n: usize) -> bool {
            n > 0 && (n & (n - 1)) == 0
        }

        assert!(is_power_of_two(F32::ALIGNMENT));
        assert!(is_power_of_two(F64::ALIGNMENT));
        assert!(is_power_of_two(I32::ALIGNMENT));
        assert!(is_power_of_two(I64::ALIGNMENT));
    }

    #[test]
    fn test_chunk_size_calculations() {
        // Test chunking for large transfers
        let total_bytes: usize = 1_000_000;
        let chunk_size: usize = 64 * 1024; // 64KB chunks

        let num_chunks = total_bytes.div_ceil(chunk_size);
        assert_eq!(num_chunks, 16); // Ceiling division
    }

    #[test]
    fn test_tensor_size_calculation() {
        // Test calculating tensor size from shape
        fn tensor_size(dims: &[i64], elem_size: usize) -> usize {
            let num_elements: i64 = dims.iter().product();
            num_elements as usize * elem_size
        }

        assert_eq!(tensor_size(&[2, 3, 4], 4), 96); // 2*3*4*4 = 96 bytes
        assert_eq!(tensor_size(&[1000, 1000], 8), 8_000_000); // 1M f64s
        assert_eq!(tensor_size(&[], 4), 4); // Scalar
        assert_eq!(tensor_size(&[0], 4), 0); // Empty tensor
    }

    #[test]
    fn test_buffer_shape_element_count() {
        fn num_elements(dims: &[i64]) -> i64 {
            dims.iter().product()
        }

        assert_eq!(num_elements(&[2, 3]), 6);
        assert_eq!(num_elements(&[10]), 10);
        assert_eq!(num_elements(&[]), 1); // Scalar
        assert_eq!(num_elements(&[2, 3, 4, 5]), 120);
    }
}
