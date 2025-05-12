mod fnv1a;
mod hash_method;

pub use fnv1a::FNV1a;
pub use hash_method::HashMethod;

pub type GlobalHashMethod = FNV1a;
