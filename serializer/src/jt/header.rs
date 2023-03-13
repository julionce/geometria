use super::common::*;

pub struct Header {
    version: [u8; 80],
    byte_order: u8,
    empty_field: i32,
    toc_offset: u64,
    lsg_segment_id: GUID,
}
