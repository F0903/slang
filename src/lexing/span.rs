#[derive(Clone, Debug)]
pub struct Span {
    start_index: usize,
    end_index: usize,
    #[cfg(debug_assertions)]
    hash: u32,
}

impl Span {
    #[cfg(debug_assertions)]
    pub fn new(start_index: usize, end_index: usize, hash: u32) -> Self {
        Self {
            start_index,
            end_index,
            hash,
        }
    }

    #[cfg(not(debug_assertions))]
    pub fn new(start_index: usize, end_index: usize) -> Self {
        Self {
            start_index,
            end_index,
        }
    }

    #[cfg(debug_assertions)]
    pub fn get_str<'a>(&self, source: &'a [u8]) -> &'a str {
        use crate::hashing::{GlobalHashMethod, HashMethod};

        assert!(
            source.len() >= self.end_index,
            "Span index is out of bounds on source string! (is a wrong source buffer being provided?)"
        );
        let byte_slice = &source[self.start_index..self.end_index];
        assert!(self.hash == GlobalHashMethod::hash(byte_slice));

        // SAFETY: This is guaranteed to be in-bounds and valid here.
        let lexeme_str = unsafe { std::str::from_utf8_unchecked(byte_slice) };
        // When we are running in debug mode, make sure that the hash matches
        lexeme_str
    }

    #[cfg(not(debug_assertions))]
    #[inline]
    pub fn get_str<'a>(&self, source: &'a [u8]) -> &'a str {
        // SAFETY: As long as the source buffer is the same as the one this was created fron, this is safe.
        unsafe { std::str::from_utf8_unchecked(&source[self.start_index..self.end_index]) }
    }
}
