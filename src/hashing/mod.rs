mod fnv1a;
mod hash_method;
mod hashable;

pub use fnv1a::FNV1a;
pub use hash_method::HashMethod;
pub use hashable::Hashable;

pub type GlobalHashMethod = FNV1a;
