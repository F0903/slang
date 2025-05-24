use super::DynArray;
use crate::encoding::Encoding;

//TODO: Redo this whole thing
pub struct EncodedDynArray<T, E>
where
    T: std::fmt::Debug,
    E: Encoding,
{
    _actual_type: std::marker::PhantomData<T>,
    _encoding: std::marker::PhantomData<E>,
    data: DynArray<T>,
    encoded: bool,
}

impl<T, E> EncodedDynArray<T, E>
where
    T: std::fmt::Debug,
    E: Encoding,
{
    pub fn new() -> Self {
        Self {
            _actual_type: std::marker::PhantomData,
            _encoding: std::marker::PhantomData,
            data: DynArray::new(None),
            encoded: false,
        }
    }

    pub const fn get_count(&self) -> usize {
        self.data.get_count()
    }

    pub const fn get_capacity(&self) -> usize {
        self.data.get_capacity()
    }

    pub fn write(&mut self, val: T) {
        if self.encoded {
            self.decode_all();
        }
        self.data.push(val);
    }

    pub fn encode_all(&mut self) {
        if self.encoded {
            self.decode_all();
        }

        let mut new_count = self.get_count();
        let old_data = self.data.get_raw_ptr();
        let new_data = E::encode_replace(old_data.cast(), &mut new_count);
        self.data
            .set_backing_data(new_data.cast(), new_count, new_count);
        self.encoded = true;
    }

    pub fn decode_all(&mut self) {
        if !self.encoded {
            return;
        }

        let mut new_count = self.get_count();
        let old_data = self.data.get_raw_ptr();
        let new_data = E::decode_replace(old_data.cast(), &mut new_count); // Will dealloc old data
        self.data
            .set_backing_data(new_data.cast(), new_count, new_count);
        self.encoded = false;
    }

    pub fn read(&mut self, index: usize) -> &T {
        self.decode_all();
        self.data.read(index)
    }

    pub fn copy_read(&mut self, index: usize) -> T {
        self.decode_all();
        self.data.copy_read(index)
    }
}
