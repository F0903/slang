/// Trait for types that require manual deallocation.
/// BE CAREFUL WITH DOUBLE FREEING OR MEMORY LEAKS!
pub trait Dealloc {
    fn dealloc(&mut self);
}
