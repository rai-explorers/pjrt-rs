use ::std::os::raw::c_void;
use pjrt_sys::PJRT_Chunk;

use crate::utils;

unsafe extern "C" fn chunk_deleter(data: *mut c_void, deleter_arg: *mut c_void) {
    let (len, cap) = *Box::from_raw(deleter_arg as *mut (usize, usize));
    let _ = Vec::from_raw_parts(data as *mut u8, len, cap);
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Chunk {
    data: Vec<u8>,
}

impl Chunk {
    pub fn new(data: Vec<u8>) -> Self {
        Self { data }
    }
}

impl From<Chunk> for PJRT_Chunk {
    fn from(chunk: Chunk) -> Self {
        let mut c = PJRT_Chunk::default();
        let (ptr, len, cap) = utils::into_raw_parts(chunk.data);
        c.data = ptr as *mut _;
        c.size = len;
        c.deleter = Some(chunk_deleter);
        let delete_args = Box::new((len, cap));
        c.deleter_arg = Box::into_raw(delete_args) as *mut _;
        c
    }
}
