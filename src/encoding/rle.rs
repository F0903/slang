use {
    super::Encoding,
    crate::{dynarray::DynArray, memory::reallocate},
    std::ptr::addr_of,
};

pub struct RLE;
impl Encoding for RLE {
    unsafe fn encode(values: *const u8, count: usize) -> DynArray<u8> {
        // First byte is sequence count, next four is the number. And so on.

        let count_u32 = count / 4;
        let values_u32 = values.cast::<u32>();

        let mut current_num: u32 = *values_u32;
        let mut current_num_count: u8 = 1;

        let mut workspace = DynArray::<u8>::new();

        for i in 1..count_u32 {
            let num = *values_u32.add(i);
            if num as u32 != current_num {
                workspace.push(current_num_count);
                workspace.push_ptr(addr_of!(current_num).cast(), 4);

                current_num = num as u32;
                current_num_count = 1;
                continue;
            }

            current_num_count += 1;
        }
        workspace.push(current_num_count);
        workspace.push_ptr(addr_of!(current_num).cast(), 4);

        workspace
    }

    unsafe fn encode_replace(values: *mut u8, count: &mut usize) -> *mut u8 {
        let workspace = Self::encode(values, *count);

        // Resize value array to the new encoded values.
        let new_count = workspace.get_count();
        let values = crate::memory::reallocate::<u8>(values, *count, new_count);
        *count = new_count;
        workspace
            .get_raw_ptr()
            .copy_to_nonoverlapping(values, new_count);

        values
    }

    unsafe fn decode(values: *const u8, count: usize) -> DynArray<u8> {
        const SEQ_NUM_VALUE_ALIGNMENT: u8 = 5;

        let mut workspace = DynArray::<u8>::new();

        let loop_to = count / SEQ_NUM_VALUE_ALIGNMENT as usize;
        for i in 0..loop_to {
            let base = i * SEQ_NUM_VALUE_ALIGNMENT as usize;
            let seq_num = *values.add(base);
            let num = values.add(base + 1).cast::<u32>();
            for _ in 0..seq_num {
                workspace.push_ptr(num.cast(), 4);
            }
        }

        workspace
    }

    unsafe fn decode_replace(values: *mut u8, count: &mut usize) -> *mut u8 {
        let workspace = Self::decode(values, *count);

        // Resize value array to the new decoded values.
        let new_count = workspace.get_count();
        let values = reallocate::<u8>(values, *count, new_count);
        *count = new_count;
        workspace
            .get_raw_ptr()
            .copy_to_nonoverlapping(values, new_count);

        values
    }
}
