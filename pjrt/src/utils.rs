use std::borrow::Cow;
use std::ffi::c_char;
use std::mem::ManuallyDrop;
use std::slice;

use pjrt_sys::PJRT_NamedValue;

use crate::NamedValueMap;

pub(crate) fn str_from_raw<'a>(ptr: *const c_char, size: usize) -> Cow<'a, str> {
    if ptr.is_null() {
        return Cow::Borrowed("");
    }
    let bytes = unsafe { slice::from_raw_parts(ptr as *const u8, size) };
    String::from_utf8_lossy(bytes)
}

pub(crate) fn into_raw_parts<T>(vec: Vec<T>) -> (*mut T, usize, usize) {
    let mut vec = ManuallyDrop::new(vec);
    let length = vec.len();
    let capacity = vec.capacity();
    (vec.as_mut_ptr(), length, capacity)
}

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

pub(super) fn to_named_value_map(values: *const PJRT_NamedValue, size: usize) -> NamedValueMap {
    if size == 0 || values.is_null() {
        NamedValueMap::new()
    } else {
        let attributes = unsafe { std::slice::from_raw_parts(values, size) };
        attributes.into()
    }
}
