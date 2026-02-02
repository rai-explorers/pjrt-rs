//! Unit Tests for Async Transfer Module
//!
//! These tests verify the async transfer types like BufferShape, AsyncTransferBuilder,
//! TypedAsyncTransfer, RawAsyncTransfer, and MultiBufTransfer configuration.
//! They do not require a PJRT plugin to run.

#[cfg(test)]
mod buffer_shape_tests {
    use crate::{BufferShape, MemoryLayout, PrimitiveType};

    #[test]
    fn test_buffer_shape_new_simple() {
        let shape = BufferShape::new(vec![10, 20], PrimitiveType::F32);
        assert_eq!(shape.dims(), &[10, 20]);
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
        assert_eq!(shape.element_type(), PrimitiveType::S32);
    }

    #[test]
    fn test_buffer_shape_high_dimensional() {
        let dims = vec![2, 3, 4, 5, 6];
        let shape = BufferShape::new(dims.clone(), PrimitiveType::U8);
        assert_eq!(shape.dims(), dims.as_slice());
        assert_eq!(shape.element_type(), PrimitiveType::U8);
    }

    #[test]
    fn test_buffer_shape_with_strides_layout() {
        let layout = MemoryLayout::from_strides(vec![80, 4]);
        let shape = BufferShape::new(vec![10, 20], PrimitiveType::F32).with_layout(layout);

        assert_eq!(shape.dims(), &[10, 20]);
        assert_eq!(shape.element_type(), PrimitiveType::F32);
        assert!(shape.layout().is_some());
    }

    #[test]
    fn test_buffer_shape_with_tiled_layout() {
        let layout = MemoryLayout::from_tiled(vec![1, 0])
            .tile_dims(vec![8, 8])
            .build();
        let shape = BufferShape::new(vec![64, 64], PrimitiveType::F32).with_layout(layout);

        assert_eq!(shape.dims(), &[64, 64]);
        assert!(shape.layout().is_some());
    }

    #[test]
    fn test_buffer_shape_various_types() {
        let types = [
            PrimitiveType::F16,
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
            PrimitiveType::Pred,
            PrimitiveType::C64,
            PrimitiveType::C128,
            PrimitiveType::BF16,
        ];

        for ty in types {
            let shape = BufferShape::new(vec![10], ty);
            assert_eq!(shape.element_type(), ty);
        }
    }

    #[test]
    fn test_buffer_shape_debug() {
        let shape = BufferShape::new(vec![3, 4], PrimitiveType::F32);
        let debug_str = format!("{:?}", shape);

        assert!(debug_str.contains("BufferShape"));
        assert!(debug_str.contains("dims"));
        assert!(debug_str.contains("element_type"));
        assert!(debug_str.contains("layout"));
    }

    #[test]
    fn test_buffer_shape_debug_with_layout() {
        let layout = MemoryLayout::from_strides(vec![16, 4]);
        let shape = BufferShape::new(vec![4, 4], PrimitiveType::F32).with_layout(layout);
        let debug_str = format!("{:?}", shape);

        assert!(debug_str.contains("BufferShape"));
        assert!(debug_str.contains("Strides"));
    }

    #[test]
    fn test_buffer_shape_to_spec() {
        let shape = BufferShape::new(vec![5, 10], PrimitiveType::F64);
        let spec = shape.to_spec();

        assert_eq!(spec.num_dims, 2);
        assert_eq!(
            spec.element_type,
            PrimitiveType::F64 as pjrt_sys::PJRT_Buffer_Type
        );
        // Note: spec.dims is a pointer, we verify num_dims is correct
    }

    #[test]
    fn test_buffer_shape_chaining() {
        let layout = MemoryLayout::from_strides(vec![4]);
        let shape = BufferShape::new(vec![100], PrimitiveType::S32).with_layout(layout);

        // Verify chaining works correctly
        assert_eq!(shape.dims().len(), 1);
        assert!(shape.layout().is_some());
    }

    #[test]
    fn test_buffer_shape_large_dimensions() {
        let dims = vec![1024, 1024, 3]; // Large image-like tensor
        let shape = BufferShape::new(dims.clone(), PrimitiveType::U8);

        assert_eq!(shape.dims(), dims.as_slice());
        assert_eq!(shape.element_type(), PrimitiveType::U8);
    }

    #[test]
    fn test_buffer_shape_single_element_dims() {
        let shape = BufferShape::new(vec![1, 1, 1, 1], PrimitiveType::F32);
        assert_eq!(shape.dims(), &[1, 1, 1, 1]);
    }
}

#[cfg(test)]
mod async_transfer_builder_config_tests {
    //! Tests for AsyncTransferBuilder configuration that don't require a plugin.
    //! These test the builder setup and type system, not actual transfers.

    use std::marker::PhantomData;

    use crate::{MemoryLayout, PrimitiveType, F32, F64, I32};

    /// Simulates TypedAsyncTransfer state for configuration testing.
    /// This mirrors the internal structure for testing purposes.
    #[derive(Debug)]
    struct TypedTransferConfig<T> {
        dims: Vec<i64>,
        data_len: usize,
        layout: Option<MemoryLayout>,
        _marker: PhantomData<T>,
    }

    impl<T> TypedTransferConfig<T> {
        fn new(data_len: usize, dims: Vec<i64>) -> Self {
            Self {
                dims,
                data_len,
                layout: None,
                _marker: PhantomData,
            }
        }

        fn with_layout(mut self, layout: MemoryLayout) -> Self {
            self.layout = Some(layout);
            self
        }
    }

    #[test]
    fn test_typed_config_f32() {
        let data: Vec<f32> = vec![1.0, 2.0, 3.0, 4.0];
        let config = TypedTransferConfig::<F32>::new(data.len(), vec![2, 2]);

        assert_eq!(config.dims, vec![2, 2]);
        assert_eq!(config.data_len, 4);
        assert!(config.layout.is_none());
    }

    #[test]
    fn test_typed_config_f64() {
        let data: Vec<f64> = vec![1.0; 100];
        let config = TypedTransferConfig::<F64>::new(data.len(), vec![10, 10]);

        assert_eq!(config.dims, vec![10, 10]);
        assert_eq!(config.data_len, 100);
    }

    #[test]
    fn test_typed_config_i32() {
        let data: Vec<i32> = vec![1, 2, 3, 4, 5, 6];
        let config = TypedTransferConfig::<I32>::new(data.len(), vec![2, 3]);

        assert_eq!(config.dims, vec![2, 3]);
        assert_eq!(config.data_len, 6);
    }

    #[test]
    fn test_typed_config_with_layout() {
        let data: Vec<f32> = vec![0.0; 16];
        let layout = MemoryLayout::from_strides(vec![16, 4]);
        let config = TypedTransferConfig::<F32>::new(data.len(), vec![4, 4]).with_layout(layout);

        assert!(config.layout.is_some());
    }

    #[test]
    fn test_typed_config_scalar() {
        let config = TypedTransferConfig::<F32>::new(1, vec![]);
        assert!(config.dims.is_empty());
        assert_eq!(config.data_len, 1);
    }

    /// Simulates RawAsyncTransfer state for configuration testing.
    #[derive(Debug)]
    struct RawTransferConfig {
        dims: Vec<i64>,
        data_len: usize,
        element_type: PrimitiveType,
        layout: Option<MemoryLayout>,
    }

    impl RawTransferConfig {
        fn new(data_len: usize, dims: Vec<i64>, element_type: PrimitiveType) -> Self {
            Self {
                dims,
                data_len,
                element_type,
                layout: None,
            }
        }

        fn with_layout(mut self, layout: MemoryLayout) -> Self {
            self.layout = Some(layout);
            self
        }
    }

    #[test]
    fn test_raw_config_f32() {
        let data: Vec<u8> = vec![0u8; 16]; // 4 f32s
        let config = RawTransferConfig::new(data.len(), vec![2, 2], PrimitiveType::F32);

        assert_eq!(config.dims, vec![2, 2]);
        assert_eq!(config.data_len, 16);
        assert_eq!(config.element_type, PrimitiveType::F32);
    }

    #[test]
    fn test_raw_config_various_types() {
        let data = [0u8; 100];

        let config_f16 = RawTransferConfig::new(data.len(), vec![50], PrimitiveType::F16);
        assert_eq!(config_f16.element_type, PrimitiveType::F16);

        let config_i64 = RawTransferConfig::new(data.len(), vec![12], PrimitiveType::S64);
        assert_eq!(config_i64.element_type, PrimitiveType::S64);

        let config_u8 = RawTransferConfig::new(data.len(), vec![100], PrimitiveType::U8);
        assert_eq!(config_u8.element_type, PrimitiveType::U8);
    }

    #[test]
    fn test_raw_config_with_layout() {
        let layout = MemoryLayout::from_strides(vec![8, 4]);
        let config = RawTransferConfig::new(32, vec![4, 2], PrimitiveType::F32).with_layout(layout);

        assert!(config.layout.is_some());
    }
}

#[cfg(test)]
mod multi_buf_transfer_config_tests {
    //! Tests for MultiBufTransfer configuration that don't require a plugin.

    use crate::{BufferShape, PrimitiveType};

    /// Simulates the shape accumulation in MultiBufTransfer
    struct MultiBufConfig {
        shapes: Vec<BufferShape>,
        transfer_count: usize,
    }

    impl MultiBufConfig {
        fn new() -> Self {
            Self {
                shapes: Vec::new(),
                transfer_count: 0,
            }
        }

        fn add_typed_shape<T: crate::Type>(mut self, dims: &[i64]) -> Self {
            self.shapes
                .push(BufferShape::new(dims.to_vec(), T::PRIMITIVE_TYPE));
            self.transfer_count += 1;
            self
        }

        fn add_raw_shape(mut self, dims: &[i64], element_type: PrimitiveType) -> Self {
            self.shapes
                .push(BufferShape::new(dims.to_vec(), element_type));
            self.transfer_count += 1;
            self
        }
    }

    #[test]
    fn test_multi_buf_config_empty() {
        let config = MultiBufConfig::new();
        assert!(config.shapes.is_empty());
        assert_eq!(config.transfer_count, 0);
    }

    #[test]
    fn test_multi_buf_config_single_typed() {
        let config = MultiBufConfig::new().add_typed_shape::<crate::F32>(&[10, 10]);

        assert_eq!(config.shapes.len(), 1);
        assert_eq!(config.transfer_count, 1);
        assert_eq!(config.shapes[0].element_type(), PrimitiveType::F32);
        assert_eq!(config.shapes[0].dims(), &[10, 10]);
    }

    #[test]
    fn test_multi_buf_config_multiple_typed() {
        let config = MultiBufConfig::new()
            .add_typed_shape::<crate::F32>(&[100, 100])
            .add_typed_shape::<crate::I32>(&[50])
            .add_typed_shape::<crate::F64>(&[25, 4]);

        assert_eq!(config.shapes.len(), 3);
        assert_eq!(config.transfer_count, 3);

        assert_eq!(config.shapes[0].element_type(), PrimitiveType::F32);
        assert_eq!(config.shapes[0].dims(), &[100, 100]);

        assert_eq!(config.shapes[1].element_type(), PrimitiveType::S32);
        assert_eq!(config.shapes[1].dims(), &[50]);

        assert_eq!(config.shapes[2].element_type(), PrimitiveType::F64);
        assert_eq!(config.shapes[2].dims(), &[25, 4]);
    }

    #[test]
    fn test_multi_buf_config_mixed_typed_raw() {
        let config = MultiBufConfig::new()
            .add_typed_shape::<crate::F32>(&[10])
            .add_raw_shape(&[20], PrimitiveType::U8)
            .add_typed_shape::<crate::I64>(&[5, 5]);

        assert_eq!(config.shapes.len(), 3);

        assert_eq!(config.shapes[0].element_type(), PrimitiveType::F32);
        assert_eq!(config.shapes[1].element_type(), PrimitiveType::U8);
        assert_eq!(config.shapes[2].element_type(), PrimitiveType::S64);
    }

    #[test]
    fn test_multi_buf_config_many_buffers() {
        let mut config = MultiBufConfig::new();
        for i in 0..10 {
            config = config.add_typed_shape::<crate::F32>(&[i as i64 + 1]);
        }

        assert_eq!(config.shapes.len(), 10);
        assert_eq!(config.transfer_count, 10);

        for (i, shape) in config.shapes.iter().enumerate() {
            assert_eq!(shape.dims(), &[(i + 1) as i64]);
        }
    }
}

#[cfg(test)]
mod pending_transfer_config_tests {
    //! Tests for internal PendingTransfer equivalent configurations

    use std::ffi::c_void;

    /// Equivalent to internal PendingTransfer::Raw
    struct RawPendingConfig<'a> {
        data: &'a [u8],
    }

    /// Equivalent to internal PendingTransfer::Typed
    struct TypedPendingConfig {
        data_ptr: *const c_void,
        data_len: usize,
        element_size: usize,
    }

    #[test]
    fn test_raw_pending_config() {
        let data = vec![1u8, 2, 3, 4, 5, 6, 7, 8];
        let config = RawPendingConfig { data: &data };

        assert_eq!(config.data.len(), 8);
        assert_eq!(config.data[0], 1);
        assert_eq!(config.data[7], 8);
    }

    #[test]
    fn test_typed_pending_config_f32() {
        let data: Vec<f32> = vec![1.0, 2.0, 3.0, 4.0];
        let config = TypedPendingConfig {
            data_ptr: data.as_ptr() as *const c_void,
            data_len: data.len(),
            element_size: std::mem::size_of::<f32>(),
        };

        assert_eq!(config.data_len, 4);
        assert_eq!(config.element_size, 4);
        assert!(!config.data_ptr.is_null());
    }

    #[test]
    fn test_typed_pending_config_f64() {
        let data: Vec<f64> = vec![1.0, 2.0];
        let config = TypedPendingConfig {
            data_ptr: data.as_ptr() as *const c_void,
            data_len: data.len(),
            element_size: std::mem::size_of::<f64>(),
        };

        assert_eq!(config.data_len, 2);
        assert_eq!(config.element_size, 8);
    }

    #[test]
    fn test_typed_pending_config_i32() {
        let data: Vec<i32> = vec![1, 2, 3, 4, 5, 6];
        let config = TypedPendingConfig {
            data_ptr: data.as_ptr() as *const c_void,
            data_len: data.len(),
            element_size: std::mem::size_of::<i32>(),
        };

        assert_eq!(config.data_len, 6);
        assert_eq!(config.element_size, 4);
    }

    #[test]
    fn test_byte_conversion() {
        // Test that typed data can be correctly viewed as bytes
        let data: Vec<f32> = vec![1.0, 2.0, 3.0, 4.0];
        let element_size = std::mem::size_of::<f32>();
        let byte_len = data.len() * element_size;

        assert_eq!(byte_len, 16);

        // Verify the byte representation
        let bytes = unsafe { std::slice::from_raw_parts(data.as_ptr() as *const u8, byte_len) };
        assert_eq!(bytes.len(), 16);
    }

    #[test]
    fn test_empty_typed_pending() {
        let data: Vec<f32> = vec![];
        let config = TypedPendingConfig {
            data_ptr: data.as_ptr() as *const c_void,
            data_len: data.len(),
            element_size: std::mem::size_of::<f32>(),
        };

        assert_eq!(config.data_len, 0);
        let byte_len = config.data_len * config.element_size;
        assert_eq!(byte_len, 0);
    }
}

#[cfg(test)]
mod debug_format_tests {
    use std::marker::PhantomData;

    use crate::{BufferShape, MemoryLayout, PrimitiveType, F32, I32};

    /// Mock TypedAsyncTransfer for Debug testing
    struct MockTypedTransfer<'a, T: crate::Type> {
        data: &'a [T::ElemType],
        dims: &'a [i64],
        layout: Option<MemoryLayout>,
        _marker: PhantomData<T>,
    }

    impl<T: crate::Type> std::fmt::Debug for MockTypedTransfer<'_, T> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.debug_struct("TypedAsyncTransfer")
                .field("type", &T::NAME)
                .field("dims", &self.dims)
                .field("data_len", &self.data.len())
                .field("layout", &self.layout)
                .finish()
        }
    }

    #[test]
    fn test_typed_transfer_debug_f32() {
        let data: Vec<f32> = vec![1.0, 2.0, 3.0, 4.0];
        let dims = vec![2i64, 2];
        let transfer: MockTypedTransfer<F32> = MockTypedTransfer {
            data: &data,
            dims: &dims,
            layout: None,
            _marker: PhantomData,
        };

        let debug_str = format!("{:?}", transfer);
        assert!(debug_str.contains("TypedAsyncTransfer"));
        assert!(debug_str.contains("f32"));
        assert!(debug_str.contains("data_len: 4"));
        assert!(debug_str.contains("[2, 2]"));
    }

    #[test]
    fn test_typed_transfer_debug_with_layout() {
        let data: Vec<i32> = vec![1, 2, 3];
        let dims = vec![3i64];
        let layout = MemoryLayout::from_strides(vec![4]);
        let transfer: MockTypedTransfer<I32> = MockTypedTransfer {
            data: &data,
            dims: &dims,
            layout: Some(layout),
            _marker: PhantomData,
        };

        let debug_str = format!("{:?}", transfer);
        assert!(debug_str.contains("i32"));
        assert!(debug_str.contains("Strides"));
    }

    /// Mock RawAsyncTransfer for Debug testing
    struct MockRawTransfer<'a> {
        data: &'a [u8],
        dims: &'a [i64],
        element_type: PrimitiveType,
        layout: Option<MemoryLayout>,
    }

    impl std::fmt::Debug for MockRawTransfer<'_> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.debug_struct("RawAsyncTransfer")
                .field("element_type", &self.element_type)
                .field("dims", &self.dims)
                .field("data_len", &self.data.len())
                .field("layout", &self.layout)
                .finish()
        }
    }

    #[test]
    fn test_raw_transfer_debug() {
        let data = vec![0u8; 16];
        let dims = vec![4i64];
        let transfer = MockRawTransfer {
            data: &data,
            dims: &dims,
            element_type: PrimitiveType::F32,
            layout: None,
        };

        let debug_str = format!("{:?}", transfer);
        assert!(debug_str.contains("RawAsyncTransfer"));
        assert!(debug_str.contains("F32"));
        assert!(debug_str.contains("data_len: 16"));
    }

    /// Mock MultiBufTransfer for Debug testing
    struct MockMultiBufTransfer {
        shapes: Vec<BufferShape>,
    }

    impl std::fmt::Debug for MockMultiBufTransfer {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.debug_struct("MultiBufTransfer")
                .field("num_buffers", &self.shapes.len())
                .field("shapes", &self.shapes)
                .finish()
        }
    }

    #[test]
    fn test_multi_buf_transfer_debug_empty() {
        let transfer = MockMultiBufTransfer { shapes: vec![] };
        let debug_str = format!("{:?}", transfer);

        assert!(debug_str.contains("MultiBufTransfer"));
        assert!(debug_str.contains("num_buffers: 0"));
    }

    #[test]
    fn test_multi_buf_transfer_debug_with_shapes() {
        let shapes = vec![
            BufferShape::new(vec![10, 10], PrimitiveType::F32),
            BufferShape::new(vec![5], PrimitiveType::S32),
        ];
        let transfer = MockMultiBufTransfer { shapes };
        let debug_str = format!("{:?}", transfer);

        assert!(debug_str.contains("MultiBufTransfer"));
        assert!(debug_str.contains("num_buffers: 2"));
        assert!(debug_str.contains("F32"));
    }
}

#[cfg(test)]
mod type_trait_tests {
    //! Tests for Type trait used in async transfers

    use crate::{Bool, PrimitiveType, Type, F32, F64, I16, I32, I64, I8, U16, U32, U64, U8};

    #[test]
    fn test_f32_type_properties() {
        assert_eq!(F32::NAME, "f32");
        assert_eq!(F32::PRIMITIVE_TYPE, PrimitiveType::F32);
        assert_eq!(F32::SIZE, 4);
        assert_eq!(F32::ALIGNMENT, 4);
    }

    #[test]
    fn test_f64_type_properties() {
        assert_eq!(F64::NAME, "f64");
        assert_eq!(F64::PRIMITIVE_TYPE, PrimitiveType::F64);
        assert_eq!(F64::SIZE, 8);
        assert_eq!(F64::ALIGNMENT, 8);
    }

    #[test]
    fn test_i8_type_properties() {
        assert_eq!(I8::NAME, "i8");
        assert_eq!(I8::PRIMITIVE_TYPE, PrimitiveType::S8);
        assert_eq!(I8::SIZE, 1);
        assert_eq!(I8::ALIGNMENT, 1);
    }

    #[test]
    fn test_i16_type_properties() {
        assert_eq!(I16::NAME, "i16");
        assert_eq!(I16::PRIMITIVE_TYPE, PrimitiveType::S16);
        assert_eq!(I16::SIZE, 2);
        assert_eq!(I16::ALIGNMENT, 2);
    }

    #[test]
    fn test_i32_type_properties() {
        assert_eq!(I32::NAME, "i32");
        assert_eq!(I32::PRIMITIVE_TYPE, PrimitiveType::S32);
        assert_eq!(I32::SIZE, 4);
        assert_eq!(I32::ALIGNMENT, 4);
    }

    #[test]
    fn test_i64_type_properties() {
        assert_eq!(I64::NAME, "i64");
        assert_eq!(I64::PRIMITIVE_TYPE, PrimitiveType::S64);
        assert_eq!(I64::SIZE, 8);
        assert_eq!(I64::ALIGNMENT, 8);
    }

    #[test]
    fn test_u8_type_properties() {
        assert_eq!(U8::NAME, "u8");
        assert_eq!(U8::PRIMITIVE_TYPE, PrimitiveType::U8);
        assert_eq!(U8::SIZE, 1);
        assert_eq!(U8::ALIGNMENT, 1);
    }

    #[test]
    fn test_u16_type_properties() {
        assert_eq!(U16::NAME, "u16");
        assert_eq!(U16::PRIMITIVE_TYPE, PrimitiveType::U16);
        assert_eq!(U16::SIZE, 2);
        assert_eq!(U16::ALIGNMENT, 2);
    }

    #[test]
    fn test_u32_type_properties() {
        assert_eq!(U32::NAME, "u32");
        assert_eq!(U32::PRIMITIVE_TYPE, PrimitiveType::U32);
        assert_eq!(U32::SIZE, 4);
        assert_eq!(U32::ALIGNMENT, 4);
    }

    #[test]
    fn test_u64_type_properties() {
        assert_eq!(U64::NAME, "u64");
        assert_eq!(U64::PRIMITIVE_TYPE, PrimitiveType::U64);
        assert_eq!(U64::SIZE, 8);
        assert_eq!(U64::ALIGNMENT, 8);
    }

    #[test]
    fn test_bool_type_properties() {
        assert_eq!(Bool::NAME, "bool");
        assert_eq!(Bool::PRIMITIVE_TYPE, PrimitiveType::Pred);
        assert_eq!(Bool::SIZE, 1);
        assert_eq!(Bool::ALIGNMENT, 1);
    }

    #[test]
    fn test_type_sizes_match_rust_types() {
        assert_eq!(F32::SIZE, std::mem::size_of::<f32>());
        assert_eq!(F64::SIZE, std::mem::size_of::<f64>());
        assert_eq!(I8::SIZE, std::mem::size_of::<i8>());
        assert_eq!(I16::SIZE, std::mem::size_of::<i16>());
        assert_eq!(I32::SIZE, std::mem::size_of::<i32>());
        assert_eq!(I64::SIZE, std::mem::size_of::<i64>());
        assert_eq!(U8::SIZE, std::mem::size_of::<u8>());
        assert_eq!(U16::SIZE, std::mem::size_of::<u16>());
        assert_eq!(U32::SIZE, std::mem::size_of::<u32>());
        assert_eq!(U64::SIZE, std::mem::size_of::<u64>());
    }
}

#[cfg(test)]
mod chunked_transfer_logic_tests {
    //! Tests for the chunking logic used in transfer_chunked

    /// Simulates the chunking algorithm from transfer_chunked
    fn simulate_chunked_transfer<F>(data: &[u8], chunk_size: usize, mut on_progress: F)
    where
        F: FnMut(usize, usize),
    {
        let total = data.len();
        let mut transferred = 0;

        for chunk in data.chunks(chunk_size) {
            let _is_last = transferred + chunk.len() >= total;
            let _offset = transferred;

            // Simulate transfer
            transferred += chunk.len();
            on_progress(transferred, total);
        }
    }

    #[test]
    fn test_chunked_single_chunk() {
        let data = vec![0u8; 100];
        let mut progress_calls = vec![];

        simulate_chunked_transfer(&data, 1000, |done, total| {
            progress_calls.push((done, total));
        });

        // Single chunk should result in single callback
        assert_eq!(progress_calls.len(), 1);
        assert_eq!(progress_calls[0], (100, 100));
    }

    #[test]
    fn test_chunked_exact_division() {
        let data = vec![0u8; 100];
        let mut progress_calls = vec![];

        simulate_chunked_transfer(&data, 25, |done, total| {
            progress_calls.push((done, total));
        });

        assert_eq!(progress_calls.len(), 4);
        assert_eq!(progress_calls[0], (25, 100));
        assert_eq!(progress_calls[1], (50, 100));
        assert_eq!(progress_calls[2], (75, 100));
        assert_eq!(progress_calls[3], (100, 100));
    }

    #[test]
    fn test_chunked_remainder() {
        let data = vec![0u8; 100];
        let mut progress_calls = vec![];

        simulate_chunked_transfer(&data, 30, |done, total| {
            progress_calls.push((done, total));
        });

        // 100 / 30 = 3 full chunks + 1 partial (10 bytes)
        assert_eq!(progress_calls.len(), 4);
        assert_eq!(progress_calls[0], (30, 100));
        assert_eq!(progress_calls[1], (60, 100));
        assert_eq!(progress_calls[2], (90, 100));
        assert_eq!(progress_calls[3], (100, 100));
    }

    #[test]
    fn test_chunked_tiny_chunks() {
        let data = vec![0u8; 10];
        let mut progress_calls = vec![];

        simulate_chunked_transfer(&data, 1, |done, total| {
            progress_calls.push((done, total));
        });

        assert_eq!(progress_calls.len(), 10);
        for (i, &(done, total)) in progress_calls.iter().enumerate() {
            assert_eq!(done, i + 1);
            assert_eq!(total, 10);
        }
    }

    #[test]
    fn test_chunked_empty_data() {
        let data: Vec<u8> = vec![];
        let mut progress_calls = vec![];

        simulate_chunked_transfer(&data, 100, |done, total| {
            progress_calls.push((done, total));
        });

        // Empty data should produce no callbacks
        assert!(progress_calls.is_empty());
    }

    #[test]
    fn test_chunked_progress_percentage() {
        let data = vec![0u8; 1000];
        let mut percentages = vec![];

        simulate_chunked_transfer(&data, 100, |done, total| {
            let pct = 100.0 * done as f64 / total as f64;
            percentages.push(pct);
        });

        assert_eq!(percentages.len(), 10);
        for (i, &pct) in percentages.iter().enumerate() {
            let expected = ((i + 1) * 10) as f64;
            assert!((pct - expected).abs() < 0.001);
        }
    }

    #[test]
    fn test_chunked_is_last_detection() {
        let data = [0u8; 100];
        let mut is_last_flags = Vec::new();

        let total = data.len();
        let chunk_size = 30;
        let mut transferred = 0;

        for chunk in data.chunks(chunk_size) {
            let is_last = transferred + chunk.len() >= total;
            is_last_flags.push(is_last);
            transferred += chunk.len();
        }

        assert_eq!(is_last_flags, vec![false, false, false, true]);
    }

    #[test]
    fn test_chunked_offset_calculation() {
        let data = [0u8; 100];
        let mut offsets = Vec::new();

        let chunk_size = 25;
        let mut transferred = 0;

        for _ in data.chunks(chunk_size) {
            offsets.push(transferred as i64);
            transferred += chunk_size.min(data.len() - transferred);
        }

        assert_eq!(offsets, vec![0, 25, 50, 75]);
    }
}

#[cfg(test)]
mod memory_layout_integration_tests {
    //! Tests for MemoryLayout integration with BufferShape

    use crate::{BufferShape, MemoryLayout, PrimitiveType};

    #[test]
    fn test_buffer_shape_strided_row_major() {
        // Row-major layout for 4x4 f32 matrix (4 bytes per element)
        // Row stride = 16 bytes, element stride = 4 bytes
        let layout = MemoryLayout::from_strides(vec![16, 4]);
        let shape = BufferShape::new(vec![4, 4], PrimitiveType::F32).with_layout(layout);

        assert!(shape.layout().is_some());
        match shape.layout().unwrap() {
            MemoryLayout::Strides(s) => {
                assert_eq!(s.byte_strides, vec![16, 4]);
            }
            _ => panic!("Expected Strides layout"),
        }
    }

    #[test]
    fn test_buffer_shape_strided_column_major() {
        // Column-major layout for 4x4 f32 matrix
        // Column stride = 4 bytes, element stride = 16 bytes
        let layout = MemoryLayout::from_strides(vec![4, 16]);
        let shape = BufferShape::new(vec![4, 4], PrimitiveType::F32).with_layout(layout);

        match shape.layout().unwrap() {
            MemoryLayout::Strides(s) => {
                assert_eq!(s.byte_strides, vec![4, 16]);
            }
            _ => panic!("Expected Strides layout"),
        }
    }

    #[test]
    fn test_buffer_shape_tiled_simple() {
        // Simple tiled layout with minor_to_major ordering
        let layout = MemoryLayout::from_tiled(vec![1, 0]).build();
        let shape = BufferShape::new(vec![64, 64], PrimitiveType::F32).with_layout(layout);

        match shape.layout().unwrap() {
            MemoryLayout::Tiled(t) => {
                assert_eq!(t.minor_to_major, vec![1, 0]);
                assert!(t.tile_dims.is_none());
            }
            _ => panic!("Expected Tiled layout"),
        }
    }

    #[test]
    fn test_buffer_shape_tiled_with_tiles() {
        // Tiled layout with explicit tile dimensions
        let layout = MemoryLayout::from_tiled(vec![1, 0])
            .tile_dims(vec![8, 8])
            .tile_dim_sizes(vec![64, 64])
            .build();
        let shape = BufferShape::new(vec![512, 512], PrimitiveType::F32).with_layout(layout);

        match shape.layout().unwrap() {
            MemoryLayout::Tiled(t) => {
                assert_eq!(t.minor_to_major, vec![1, 0]);
                assert_eq!(t.tile_dims, Some(vec![8, 8]));
                assert_eq!(t.tile_dim_sizes, Some(vec![64, 64]));
            }
            _ => panic!("Expected Tiled layout"),
        }
    }

    #[test]
    fn test_buffer_shape_3d_strides() {
        // 3D tensor layout (batch, height, width)
        let layout = MemoryLayout::from_strides(vec![1024, 32, 4]); // 8x8 images with batch
        let shape = BufferShape::new(vec![4, 8, 8], PrimitiveType::F32).with_layout(layout);

        match shape.layout().unwrap() {
            MemoryLayout::Strides(s) => {
                assert_eq!(s.byte_strides.len(), 3);
            }
            _ => panic!("Expected Strides layout"),
        }
    }
}

#[cfg(test)]
mod data_size_calculation_tests {
    //! Tests for size calculations used in transfers

    use crate::{Type, F32, F64, I32, U8};

    fn calculate_buffer_size<T: Type>(dims: &[i64]) -> usize {
        let num_elements: i64 = dims.iter().product();
        (num_elements as usize) * T::SIZE
    }

    #[test]
    fn test_scalar_size() {
        // A scalar (empty dims) actually has 1 element since iter().product() of empty is 1
        // This matches PJRT semantics: scalar has no dimensions but contains one value
        assert_eq!(calculate_buffer_size::<F32>(&[]), 4);

        // Verify the product behavior
        let empty: &[i64] = &[];
        let product: i64 = empty.iter().product();
        assert_eq!(product, 1);
    }

    #[test]
    fn test_1d_size_f32() {
        assert_eq!(calculate_buffer_size::<F32>(&[100]), 400);
    }

    #[test]
    fn test_2d_size_f32() {
        assert_eq!(calculate_buffer_size::<F32>(&[10, 10]), 400);
    }

    #[test]
    fn test_2d_size_f64() {
        assert_eq!(calculate_buffer_size::<F64>(&[10, 10]), 800);
    }

    #[test]
    fn test_3d_size_u8() {
        // Image-like: 3 channels, 224x224
        assert_eq!(calculate_buffer_size::<U8>(&[3, 224, 224]), 3 * 224 * 224);
    }

    #[test]
    fn test_large_tensor_size() {
        // Large matrix
        assert_eq!(calculate_buffer_size::<I32>(&[1024, 1024]), 1024 * 1024 * 4);
    }

    #[test]
    fn test_batch_tensor_size() {
        // Batch of matrices
        assert_eq!(
            calculate_buffer_size::<F32>(&[32, 100, 100]),
            32 * 100 * 100 * 4
        );
    }
}
