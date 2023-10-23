use std::ptr::addr_of;

use crate::{dynarray::DynArray, memory::reallocate};

pub trait Encoding {
    // Encodes sequence and returns the resulting array.
    unsafe fn encode(values: *const u8, count: usize) -> DynArray<u8>;

    // Encodes sequence, reallocates 'values' to new size, copies new values into it, and returns the new pointer.
    unsafe fn encode_replace(values: *mut u8, count: &mut usize) -> *mut u8;

    // Decodes sequence and returns the resulting array.
    unsafe fn decode(values: *const u8, count: usize) -> DynArray<u8>;

    // Decodes sequence, reallocates 'values' to new size, copies new values into it, and returns the new pointer.
    unsafe fn decode_replace(values: *mut u8, count: &mut usize) -> *mut u8;
}

pub struct RLE;
impl Encoding for RLE {
    unsafe fn encode(values: *const u8, count: usize) -> DynArray<u8> {
        // First byte is sequence count, next four is the number. And so on.

        println!("ENCODE START: ");
        for i in 0..count {
            print!("{}", values.add(i).read());
        }
        println!();

        let count_u32 = count / 4;
        let values_u32 = values.cast::<u32>();

        let mut current_num: u32 = values_u32.read();
        let mut current_num_count: u8 = 1;

        let mut workspace = DynArray::<u8>::new();

        for i in 1..count_u32 {
            let num = values_u32.add(i).read();
            if num as u32 != current_num {
                workspace.write(current_num_count);
                workspace.write_ptr(addr_of!(current_num).cast(), 4);

                current_num = num as u32;
                current_num_count = 1;
                continue;
            }

            current_num_count += 1;
        }
        workspace.write(current_num_count);
        workspace.write_ptr(addr_of!(current_num).cast(), 4);

        println!("ENCODE END: ");
        for i in 0..workspace.get_count() {
            print!("{}", workspace.get_raw_ptr().add(i).read());
        }
        println!();

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

        println!("ENCODE END: ");
        for i in 0..new_count {
            print!("{}", values.add(i).read());
        }
        println!();

        values
    }

    unsafe fn decode(values: *const u8, count: usize) -> DynArray<u8> {
        const SEQ_NUM_VALUE_ALIGNMENT: u8 = 5;

        let mut workspace = DynArray::<u8>::new();

        let loop_to = count / SEQ_NUM_VALUE_ALIGNMENT as usize;
        for i in 0..loop_to {
            let base = i * SEQ_NUM_VALUE_ALIGNMENT as usize;
            let seq_num = values.add(base).read();
            let num = values.add(base + 1).cast::<u32>();
            for _ in 0..seq_num {
                workspace.write_ptr(num.cast(), 4);
            }
        }

        println!("DECODE END: ");
        for i in 0..workspace.get_count() {
            print!("{}", workspace.get_raw_ptr().add(i).read());
        }
        println!();

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

        println!("DECODE END: ");
        for i in 0..new_count {
            print!("{}", values.add(i).read());
        }
        println!();

        values
    }
}

pub struct EncodedDynArray<E>
where
    E: Encoding,
{
    _encoding: std::marker::PhantomData<E>,
    array: DynArray<u8>,
    encoded: bool,
}

impl<E> EncodedDynArray<E>
where
    E: Encoding,
{
    pub fn new() -> Self {
        Self {
            _encoding: std::marker::PhantomData,
            array: DynArray::new(),
            encoded: false,
        }
    }

    pub const fn get_count(&self) -> usize {
        self.array.get_count()
    }

    pub const fn get_capacity(&self) -> usize {
        self.array.get_capacity()
    }

    pub fn write(&mut self, val: u32) {
        self.write_ptr(addr_of!(val).cast(), 4);
    }

    pub fn write_ptr(&mut self, val: *const u8, count: usize) {
        //TODO: Probably not the most efficient to be decoding and encoding all the time. Might find a better place to call these later on.
        if self.encoded {
            self.decode_all();
        }
        self.array.write_ptr(val, count);
        self.encode_all();
    }

    pub fn encode_all(&mut self) {
        if self.encoded {
            self.decode_all();
        }

        unsafe {
            let mut new_count = self.get_count();
            let new_data = E::encode_replace(self.array.get_raw_ptr(), &mut new_count);
            self.array.set_backing_data(new_data, new_count, new_count);
            self.encoded = true;
        }
    }

    pub fn decode_all(&mut self) {
        if !self.encoded {
            return;
        }

        unsafe {
            let mut new_count = self.get_count();
            let new_data = E::decode_replace(self.array.get_raw_ptr(), &mut new_count);
            self.array.set_backing_data(new_data, new_count, new_count);
            self.encoded = false;
        }
    }

    /// RETURNS VALUE POINTING INTO THE ARRAY
    pub fn read(&mut self, offset: usize) -> u32 {
        self.decode_all();
        self.array.read_cast(offset)
    }
}
