#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(dead_code)]
#![allow(clippy::too_long_first_doc_paragraph)]
#![allow(clippy::doc_lazy_continuation)]
#![allow(clippy::doc_overindented_list_items)]

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

pub mod protos;
mod structs;
