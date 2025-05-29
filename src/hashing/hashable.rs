// This is used as a simplified alternative to the `Hash` trait from the standard library.
pub trait Hashable {
    fn get_hash(&self) -> u32;
}
