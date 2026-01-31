//! PJRT Memory Layout
//!
//! This module provides types for describing how data is laid out in memory.
//! Memory layout is crucial for:
//!
//! - Optimizing data access patterns on accelerators
//! - Handling different memory formats (row-major, column-major, tiled)
//! - Converting between host and device memory layouts
//!
//! The module provides:
//!
//! - `MemoryLayout`: An enum representing different layout types
//! - `MemoryLayoutTiled`: Tiled memory layout for optimized accelerator access
//! - `MemoryLayoutStrides`: Strided layout with custom byte strides
//! - `MemoryLayoutType`: Enum discriminant for layout types
//!
//! # Example
//!
//! ```rust,ignore
//! // Create a tiled layout for accelerator-optimized access
//! let layout = MemoryLayout::from_tiled(vec![0, 1, 2])
//!     .tile_dims(vec![8, 8])
//!     .tile_dim_sizes(vec![64, 64])
//!     .build();
//!
//! // Create a strided layout
//! let layout = MemoryLayout::from_strides(vec![32, 8, 4]);
//! ```

use std::vec;

use bon::bon;
use pjrt_sys::{
    PJRT_Buffer_MemoryLayout, PJRT_Buffer_MemoryLayout_Type,
    PJRT_Buffer_MemoryLayout_Type_PJRT_Buffer_MemoryLayout_Type_Strides,
    PJRT_Buffer_MemoryLayout_Type_PJRT_Buffer_MemoryLayout_Type_Tiled,
};

use crate::error::{Error, Result};

/// Represents different memory layout strategies for PJRT buffers.
///
/// Memory layout describes how multi-dimensional data is organized in memory.
/// Different layouts can significantly impact performance on different hardware.
///
/// # Variants
///
/// - `Tiled`: Tiled layout optimized for accelerator memory access patterns
/// - `Strides`: Strided layout with custom byte strides for each dimension
#[derive(Debug, Clone)]
pub enum MemoryLayout {
    Tiled(MemoryLayoutTiled),
    Strides(MemoryLayoutStrides),
}

#[bon]
impl MemoryLayout {
    #[builder(finish_fn = build)]
    pub fn from_tiled(
        #[builder(start_fn, into)] minor_to_major: Vec<i64>,
        tile_dims: Option<Vec<i64>>,
        tile_dim_sizes: Option<Vec<usize>>,
    ) -> MemoryLayout {
        MemoryLayout::Tiled(MemoryLayoutTiled {
            minor_to_major,
            tile_dims,
            tile_dim_sizes,
        })
    }

    pub fn from_strides(byte_strides: impl Into<Vec<i64>>) -> MemoryLayout {
        MemoryLayout::Strides(MemoryLayoutStrides {
            byte_strides: byte_strides.into(),
        })
    }
}

/// Tiled memory layout for optimized accelerator access.
///
/// Tiled layouts organize data in tiles (blocks) that match the access patterns
/// of accelerators like GPUs and TPUs, improving cache locality and performance.
///
/// # Fields
///
/// - `minor_to_major`: Dimension ordering from minor (fastest changing) to major
/// - `tile_dims`: Optional tile dimensions for each axis
/// - `tile_dim_sizes`: Optional sizes for each tile dimension
#[derive(Debug, Clone)]
pub struct MemoryLayoutTiled {
    pub minor_to_major: Vec<i64>,
    pub tile_dims: Option<Vec<i64>>,
    pub tile_dim_sizes: Option<Vec<usize>>,
}

/// Strided memory layout with custom byte strides.
///
/// Strided layouts specify the number of bytes between consecutive elements
/// in each dimension, allowing for flexible data organization including
/// row-major, column-major, and arbitrary strided layouts.
///
/// # Fields
///
/// - `byte_strides`: The byte stride for each dimension
#[derive(Debug, Clone)]
pub struct MemoryLayoutStrides {
    pub byte_strides: Vec<i64>,
}

/// Enum discriminant for memory layout types.
///
/// Used to identify whether a memory layout is tiled or strided.
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum MemoryLayoutType {
    Tiled = PJRT_Buffer_MemoryLayout_Type_PJRT_Buffer_MemoryLayout_Type_Tiled as i32,
    Strides = PJRT_Buffer_MemoryLayout_Type_PJRT_Buffer_MemoryLayout_Type_Strides as i32,
}

impl TryFrom<PJRT_Buffer_MemoryLayout_Type> for MemoryLayoutType {
    type Error = Error;

    #[allow(non_upper_case_globals)]
    #[allow(non_snake_case)]
    fn try_from(value: PJRT_Buffer_MemoryLayout_Type) -> Result<Self> {
        match value {
            PJRT_Buffer_MemoryLayout_Type_PJRT_Buffer_MemoryLayout_Type_Tiled => {
                Ok(MemoryLayoutType::Tiled)
            }
            PJRT_Buffer_MemoryLayout_Type_PJRT_Buffer_MemoryLayout_Type_Strides => {
                Ok(MemoryLayoutType::Strides)
            }
            _ => Err(Error::InvalidMemoryLayoutType(value as i32)),
        }
    }
}

impl<'a> TryFrom<&'a PJRT_Buffer_MemoryLayout> for MemoryLayout {
    type Error = Error;

    fn try_from(layout: &'a PJRT_Buffer_MemoryLayout) -> std::result::Result<Self, Self::Error> {
        let layout_ty = MemoryLayoutType::try_from(layout.type_)?;
        match layout_ty {
            MemoryLayoutType::Tiled => {
                let tiled = unsafe { layout.__bindgen_anon_1.tiled };
                let minor_to_major = if tiled.minor_to_major_size == 0 {
                    vec![]
                } else {
                    let minor_to_major: &[i64] = unsafe {
                        std::slice::from_raw_parts(tiled.minor_to_major, tiled.minor_to_major_size)
                    };
                    minor_to_major.to_vec()
                };
                if tiled.num_tiles == 0 {
                    let layout = MemoryLayoutTiled {
                        minor_to_major,
                        tile_dims: None,
                        tile_dim_sizes: None,
                    };
                    Ok(MemoryLayout::Tiled(layout))
                } else {
                    let tile_dims: &[i64] =
                        unsafe { std::slice::from_raw_parts(tiled.tile_dims, tiled.num_tiles) };
                    let tile_dim_sizes = unsafe {
                        std::slice::from_raw_parts(tiled.tile_dim_sizes, tiled.num_tiles)
                    };
                    let layout = MemoryLayoutTiled {
                        minor_to_major,
                        tile_dims: Some(tile_dims.to_vec()),
                        tile_dim_sizes: Some(tile_dim_sizes.to_vec()),
                    };
                    Ok(MemoryLayout::Tiled(layout))
                }
            }
            MemoryLayoutType::Strides => {
                let byte_strides: &[i64] = unsafe {
                    std::slice::from_raw_parts(
                        layout.__bindgen_anon_1.strides.byte_strides,
                        layout.__bindgen_anon_1.strides.num_byte_strides,
                    )
                };
                let layout = MemoryLayoutStrides {
                    byte_strides: byte_strides.to_vec(),
                };
                Ok(MemoryLayout::Strides(layout))
            }
        }
    }
}

impl<'a> From<&'a MemoryLayout> for PJRT_Buffer_MemoryLayout {
    fn from(layout: &'a MemoryLayout) -> Self {
        match layout {
            MemoryLayout::Tiled(layout) => PJRT_Buffer_MemoryLayout::from(layout),
            MemoryLayout::Strides(layout) => PJRT_Buffer_MemoryLayout::from(layout),
        }
    }
}

impl<'a> From<&'a MemoryLayoutTiled> for PJRT_Buffer_MemoryLayout {
    fn from(layout: &'a MemoryLayoutTiled) -> Self {
        let mut pjrt_layout = PJRT_Buffer_MemoryLayout::new();
        pjrt_layout.type_ = MemoryLayoutType::Tiled as PJRT_Buffer_MemoryLayout_Type;
        pjrt_layout.__bindgen_anon_1.tiled.minor_to_major = layout.minor_to_major.as_ptr();
        pjrt_layout.__bindgen_anon_1.tiled.minor_to_major_size = layout.minor_to_major.len();
        if let Some(tile_dims) = &layout.tile_dims {
            pjrt_layout.__bindgen_anon_1.tiled.tile_dims = tile_dims.as_ptr();
            pjrt_layout.__bindgen_anon_1.tiled.num_tiles = tile_dims.len();
        }
        if let Some(tile_dim_sizes) = &layout.tile_dim_sizes {
            pjrt_layout.__bindgen_anon_1.tiled.tile_dim_sizes = tile_dim_sizes.as_ptr();
        }
        pjrt_layout
    }
}

impl<'a> From<&'a MemoryLayoutStrides> for PJRT_Buffer_MemoryLayout {
    fn from(layout: &'a MemoryLayoutStrides) -> Self {
        let mut pjrt_layout = PJRT_Buffer_MemoryLayout::new();
        pjrt_layout.type_ = MemoryLayoutType::Strides as PJRT_Buffer_MemoryLayout_Type;
        pjrt_layout.__bindgen_anon_1.strides.byte_strides = layout.byte_strides.as_ptr();
        pjrt_layout.__bindgen_anon_1.strides.num_byte_strides = layout.byte_strides.len();
        pjrt_layout
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_layout_tiled_creation() {
        let layout = MemoryLayout::from_tiled(vec![0, 1, 2]).build();
        match layout {
            MemoryLayout::Tiled(tiled) => {
                assert_eq!(tiled.minor_to_major, vec![0, 1, 2]);
                assert!(tiled.tile_dims.is_none());
                assert!(tiled.tile_dim_sizes.is_none());
            }
            _ => panic!("Expected Tiled layout"),
        }
    }

    #[test]
    fn test_memory_layout_tiled_with_tile_dims() {
        let layout = MemoryLayout::from_tiled(vec![0, 1])
            .tile_dims(vec![8, 8])
            .tile_dim_sizes(vec![64, 64])
            .build();
        match layout {
            MemoryLayout::Tiled(tiled) => {
                assert_eq!(tiled.minor_to_major, vec![0, 1]);
                assert_eq!(tiled.tile_dims, Some(vec![8, 8]));
                assert_eq!(tiled.tile_dim_sizes, Some(vec![64, 64]));
            }
            _ => panic!("Expected Tiled layout"),
        }
    }

    #[test]
    fn test_memory_layout_strides() {
        let layout = MemoryLayout::from_strides(vec![32, 8, 4]);
        match layout {
            MemoryLayout::Strides(strides) => {
                assert_eq!(strides.byte_strides, vec![32, 8, 4]);
            }
            _ => panic!("Expected Strides layout"),
        }
    }

    #[test]
    fn test_memory_layout_clone() {
        let layout = MemoryLayout::from_strides(vec![16, 4]);
        let cloned = layout.clone();
        assert_eq!(format!("{:?}", layout), format!("{:?}", cloned));
    }

    #[test]
    fn test_memory_layout_debug() {
        let tiled = MemoryLayout::from_tiled(vec![0, 1]).build();
        let debug = format!("{:?}", tiled);
        assert!(debug.contains("Tiled"));

        let strides = MemoryLayout::from_strides(vec![32, 8]);
        let debug = format!("{:?}", strides);
        assert!(debug.contains("Strides"));
    }

    #[test]
    fn test_memory_layout_type_values() {
        assert_eq!(
            MemoryLayoutType::Tiled as i32,
            PJRT_Buffer_MemoryLayout_Type_PJRT_Buffer_MemoryLayout_Type_Tiled as i32
        );
        assert_eq!(
            MemoryLayoutType::Strides as i32,
            PJRT_Buffer_MemoryLayout_Type_PJRT_Buffer_MemoryLayout_Type_Strides as i32
        );
    }

    #[test]
    fn test_memory_layout_type_equality() {
        assert_eq!(MemoryLayoutType::Tiled, MemoryLayoutType::Tiled);
        assert_eq!(MemoryLayoutType::Strides, MemoryLayoutType::Strides);
        assert_ne!(MemoryLayoutType::Tiled, MemoryLayoutType::Strides);
    }

    #[test]
    fn test_memory_layout_type_ordering() {
        assert!(MemoryLayoutType::Tiled < MemoryLayoutType::Strides);
    }

    #[test]
    fn test_memory_layout_type_hash() {
        use std::collections::HashSet;
        let mut set = HashSet::new();
        set.insert(MemoryLayoutType::Tiled);
        set.insert(MemoryLayoutType::Strides);
        set.insert(MemoryLayoutType::Tiled); // Duplicate
        assert_eq!(set.len(), 2);
    }

    #[test]
    fn test_memory_layout_type_try_from_valid() {
        use pjrt_sys::{
            PJRT_Buffer_MemoryLayout_Type_PJRT_Buffer_MemoryLayout_Type_Strides,
            PJRT_Buffer_MemoryLayout_Type_PJRT_Buffer_MemoryLayout_Type_Tiled,
        };

        let ty: MemoryLayoutType =
            PJRT_Buffer_MemoryLayout_Type_PJRT_Buffer_MemoryLayout_Type_Tiled
                .try_into()
                .unwrap();
        assert_eq!(ty, MemoryLayoutType::Tiled);

        let ty: MemoryLayoutType =
            PJRT_Buffer_MemoryLayout_Type_PJRT_Buffer_MemoryLayout_Type_Strides
                .try_into()
                .unwrap();
        assert_eq!(ty, MemoryLayoutType::Strides);
    }

    #[test]
    fn test_memory_layout_type_try_from_invalid() {
        // Test with an invalid value using the raw type
        use pjrt_sys::PJRT_Buffer_MemoryLayout_Type;
        let invalid_type: PJRT_Buffer_MemoryLayout_Type = 999;
        let result: Result<MemoryLayoutType> = invalid_type.try_into();
        assert!(result.is_err());
    }

    #[test]
    fn test_memory_layout_tiled_struct() {
        let tiled = MemoryLayoutTiled {
            minor_to_major: vec![0, 1, 2],
            tile_dims: Some(vec![8, 8]),
            tile_dim_sizes: Some(vec![64, 64]),
        };
        assert_eq!(tiled.minor_to_major, vec![0, 1, 2]);
        assert_eq!(tiled.tile_dims, Some(vec![8, 8]));
        assert_eq!(tiled.tile_dim_sizes, Some(vec![64, 64]));
    }

    #[test]
    fn test_memory_layout_strides_struct() {
        let strides = MemoryLayoutStrides {
            byte_strides: vec![128, 64, 32, 16],
        };
        assert_eq!(strides.byte_strides, vec![128, 64, 32, 16]);
    }

    #[test]
    fn test_memory_layout_strides_empty() {
        let layout = MemoryLayout::from_strides(Vec::<i64>::new());
        match layout {
            MemoryLayout::Strides(strides) => {
                assert!(strides.byte_strides.is_empty());
            }
            _ => panic!("Expected Strides layout"),
        }
    }

    #[test]
    fn test_memory_layout_tiled_clone() {
        let tiled = MemoryLayoutTiled {
            minor_to_major: vec![0, 1],
            tile_dims: Some(vec![8, 8]),
            tile_dim_sizes: Some(vec![64]),
        };
        let cloned = tiled.clone();
        assert_eq!(tiled.minor_to_major, cloned.minor_to_major);
        assert_eq!(tiled.tile_dims, cloned.tile_dims);
        assert_eq!(tiled.tile_dim_sizes, cloned.tile_dim_sizes);
    }

    #[test]
    fn test_memory_layout_strides_clone() {
        let strides = MemoryLayoutStrides {
            byte_strides: vec![32, 8, 4],
        };
        let cloned = strides.clone();
        assert_eq!(strides.byte_strides, cloned.byte_strides);
    }
}
