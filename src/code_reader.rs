use std::cell::RefCell;
use std::fs::File;
use std::io::Read;

type Result<T> = std::result::Result<T, String>;

pub struct CodeReader {
    lines: Vec<String>,
    index: RefCell<usize>,
}

impl CodeReader {
    pub fn from_str(code: impl AsRef<str>) -> Self {
        let code = code.as_ref();
        let lines: Vec<String> = code.lines().map(|x| x.to_owned()).collect();
        Self {
            lines,
            index: RefCell::new(0),
        }
    }

    pub fn from_file(mut code: File) -> Self {
        let mut str_buf = String::default();
        code.read_to_string(&mut str_buf)
            .expect("Could not read code file to string!");
        Self::from_str(str_buf)
    }

    pub fn get_index(&self) -> usize {
        self.index.borrow().clone()
    }

    pub fn seek(&mut self, to: usize) {
        *self.index.borrow_mut() = to;
    }

    pub fn get_next(&self) -> Option<&String> {
        let line = self.lines.get(self.index.borrow().clone());
        *self.index.borrow_mut() += 1;
        line
    }

    pub fn read_line(&mut self, buf: &mut String) -> Result<usize> {
        let line = self.get_next().ok_or("Could not get next line.")?;
        let len = line.len();
        *buf = line.clone();
        Ok(len)
    }
}

impl Iterator for CodeReader {
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        self.get_next().map(|x| x.clone())
    }
}

impl Iterator for &CodeReader {
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        self.get_next().map(|x| x.clone())
    }
}
