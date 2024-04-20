use crate::dynarray::DynArray;

pub use encoded_dynarray::EncodedDynArray;
pub use rle::RLE;

mod encoded_dynarray;
mod rle;

pub trait Encoding {
    // Encodes sequence and returns the resulting array.
    unsafe fn encode(values: *const u8, count: usize) -> DynArray<u8>;

    // Encodes sequence, reallocates 'values' to new size, copies new values into it, and returns the new pointer.
    unsafe fn encode_realloc(values: *mut u8, count: &mut usize) -> *mut u8;

    // Decodes sequence and returns the resulting array.
    unsafe fn decode(values: *const u8, count: usize) -> DynArray<u8>;

    // Decodes sequence, reallocates 'values' to new size, copies new values into it, and returns the new pointer.
    unsafe fn decode_realloc(values: *mut u8, count: &mut usize) -> *mut u8;
}
