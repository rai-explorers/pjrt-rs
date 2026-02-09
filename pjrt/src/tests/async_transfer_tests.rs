//! Unit Tests for Async Transfer Module
//!
//! These tests verify `BufferShape` and `MemoryLayout` integration.
//! Full transfer tests require a PJRT plugin — see `examples/async_transfer.rs`.

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
    }

    #[test]
    fn test_buffer_shape_chaining() {
        let layout = MemoryLayout::from_strides(vec![4]);
        let shape = BufferShape::new(vec![100], PrimitiveType::S32).with_layout(layout);

        assert_eq!(shape.dims().len(), 1);
        assert!(shape.layout().is_some());
    }

    #[test]
    fn test_buffer_shape_large_dimensions() {
        let dims = vec![1024, 1024, 3];
        let shape = BufferShape::new(dims.clone(), PrimitiveType::U8);
        assert_eq!(shape.dims(), dims.as_slice());
    }

    #[test]
    fn test_buffer_shape_single_element_dims() {
        let shape = BufferShape::new(vec![1, 1, 1, 1], PrimitiveType::F32);
        assert_eq!(shape.dims(), &[1, 1, 1, 1]);
    }
}

#[cfg(test)]
mod memory_layout_integration_tests {
    use crate::{BufferShape, MemoryLayout, PrimitiveType};

    #[test]
    fn test_buffer_shape_strided_row_major() {
        let layout = MemoryLayout::from_strides(vec![16, 4]);
        let shape = BufferShape::new(vec![4, 4], PrimitiveType::F32).with_layout(layout);

        match shape.layout().unwrap() {
            MemoryLayout::Strides(s) => {
                assert_eq!(s.byte_strides, vec![16, 4]);
            }
            _ => panic!("Expected Strides layout"),
        }
    }

    #[test]
    fn test_buffer_shape_strided_column_major() {
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
        let layout = MemoryLayout::from_strides(vec![1024, 32, 4]);
        let shape = BufferShape::new(vec![4, 8, 8], PrimitiveType::F32).with_layout(layout);

        match shape.layout().unwrap() {
            MemoryLayout::Strides(s) => {
                assert_eq!(s.byte_strides.len(), 3);
            }
            _ => panic!("Expected Strides layout"),
        }
    }
}
