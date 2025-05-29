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

        let lexeme_str =
            unsafe { std::str::from_utf8_unchecked(&source[self.start_index..self.end_index]) };
        // When we are running in debug mode, make sure that the hash matches
        debug_assert!(self.hash == GlobalHashMethod::hash(lexeme_str.as_bytes()));
        lexeme_str
    }

    #[cfg(not(debug_assertions))]
    pub fn get_str<'a>(&self, source: &'a [u8]) -> &'a str {
        unsafe { std::str::from_utf8_unchecked(&source[self.start_index..self.end_index]) }
    }
}
