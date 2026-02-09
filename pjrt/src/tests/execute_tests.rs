//! Unit Tests for Execute Module
//!
//! These tests verify the types in the execute module:
//! - `ExecuteOptions`: Configuration options for execution
//! - `CallLocation`: Source location information for debugging
//! - `TransferMetadata`: Metadata for data transfers
//! - `SendCallbackInfo` / `RecvCallbackInfo`: Callback configuration
//!
//! Tests do not require a PJRT plugin to run.

#[cfg(test)]
mod execute_options_tests {
    use crate::execute::ExecuteOptions;

    #[test]
    fn test_execute_options_new() {
        let options = ExecuteOptions::new();
        assert_eq!(options.get_launch_id(), 0);
        assert!(options.get_non_donatable_input_indices().is_empty());
        assert!(options.get_call_location().is_none());
        assert!(options.get_send_callbacks().is_empty());
        assert!(options.get_recv_callbacks().is_empty());
    }

    #[test]
    fn test_execute_options_default() {
        let options = ExecuteOptions::default();
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
    fn test_execute_options_launch_id_zero() {
        let options = ExecuteOptions::new().launch_id(0);
        assert_eq!(options.get_launch_id(), 0);
    }

    #[test]
    fn test_execute_options_launch_id_negative() {
        let options = ExecuteOptions::new().launch_id(-1);
        assert_eq!(options.get_launch_id(), -1);
    }

    #[test]
    fn test_execute_options_non_donatable_input_indices_vec() {
        let indices = vec![0, 2, 5];
        let options = ExecuteOptions::new().non_donatable_input_indices(indices.clone());
        assert_eq!(options.get_non_donatable_input_indices(), &[0, 2, 5]);
    }

    #[test]
    fn test_execute_options_non_donatable_input_indices_array() {
        let options = ExecuteOptions::new().non_donatable_input_indices([1i64, 3, 7]);
        assert_eq!(options.get_non_donatable_input_indices(), &[1, 3, 7]);
    }

    #[test]
    fn test_execute_options_non_donatable_input_indices_empty() {
        let options = ExecuteOptions::new().non_donatable_input_indices(Vec::<i64>::new());
        assert!(options.get_non_donatable_input_indices().is_empty());
    }

    #[test]
    fn test_execute_options_call_location() {
        use crate::CallLocation;

        let location = CallLocation::new("test_func", "test.py", 42).unwrap();
        let options = ExecuteOptions::new().call_location(location);

        let retrieved = options.get_call_location().unwrap();
        assert_eq!(retrieved.function_name(), Some("test_func"));
        assert_eq!(retrieved.file_name(), Some("test.py"));
        assert_eq!(retrieved.line_number(), Some(42));
    }

    #[test]
    fn test_execute_options_task_incarnation_ids() {
        let task_ids = vec![0, 1, 2, 3];
        let incarnation_ids = vec![100i64, 101, 102, 103];
        let options =
            ExecuteOptions::new().task_incarnation_ids(task_ids.clone(), incarnation_ids.clone());

        // Verify the options were set (accessing private fields indirectly via conversion)
        // We verify this worked by ensuring no panic occurred during construction
        let _ = options;
    }

    #[test]
    #[should_panic(expected = "task_ids and incarnation_ids must have the same length")]
    fn test_execute_options_task_incarnation_ids_length_mismatch() {
        let task_ids = vec![0, 1, 2];
        let incarnation_ids = vec![100i64, 101]; // Mismatched length
        let _options = ExecuteOptions::new().task_incarnation_ids(task_ids, incarnation_ids);
    }

    #[test]
    fn test_execute_options_chained_builder() {
        use crate::CallLocation;

        let location = CallLocation::new("my_func", "file.py", 100).unwrap();
        let options = ExecuteOptions::new()
            .launch_id(5)
            .non_donatable_input_indices(vec![0, 1])
            .call_location(location);

        assert_eq!(options.get_launch_id(), 5);
        assert_eq!(options.get_non_donatable_input_indices(), &[0, 1]);
        assert!(options.get_call_location().is_some());
    }

    #[test]
    fn test_execute_options_send_callbacks_empty() {
        let options = ExecuteOptions::new().send_callbacks(vec![]);
        assert!(options.get_send_callbacks().is_empty());
    }

    #[test]
    fn test_execute_options_recv_callbacks_empty() {
        let options = ExecuteOptions::new().recv_callbacks(vec![]);
        assert!(options.get_recv_callbacks().is_empty());
    }

    #[test]
    fn test_execute_options_conversion_to_raw() {
        use pjrt_sys::PJRT_ExecuteOptions;

        let options = ExecuteOptions::new()
            .launch_id(123)
            .non_donatable_input_indices(vec![0, 2, 4]);

        let raw: PJRT_ExecuteOptions = (&options).into();
        assert_eq!(raw.launch_id, 123);
        assert_eq!(raw.num_non_donatable_input_indices, 3);
    }

    #[test]
    fn test_execute_options_conversion_to_raw_with_location() {
        use pjrt_sys::PJRT_ExecuteOptions;

        use crate::CallLocation;

        let location = CallLocation::new("func", "file.py", 50).unwrap();
        let options = ExecuteOptions::new().call_location(location);

        let raw: PJRT_ExecuteOptions = (&options).into();
        assert!(!raw.call_location.is_null());
    }

    #[test]
    fn test_execute_options_conversion_preserves_indices_pointer() {
        use pjrt_sys::PJRT_ExecuteOptions;

        let indices = vec![10i64, 20, 30];
        let options = ExecuteOptions::new().non_donatable_input_indices(indices);

        let raw: PJRT_ExecuteOptions = (&options).into();

        // The pointer should point to valid data
        assert!(!raw.non_donatable_input_indices.is_null());
        assert_eq!(raw.num_non_donatable_input_indices, 3);

        // Verify the data is accessible (must be done while options is still alive)
        unsafe {
            assert_eq!(*raw.non_donatable_input_indices, 10);
            assert_eq!(*raw.non_donatable_input_indices.add(1), 20);
            assert_eq!(*raw.non_donatable_input_indices.add(2), 30);
        }
    }
}

#[cfg(test)]
mod call_location_tests {
    use crate::CallLocation;

    #[test]
    fn test_call_location_new() {
        let location = CallLocation::new("train_step", "model.py", 42).unwrap();
        assert_eq!(location.function_name(), Some("train_step"));
        assert_eq!(location.file_name(), Some("model.py"));
        assert_eq!(location.line_number(), Some(42));
    }

    #[test]
    fn test_call_location_new_empty_function_name() {
        let location = CallLocation::new("", "file.py", 1).unwrap();
        assert_eq!(location.function_name(), Some(""));
        assert_eq!(location.file_name(), Some("file.py"));
        assert_eq!(location.line_number(), Some(1));
    }

    #[test]
    fn test_call_location_new_empty_file_name() {
        let location = CallLocation::new("func", "", 10).unwrap();
        assert_eq!(location.function_name(), Some("func"));
        assert_eq!(location.file_name(), Some(""));
        assert_eq!(location.line_number(), Some(10));
    }

    #[test]
    fn test_call_location_new_zero_line() {
        let location = CallLocation::new("func", "file.py", 0).unwrap();
        assert_eq!(location.line_number(), Some(0));
    }

    #[test]
    fn test_call_location_new_large_line_number() {
        let location = CallLocation::new("func", "file.py", u32::MAX).unwrap();
        assert_eq!(location.line_number(), Some(u32::MAX));
    }

    #[test]
    fn test_call_location_from_string_function_file_line() {
        let location = CallLocation::from_string("my_func:my_file.py:99").unwrap();
        assert_eq!(location.function_name(), Some("my_func"));
        assert_eq!(location.file_name(), Some("my_file.py"));
        assert_eq!(location.line_number(), Some(99));
    }

    #[test]
    fn test_call_location_from_string_file_line_only() {
        let location = CallLocation::from_string("script.py:123").unwrap();
        // With two parts, function_name should be None (only file:line format)
        assert_eq!(location.function_name(), None);
        assert_eq!(location.file_name(), Some("script.py"));
        assert_eq!(location.line_number(), Some(123));
    }

    #[test]
    fn test_call_location_from_string_line_only() {
        let location = CallLocation::from_string("42").unwrap();
        assert_eq!(location.function_name(), None);
        assert_eq!(location.file_name(), None);
        assert_eq!(location.line_number(), Some(42));
    }

    #[test]
    fn test_call_location_from_string_no_line_number() {
        let location = CallLocation::from_string("func:file:notanumber").unwrap();
        assert_eq!(location.function_name(), Some("func"));
        assert_eq!(location.file_name(), Some("file"));
        assert_eq!(location.line_number(), None); // Can't parse "notanumber"
    }

    #[test]
    fn test_call_location_from_string_empty() {
        let location = CallLocation::from_string("").unwrap();
        assert_eq!(location.function_name(), None);
        assert_eq!(location.file_name(), None);
        assert_eq!(location.line_number(), None);
    }

    #[test]
    fn test_call_location_from_string_colons_in_path() {
        // Windows-style path with drive letter
        let location = CallLocation::from_string("func:C:/path/to/file.py:100").unwrap();
        // This will parse as func, C, /path/to/file.py:100
        // The parsing is simplistic and doesn't handle this case well
        assert_eq!(location.function_name(), Some("func"));
        // file_name will be the second part
        assert_eq!(location.file_name(), Some("C"));
    }

    #[test]
    fn test_call_location_clone() {
        let location = CallLocation::new("func", "file.py", 50).unwrap();
        let cloned = location.clone();

        assert_eq!(location.function_name(), cloned.function_name());
        assert_eq!(location.file_name(), cloned.file_name());
        assert_eq!(location.line_number(), cloned.line_number());
    }

    #[test]
    fn test_call_location_debug() {
        let location = CallLocation::new("func", "file.py", 42).unwrap();
        let debug_str = format!("{:?}", location);
        assert!(debug_str.contains("CallLocation"));
    }

    #[test]
    fn test_call_location_as_ptr_not_null() {
        let location = CallLocation::new("func", "file.py", 1).unwrap();
        let ptr = location.as_ptr();
        assert!(!ptr.is_null());
    }

    #[test]
    fn test_call_location_unicode_function_name() {
        let location = CallLocation::new("函数名", "文件.py", 123).unwrap();
        assert_eq!(location.function_name(), Some("函数名"));
        assert_eq!(location.file_name(), Some("文件.py"));
        assert_eq!(location.line_number(), Some(123));
    }

    #[test]
    fn test_call_location_special_characters() {
        // Function and file names with special characters (but no colons or nulls)
        let location =
            CallLocation::new("func_with-dash.and_underscore", "path/to/file.py", 1).unwrap();
        assert_eq!(
            location.function_name(),
            Some("func_with-dash.and_underscore")
        );
        assert_eq!(location.file_name(), Some("path/to/file.py"));
    }
}

#[cfg(test)]
mod transfer_metadata_tests {
    use crate::{PrimitiveType, TransferMetadata};

    #[test]
    fn test_transfer_metadata_new() {
        let metadata = TransferMetadata::new(vec![2, 3, 4], PrimitiveType::F32);
        assert_eq!(metadata.dims, vec![2, 3, 4]);
        assert_eq!(metadata.element_type, PrimitiveType::F32);
        assert!(metadata.layout.is_none());
    }

    #[test]
    fn test_transfer_metadata_scalar() {
        let metadata = TransferMetadata::new(vec![], PrimitiveType::F64);
        assert!(metadata.dims.is_empty());
        assert_eq!(metadata.element_type, PrimitiveType::F64);
    }

    #[test]
    fn test_transfer_metadata_1d() {
        let metadata = TransferMetadata::new(vec![100], PrimitiveType::S32);
        assert_eq!(metadata.dims, vec![100]);
        assert_eq!(metadata.element_type, PrimitiveType::S32);
    }

    #[test]
    fn test_transfer_metadata_num_elements() {
        let metadata = TransferMetadata::new(vec![2, 3, 4], PrimitiveType::F32);
        assert_eq!(metadata.num_elements(), 24);
    }

    #[test]
    fn test_transfer_metadata_num_elements_scalar() {
        let metadata = TransferMetadata::new(vec![], PrimitiveType::F32);
        assert_eq!(metadata.num_elements(), 1); // Product of empty vec is 1
    }

    #[test]
    fn test_transfer_metadata_num_elements_1d() {
        let metadata = TransferMetadata::new(vec![10], PrimitiveType::F32);
        assert_eq!(metadata.num_elements(), 10);
    }

    #[test]
    fn test_transfer_metadata_num_elements_large() {
        let metadata = TransferMetadata::new(vec![1000, 1000], PrimitiveType::F32);
        assert_eq!(metadata.num_elements(), 1_000_000);
    }

    #[test]
    fn test_transfer_metadata_size_in_bytes_f32() {
        let metadata = TransferMetadata::new(vec![2, 3, 4], PrimitiveType::F32);
        // 24 elements * 4 bytes = 96
        assert_eq!(metadata.size_in_bytes(), Some(96));
    }

    #[test]
    fn test_transfer_metadata_size_in_bytes_f64() {
        let metadata = TransferMetadata::new(vec![2, 3], PrimitiveType::F64);
        // 6 elements * 8 bytes = 48
        assert_eq!(metadata.size_in_bytes(), Some(48));
    }

    #[test]
    fn test_transfer_metadata_size_in_bytes_s8() {
        let metadata = TransferMetadata::new(vec![100], PrimitiveType::S8);
        // 100 elements * 1 byte = 100
        assert_eq!(metadata.size_in_bytes(), Some(100));
    }

    #[test]
    fn test_transfer_metadata_size_in_bytes_s16() {
        let metadata = TransferMetadata::new(vec![50], PrimitiveType::S16);
        // 50 elements * 2 bytes = 100
        assert_eq!(metadata.size_in_bytes(), Some(100));
    }

    #[test]
    fn test_transfer_metadata_size_in_bytes_s32() {
        let metadata = TransferMetadata::new(vec![10], PrimitiveType::S32);
        // 10 elements * 4 bytes = 40
        assert_eq!(metadata.size_in_bytes(), Some(40));
    }

    #[test]
    fn test_transfer_metadata_size_in_bytes_s64() {
        let metadata = TransferMetadata::new(vec![5], PrimitiveType::S64);
        // 5 elements * 8 bytes = 40
        assert_eq!(metadata.size_in_bytes(), Some(40));
    }

    #[test]
    fn test_transfer_metadata_size_in_bytes_invalid_type() {
        let metadata = TransferMetadata::new(vec![10], PrimitiveType::Invalid);
        assert!(metadata.size_in_bytes().is_none());
    }

    #[test]
    fn test_transfer_metadata_size_in_bytes_token_type() {
        let metadata = TransferMetadata::new(vec![10], PrimitiveType::Token);
        assert!(metadata.size_in_bytes().is_none());
    }

    #[test]
    fn test_transfer_metadata_with_layout() {
        use crate::MemoryLayout;

        let layout = MemoryLayout::from_strides(vec![16, 4]);
        let metadata = TransferMetadata::new(vec![4, 4], PrimitiveType::F32).with_layout(layout);

        assert!(metadata.layout.is_some());
    }

    #[test]
    fn test_transfer_metadata_debug() {
        let metadata = TransferMetadata::new(vec![2, 3], PrimitiveType::F32);
        let debug_str = format!("{:?}", metadata);
        assert!(debug_str.contains("TransferMetadata"));
        assert!(debug_str.contains("dims"));
        assert!(debug_str.contains("element_type"));
    }

    #[test]
    fn test_transfer_metadata_all_supported_types() {
        // Test all types that should have known sizes
        let test_cases = [
            (PrimitiveType::Pred, Some(1)),  // bool
            (PrimitiveType::S8, Some(1)),    // i8
            (PrimitiveType::S16, Some(2)),   // i16
            (PrimitiveType::S32, Some(4)),   // i32
            (PrimitiveType::S64, Some(8)),   // i64
            (PrimitiveType::U8, Some(1)),    // u8
            (PrimitiveType::U16, Some(2)),   // u16
            (PrimitiveType::U32, Some(4)),   // u32
            (PrimitiveType::U64, Some(8)),   // u64
            (PrimitiveType::F16, Some(2)),   // f16
            (PrimitiveType::BF16, Some(2)),  // bf16
            (PrimitiveType::F32, Some(4)),   // f32
            (PrimitiveType::F64, Some(8)),   // f64
            (PrimitiveType::C64, Some(8)),   // complex64 (2 * f32)
            (PrimitiveType::C128, Some(16)), // complex128 (2 * f64)
        ];

        for (ptype, expected_size_per_elem) in test_cases {
            let metadata = TransferMetadata::new(vec![1], ptype);
            assert_eq!(
                metadata.size_in_bytes(),
                expected_size_per_elem,
                "Failed for type {:?}",
                ptype
            );
        }
    }

    #[test]
    fn test_transfer_metadata_unsupported_types_return_none() {
        // Test types that don't have DType mappings
        let unsupported_types = [
            PrimitiveType::Invalid,
            PrimitiveType::Token,
            PrimitiveType::S2,
            PrimitiveType::U2,
            PrimitiveType::S4,
            PrimitiveType::U4,
        ];

        for ptype in unsupported_types {
            let metadata = TransferMetadata::new(vec![1], ptype);
            assert!(
                metadata.size_in_bytes().is_none(),
                "Expected None for type {:?}",
                ptype
            );
        }
    }

    #[test]
    fn test_transfer_metadata_f8_types_return_some() {
        let f8_types = [
            PrimitiveType::F8E5M2,
            PrimitiveType::F8E4M3FN,
            PrimitiveType::F8E4M3B11FNUZ,
            PrimitiveType::F8E5M2FNUZ,
            PrimitiveType::F8E4M3FNUZ,
        ];

        for ptype in f8_types {
            let metadata = TransferMetadata::new(vec![2, 3], ptype);
            // F8 types are 1 byte each, so 2*3*1 = 6 bytes
            assert_eq!(
                metadata.size_in_bytes(),
                Some(6),
                "Expected Some(6) for F8 type {:?}",
                ptype
            );
        }
    }
}

#[cfg(test)]
mod callback_info_tests {
    use std::ffi::c_void;
    use std::ptr;

    use crate::execute::{RecvCallbackInfo, SendCallbackInfo};

    #[test]
    fn test_send_callback_info_new() {
        let user_data: i32 = 42;
        let user_arg = &user_data as *const i32 as *mut c_void;

        // Note: we're passing None for the callback function for testing
        let info = unsafe { SendCallbackInfo::new(123, user_arg, None) };

        assert_eq!(info.channel_id, 123);
        assert_eq!(info.user_arg, user_arg);
        assert!(info.send_callback.is_none());
    }

    #[test]
    fn test_send_callback_info_null_user_arg() {
        let info = unsafe { SendCallbackInfo::new(456, ptr::null_mut(), None) };

        assert_eq!(info.channel_id, 456);
        assert!(info.user_arg.is_null());
    }

    #[test]
    fn test_send_callback_info_to_raw() {
        let user_data: i32 = 100;
        let user_arg = &user_data as *const i32 as *mut c_void;
        let info = unsafe { SendCallbackInfo::new(789, user_arg, None) };

        let raw = info.to_raw();
        assert_eq!(raw.channel_id, 789);
        assert_eq!(raw.user_arg, user_arg);
    }

    #[test]
    fn test_send_callback_info_debug() {
        let info = unsafe { SendCallbackInfo::new(42, ptr::null_mut(), None) };
        let debug_str = format!("{:?}", info);
        assert!(debug_str.contains("SendCallbackInfo"));
        assert!(debug_str.contains("channel_id"));
        assert!(debug_str.contains("42"));
    }

    #[test]
    fn test_recv_callback_info_new() {
        let user_data: i32 = 42;
        let user_arg = &user_data as *const i32 as *mut c_void;

        let info = unsafe { RecvCallbackInfo::new(123, user_arg, None) };

        assert_eq!(info.channel_id, 123);
        assert_eq!(info.user_arg, user_arg);
        assert!(info.recv_callback.is_none());
    }

    #[test]
    fn test_recv_callback_info_null_user_arg() {
        let info = unsafe { RecvCallbackInfo::new(456, ptr::null_mut(), None) };

        assert_eq!(info.channel_id, 456);
        assert!(info.user_arg.is_null());
    }

    #[test]
    fn test_recv_callback_info_to_raw() {
        let user_data: i32 = 100;
        let user_arg = &user_data as *const i32 as *mut c_void;
        let info = unsafe { RecvCallbackInfo::new(789, user_arg, None) };

        let raw = info.to_raw();
        assert_eq!(raw.channel_id, 789);
        assert_eq!(raw.user_arg, user_arg);
    }

    #[test]
    fn test_recv_callback_info_debug() {
        let info = unsafe { RecvCallbackInfo::new(42, ptr::null_mut(), None) };
        let debug_str = format!("{:?}", info);
        assert!(debug_str.contains("RecvCallbackInfo"));
        assert!(debug_str.contains("channel_id"));
        assert!(debug_str.contains("42"));
    }

    #[test]
    fn test_send_callback_info_negative_channel_id() {
        // Channel IDs might be negative in some scenarios
        let info = unsafe { SendCallbackInfo::new(-1, ptr::null_mut(), None) };
        assert_eq!(info.channel_id, -1);

        let raw = info.to_raw();
        assert_eq!(raw.channel_id, -1);
    }

    #[test]
    fn test_recv_callback_info_large_channel_id() {
        let info = unsafe { RecvCallbackInfo::new(i64::MAX, ptr::null_mut(), None) };
        assert_eq!(info.channel_id, i64::MAX);

        let raw = info.to_raw();
        assert_eq!(raw.channel_id, i64::MAX);
    }
}

#[cfg(test)]
mod execute_options_raw_tests {
    use pjrt_sys::PJRT_ExecuteOptions;

    use crate::execute::{ExecuteOptions, ExecuteOptionsRaw};

    #[test]
    fn test_execute_options_raw_empty() {
        let options = ExecuteOptions::new();
        let mut raw = PJRT_ExecuteOptions::new();

        let _raw_holder = ExecuteOptionsRaw::new(&options, &mut raw);

        assert_eq!(raw.launch_id, 0);
        assert_eq!(raw.num_non_donatable_input_indices, 0);
        assert!(raw.call_location.is_null());
        assert_eq!(raw.num_tasks, 0);
    }

    #[test]
    fn test_execute_options_raw_with_launch_id() {
        let options = ExecuteOptions::new().launch_id(42);
        let mut raw = PJRT_ExecuteOptions::new();

        let _raw_holder = ExecuteOptionsRaw::new(&options, &mut raw);

        assert_eq!(raw.launch_id, 42);
    }

    #[test]
    fn test_execute_options_raw_with_non_donatable_indices() {
        let options = ExecuteOptions::new().non_donatable_input_indices(vec![1, 3, 5]);
        let mut raw = PJRT_ExecuteOptions::new();

        let _raw_holder = ExecuteOptionsRaw::new(&options, &mut raw);

        assert_eq!(raw.num_non_donatable_input_indices, 3);
        assert!(!raw.non_donatable_input_indices.is_null());
    }

    #[test]
    fn test_execute_options_raw_with_call_location() {
        use crate::CallLocation;

        let location = CallLocation::new("func", "file.py", 100).unwrap();
        let options = ExecuteOptions::new().call_location(location);
        let mut raw = PJRT_ExecuteOptions::new();

        let _raw_holder = ExecuteOptionsRaw::new(&options, &mut raw);

        assert!(!raw.call_location.is_null());
    }

    #[test]
    fn test_execute_options_raw_with_task_incarnation_ids() {
        let options = ExecuteOptions::new().task_incarnation_ids(vec![0, 1], vec![10i64, 11]);
        let mut raw = PJRT_ExecuteOptions::new();

        let _raw_holder = ExecuteOptionsRaw::new(&options, &mut raw);

        assert_eq!(raw.num_tasks, 2);
        assert!(!raw.task_ids.is_null());
        assert!(!raw.incarnation_ids.is_null());
    }

    #[test]
    fn test_execute_options_raw_lifetime() {
        // Test that data remains valid while raw_holder is alive
        let options = ExecuteOptions::new()
            .launch_id(99)
            .non_donatable_input_indices(vec![0, 2, 4]);
        let mut raw = PJRT_ExecuteOptions::new();

        let _raw_holder = ExecuteOptionsRaw::new(&options, &mut raw);

        // Access raw fields while holder is alive - should be safe
        assert_eq!(raw.launch_id, 99);
        assert_eq!(raw.num_non_donatable_input_indices, 3);

        // Verify data through pointer
        unsafe {
            assert_eq!(*raw.non_donatable_input_indices, 0);
            assert_eq!(*raw.non_donatable_input_indices.add(1), 2);
            assert_eq!(*raw.non_donatable_input_indices.add(2), 4);
        }
    }
}

#[cfg(test)]
mod execution_inputs_tests {
    // Note: ExecutionInputs trait implementations require Buffer which needs a plugin.
    // We test the trait definition and empty input case here.

    use crate::ExecutionInputs;

    #[test]
    fn test_unit_execution_inputs() {
        let inputs: () = ();
        let ptrs = inputs.buffer_ptrs();

        // Empty input should produce one empty vec
        assert_eq!(ptrs.len(), 1);
        assert!(ptrs[0].is_empty());
    }

    #[test]
    fn test_unit_non_donatable_indices() {
        let inputs: () = ();
        let indices = inputs.non_donatable_input_indices();
        assert!(indices.is_empty());
    }
}
