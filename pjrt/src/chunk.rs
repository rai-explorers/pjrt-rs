use std::os::raw::c_void;

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chunk_new() {
        let data = vec![1u8, 2, 3, 4, 5];
        let chunk = Chunk::new(data.clone());
        assert_eq!(chunk.data, data);
    }

    #[test]
    fn test_chunk_empty() {
        let chunk = Chunk::new(vec![]);
        assert!(chunk.data.is_empty());
    }

    #[test]
    fn test_chunk_clone() {
        let data = vec![1u8, 2, 3, 4, 5];
        let chunk = Chunk::new(data);
        let cloned = chunk.clone();
        assert_eq!(chunk.data, cloned.data);
    }

    #[test]
    fn test_chunk_equality() {
        let chunk1 = Chunk::new(vec![1, 2, 3]);
        let chunk2 = Chunk::new(vec![1, 2, 3]);
        let chunk3 = Chunk::new(vec![4, 5, 6]);

        assert_eq!(chunk1, chunk2);
        assert_ne!(chunk1, chunk3);
    }

    #[test]
    fn test_chunk_ordering() {
        let chunk1 = Chunk::new(vec![1, 2, 3]);
        let chunk2 = Chunk::new(vec![1, 2, 4]);
        let chunk3 = Chunk::new(vec![2, 0, 0]);

        assert!(chunk1 < chunk2);
        assert!(chunk1 < chunk3);
        assert!(chunk2 < chunk3);
    }

    #[test]
    fn test_chunk_debug() {
        let chunk = Chunk::new(vec![1, 2, 3, 4, 5]);
        let debug = format!("{:?}", chunk);
        assert!(debug.contains("Chunk"));
        assert!(debug.contains("1"));
        assert!(debug.contains("5"));
    }

    #[test]
    fn test_chunk_hash() {
        use std::collections::HashSet;

        let mut set = HashSet::new();
        set.insert(Chunk::new(vec![1, 2, 3]));
        set.insert(Chunk::new(vec![4, 5, 6]));
        set.insert(Chunk::new(vec![1, 2, 3])); // Duplicate

        assert_eq!(set.len(), 2);
    }

    #[test]
    fn test_chunk_large_data() {
        let data: Vec<u8> = (0..1000).map(|i| (i % 256) as u8).collect();
        let chunk = Chunk::new(data.clone());
        assert_eq!(chunk.data.len(), 1000);
        assert_eq!(chunk.data, data);
    }
}
