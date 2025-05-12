use {super::DynArray, crate::encoding::Encoding, std::ptr::addr_of};

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
            array: DynArray::new(None),
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
        if self.encoded {
            self.decode_all();
        }
        self.array.push_ptr(val, count);
    }

    pub fn encode_all(&mut self) {
        if self.encoded {
            self.decode_all();
        }

        let mut new_count = self.get_count();
        let old_data = self.array.get_raw_ptr();
        let new_data = E::encode_replace(old_data, &mut new_count); // Will dealloc old data
        self.array.set_backing_data(new_data, new_count, new_count);
        self.encoded = true;
    }

    pub fn decode_all(&mut self) {
        if !self.encoded {
            return;
        }

        let mut new_count = self.get_count();
        let old_data = self.array.get_raw_ptr();
        let new_data = E::decode_replace(old_data, &mut new_count); // Will dealloc old data
        self.array.set_backing_data(new_data, new_count, new_count);
        self.encoded = false;
    }

    pub fn read(&mut self, offset: usize) -> u32 {
        self.decode_all();
        *self.array.read_cast(offset)
    }
}
