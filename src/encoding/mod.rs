//TODO: Have another go at an EncodedDynArray that uses these.

pub use rle::RLE;

use crate::collections::DynArray;

mod rle;

pub trait Encoding {
    // Encodes sequence and returns the resulting array.
    fn encode(values: *const u8, count: usize) -> DynArray<u8>;

    // Encodes sequence, reallocates 'values' to new size, copies new values into it, and returns the new pointer.
    fn encode_replace(values: *mut u8, count: &mut usize) -> *mut u8;

    // Decodes sequence and returns the resulting array.
    fn decode(values: *const u8, count: usize) -> DynArray<u8>;

    // Decodes sequence, reallocates 'values' to new size, copies new values into it, and returns the new pointer.
    fn decode_replace(values: *mut u8, count: &mut usize) -> *mut u8;
}
