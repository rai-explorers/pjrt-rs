use std::borrow::Cow;
use std::ffi::c_char;
use std::slice;

use pjrt_sys::PJRT_NamedValue;

use crate::{NamedValueMap, Result};

pub(crate) fn str_from_raw<'a>(ptr: *const c_char, size: usize) -> Cow<'a, str> {
    if ptr.is_null() {
        return Cow::Borrowed("");
    }
    let bytes = unsafe { slice::from_raw_parts(ptr as *const u8, size) };
    String::from_utf8_lossy(bytes)
}

/// Decompose a 2D C-style pointer array into a `Vec<Vec<U>>`.
///
/// # Safety
///
/// - `list` must point to a valid array of `num_rows` pointers.
/// - Each inner pointer must point to a valid array of `num_cols` `*mut T` pointers.
/// - All pointers must be valid for reads and must not be aliased mutably.
pub(crate) unsafe fn slice_to_vec2d<T, U>(
    list: *const *mut *mut T,
    num_rows: usize,
    num_cols: usize,
    func: impl Fn(*mut T) -> U,
) -> Vec<Vec<U>> {
    let mut outer_vec: Vec<Vec<U>> = Vec::with_capacity(num_rows);
    for i in 0..num_rows {
        let inner = *list.add(i);
        let mut inner_vec: Vec<U> = Vec::with_capacity(num_cols);
        for j in 0..num_cols {
            let ptr = *inner.add(j);
            inner_vec.push(func(ptr));
        }
        outer_vec.push(inner_vec);
    }
    outer_vec
}

pub(crate) fn byte_strides(shape: &[i64], elem_ty_size: usize) -> Vec<i64> {
    let mut strides = vec![0; shape.len()];
    let mut current_stride = elem_ty_size as i64;

    for i in (0..shape.len()).rev() {
        strides[i] = current_stride;
        current_stride *= shape[i];
    }

    strides
}

pub(super) fn to_named_value_map(
    values: *const PJRT_NamedValue,
    size: usize,
) -> Result<NamedValueMap> {
    if size == 0 || values.is_null() {
        Ok(NamedValueMap::new())
    } else {
        let attributes = unsafe { std::slice::from_raw_parts(values, size) };
        Ok(attributes.try_into()?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_str_from_raw_empty() {
        let s = str_from_raw(std::ptr::null(), 0);
        assert_eq!(s, "");
    }

    #[test]
    fn test_str_from_raw_valid() {
        let bytes = b"hello world";
        let s = str_from_raw(bytes.as_ptr() as *const c_char, bytes.len());
        assert_eq!(s, "hello world");
    }

    #[test]
    fn test_str_from_raw_with_null() {
        // Test with null pointer returns empty string
        let s = str_from_raw(std::ptr::null(), 10);
        assert_eq!(s, "");
    }

    #[test]
    fn test_byte_strides_1d() {
        // 1D shape [5], element size 4 (f32)
        let strides = byte_strides(&[5], 4);
        assert_eq!(strides, vec![4]);
    }

    #[test]
    fn test_byte_strides_2d() {
        // 2D shape [3, 4], element size 4 (f32)
        // Strides should be [16, 4] (row-major)
        let strides = byte_strides(&[3, 4], 4);
        assert_eq!(strides, vec![16, 4]);
    }

    #[test]
    fn test_byte_strides_3d() {
        // 3D shape [2, 3, 4], element size 8 (f64)
        // Strides should be [96, 32, 8] (row-major)
        let strides = byte_strides(&[2, 3, 4], 8);
        assert_eq!(strides, vec![96, 32, 8]);
    }

    #[test]
    fn test_byte_strides_empty() {
        let strides = byte_strides(&[], 4);
        assert!(strides.is_empty());
    }

    #[test]
    fn test_byte_strides_different_element_sizes() {
        // Same shape, different element sizes
        let shape = &[2, 3];

        let strides_u8 = byte_strides(shape, 1);
        assert_eq!(strides_u8, vec![3, 1]);

        let strides_f32 = byte_strides(shape, 4);
        assert_eq!(strides_f32, vec![12, 4]);

        let strides_f64 = byte_strides(shape, 8);
        assert_eq!(strides_f64, vec![24, 8]);
    }

    #[test]
    fn test_to_named_value_map_empty() {
        let map = to_named_value_map(std::ptr::null(), 0).unwrap();
        assert!(map.into_inner().is_empty());

        let map = to_named_value_map(std::ptr::null(), 5).unwrap();
        assert!(map.into_inner().is_empty());
    }
}
