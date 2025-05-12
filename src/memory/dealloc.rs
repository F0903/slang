// DO NOT IMPLEMENT THIS TRAIT FOR STRUCTS THAT HAVE A DROP IMPLEMENTATION
pub trait Dealloc {
    fn dealloc(&mut self);
}

impl<T: Drop> !Dealloc for T {}
