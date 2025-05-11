pub trait HashMethod {
    fn hash(data: &[u8]) -> u32;
}
