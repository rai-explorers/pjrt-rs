use std::vec;

use bon::bon;
use pjrt_sys::{
    PJRT_Buffer_MemoryLayout, PJRT_Buffer_MemoryLayout_Type,
    PJRT_Buffer_MemoryLayout_Type_PJRT_Buffer_MemoryLayout_Type_Strides,
    PJRT_Buffer_MemoryLayout_Type_PJRT_Buffer_MemoryLayout_Type_Tiled,
};

use crate::error::{Error, Result};

#[derive(Debug, Clone)]
pub enum MemoryLayout {
    Tiled(MemoryLayoutTiled),
    Strides(MemoryLayoutStrides),
}

#[bon]
impl MemoryLayout {
    #[builder(finish_fn = build)]
    pub fn tiled(
        #[builder(start_fn, into)] minor_to_major: Vec<i64>,
        #[builder] tile_dims: Option<Vec<i64>>,
        #[builder] tile_dim_sizes: Option<Vec<usize>>,
    ) -> MemoryLayout {
        MemoryLayout::Tiled(MemoryLayoutTiled {
            minor_to_major,
            tile_dims,
            tile_dim_sizes,
        })
    }

    pub fn strides(byte_strides: impl Into<Vec<i64>>) -> MemoryLayout {
        MemoryLayout::Strides(MemoryLayoutStrides {
            byte_strides: byte_strides.into(),
        })
    }
}

#[derive(Debug, Clone)]
pub struct MemoryLayoutTiled {
    pub minor_to_major: Vec<i64>,
    pub tile_dims: Option<Vec<i64>>,
    pub tile_dim_sizes: Option<Vec<usize>>,
}

#[derive(Debug, Clone)]
pub struct MemoryLayoutStrides {
    pub byte_strides: Vec<i64>,
}

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

// impl TryFrom<*mut PJRT_Buffer_MemoryLayout> for MemoryLayout {
//     type Error = Error;
//     fn try_from(layout: *mut PJRT_Buffer_MemoryLayout) -> Result<Self> {
//         if layout.is_null() {
//             return Err(Error::NullPointer);
//         }
//         let layout = unsafe { &*layout };
//         MemoryLayout::try_from(layout)
//     }
// }

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
