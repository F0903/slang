use super::HashMethod;

#[derive(Debug)]
pub struct FNV1a;

impl HashMethod for FNV1a {
    fn hash(data: &[u8]) -> u32 {
        let mut hash = 2166136261u32;
        for &byte in data {
            hash ^= byte as u32;
            hash = hash.wrapping_mul(16777619);
        }
        hash
    }
}
