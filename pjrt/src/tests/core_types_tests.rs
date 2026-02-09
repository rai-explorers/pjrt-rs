//! Unit Tests for Core Types
//!
//! Tests for types that don't require a PJRT plugin. Tests that duplicate
//! inline `#[cfg(test)]` modules in source files (error.rs, ty.rs, compile.rs,
//! device_assignment.rs, named_value.rs, program.rs) have been removed.

#[cfg(test)]
mod memory_layout_tests {
    use crate::MemoryLayout;

    #[test]
    fn test_memory_layout_from_strides() {
        let strides = vec![8, 4];
        let layout = MemoryLayout::from_strides(strides);
        let debug_str = format!("{:?}", layout);
        assert!(debug_str.contains("MemoryLayout"));
    }

    #[test]
    fn test_memory_layout_empty_strides() {
        let layout = MemoryLayout::from_strides(vec![]);
        let debug_str = format!("{:?}", layout);
        assert!(debug_str.contains("MemoryLayout"));
    }

    #[test]
    fn test_memory_layout_clone() {
        let layout = MemoryLayout::from_strides(vec![16, 8, 4]);
        let cloned = layout.clone();
        assert_eq!(format!("{:?}", layout), format!("{:?}", cloned));
    }
}

#[cfg(test)]
mod error_tests {
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
    fn test_poison_error_display() {
        let err = Error::PoisonError("RwLock poisoned".to_string());
        let display = format!("{}", err);
        assert!(display.contains("lock poison error"));
        assert!(display.contains("RwLock poisoned"));
    }

    #[test]
    fn test_invalid_primitive_type_edge_values() {
        let edge_values: Vec<u32> = vec![0, 100, u32::MAX, u32::MAX - 1];

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
mod type_tests {
    use std::any::TypeId;

    use num_complex::Complex;

    use crate::{
        AsDType, Bool, DType, ElemType, F8E4M3B11FNUZElem, F8E4M3FNElem, F8E4M3FNUZElem,
        F8E5M2Elem, F8E5M2FNUZElem, PrimitiveType, Type, BF16, C128, C64, F16, F32, F64, I16, I32,
        I64, I8, U16, U32, U64, U8,
    };

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
            PrimitiveType::Token,
            PrimitiveType::S2,
            PrimitiveType::U2,
            PrimitiveType::S4,
            PrimitiveType::U4,
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
    fn test_try_into_dtype_f8_types() {
        let f8_types = vec![
            (PrimitiveType::F8E5M2, "f8e5m2"),
            (PrimitiveType::F8E4M3FN, "f8e4m3fn"),
            (PrimitiveType::F8E4M3B11FNUZ, "f8e4m3b11fnuz"),
            (PrimitiveType::F8E5M2FNUZ, "f8e5m2fnuz"),
            (PrimitiveType::F8E4M3FNUZ, "f8e4m3fnuz"),
        ];

        for (primitive_type, expected_name) in f8_types {
            let dtype = primitive_type.try_into_dtype();
            assert!(
                dtype.is_ok(),
                "Expected Ok for F8 type {:?}",
                primitive_type
            );
            let dtype = dtype.unwrap();
            assert_eq!(dtype.name(), expected_name);
            assert_eq!(dtype.size(), 1);
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
        assert_eq!(<F8E5M2Elem as ElemType>::Type::NAME, "f8e5m2");
        assert_eq!(<F8E4M3FNElem as ElemType>::Type::NAME, "f8e4m3fn");
        assert_eq!(<F8E4M3B11FNUZElem as ElemType>::Type::NAME, "f8e4m3b11fnuz");
        assert_eq!(<F8E5M2FNUZElem as ElemType>::Type::NAME, "f8e5m2fnuz");
        assert_eq!(<F8E4M3FNUZElem as ElemType>::Type::NAME, "f8e4m3fnuz");
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
mod host_buffer_tests {
    use half::{bf16, f16};
    use num_complex::Complex;

    use crate::{
        HostBuffer, PrimitiveType, TypedHostBuffer, BF16, C128, C64, F16, F32, F64, I16, I32, I64,
        I8, U16, U32, U64, U8,
    };

    #[test]
    fn test_typed_host_buffer_from_scalar() {
        let buffer = TypedHostBuffer::<F32>::from_scalar(42.0f32);
        assert_eq!(buffer.data().len(), 1);
        assert_eq!(buffer.data()[0], 42.0f32);
        assert!(buffer.dims().is_empty());
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
        let buffer = TypedHostBuffer::<F32>::from_data(data, None, None);
        assert_eq!(buffer.dims(), &[6]);
    }

    #[test]
    fn test_typed_host_buffer_f16() {
        let data = vec![f16::from_f32(1.0), f16::from_f32(2.5)];
        let buffer = TypedHostBuffer::<F16>::from_data(data, None, None);
        assert_eq!(buffer.data().len(), 2);
        assert_eq!(buffer.data()[0], f16::from_f32(1.0));
        assert_eq!(buffer.dims(), &[2]);
    }

    #[test]
    fn test_typed_host_buffer_bf16() {
        let data = vec![bf16::from_f32(3.14), bf16::from_f32(-2.7)];
        let buffer = TypedHostBuffer::<BF16>::from_data(data, None, None);
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
        let buffer = TypedHostBuffer::<C64>::from_data(data, None, None);
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
    fn test_typed_host_buffer_3d_dims() {
        let data: Vec<f32> = (0..24).map(|x| x as f32).collect();
        let buffer = TypedHostBuffer::<F32>::from_data(data, Some(vec![2, 3, 4]), None);
        assert_eq!(buffer.dims(), &[2, 3, 4]);
        assert_eq!(buffer.data().len(), 24);
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
        let buffer = TypedHostBuffer::<F32>::from_data(data, None, None);
        assert_eq!(buffer.data().len(), 8);
        assert!(buffer.data()[0].is_infinite() && buffer.data()[0].is_sign_positive());
        assert!(buffer.data()[1].is_infinite() && buffer.data()[1].is_sign_negative());
        assert!(buffer.data()[2].is_nan());
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
        let location = CallLocation::new("my_function", "test.py", 42).unwrap();
        assert_eq!(location.function_name(), Some("my_function"));
        assert_eq!(location.file_name(), Some("test.py"));
        assert_eq!(location.line_number(), Some(42));
    }

    #[test]
    fn test_execute_options_with_call_location() {
        let location = CallLocation::new("train", "model.py", 150).unwrap();
        let options = ExecuteOptions::new().call_location(location);
        let loc = options.get_call_location().unwrap();
        assert_eq!(loc.function_name(), Some("train"));
    }

    #[test]
    fn test_execute_options_chaining() {
        let options = ExecuteOptions::new()
            .launch_id(5)
            .non_donatable_input_indices(vec![1, 3])
            .call_location(CallLocation::new("f", "x.py", 1).unwrap());

        assert_eq!(options.get_launch_id(), 5);
        assert_eq!(options.get_non_donatable_input_indices(), &[1, 3]);
        assert!(options.get_call_location().is_some());
    }
}

#[cfg(test)]
#[allow(clippy::approx_constant)]
mod named_value_tests {
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
    fn test_value_string_unicode() {
        let unicode_str = "Hello, 世界! 🌍";
        let value = Value::String(unicode_str.to_string());
        assert_eq!(value, Value::String(unicode_str.to_string()));
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
    fn test_named_value_map_from_empty_hashmap() {
        let map = NamedValueMap::from(HashMap::<String, Value>::new());
        assert!(map.get("anything").is_none());
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
    fn test_buffer_shape_scalar() {
        let shape = BufferShape::new(vec![], PrimitiveType::S32);
        assert!(shape.dims().is_empty());
    }

    #[test]
    fn test_buffer_shape_high_dimensional() {
        let shape = BufferShape::new(vec![2, 3, 4, 5, 6], PrimitiveType::F16);
        assert_eq!(shape.dims().len(), 5);
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
        let shape = BufferShape::new(vec![0], PrimitiveType::F32);
        assert_eq!(shape.dims()[0], 0);
    }
}

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
    fn test_memory_stats_zero_values() {
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
}

#[cfg(test)]
mod compiled_memory_stats_tests {
    use crate::CompiledMemoryStats;

    #[test]
    fn test_compiled_memory_stats_fields() {
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
}
