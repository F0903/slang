use std::cell::RefCell;
use std::io::BufRead;
use std::rc::Rc;

type Result<T> = std::result::Result<T, std::io::Error>;

pub struct LineReader<'a> {
    reader: Rc<RefCell<dyn BufRead + 'a>>,
}

impl<'a> LineReader<'a> {
    pub fn new(reader: impl BufRead + 'a) -> Self {
        Self {
            reader: Rc::new(RefCell::new(reader)),
        }
    }

    pub fn read_line(&self, line_buf: &mut String) -> Result<usize> {
        self.reader.borrow_mut().read_line(line_buf)
    }
}

impl<'a> Iterator for LineReader<'a> {
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        let mut line_buf = String::default();
        match self.read_line(&mut line_buf) {
            Ok(x) => match x {
                0 => None,
                _ => Some(line_buf),
            },
            Err(_) => None,
        }
    }
}

impl<'a> Iterator for &LineReader<'a> {
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        let mut line_buf = String::default();
        match self.read_line(&mut line_buf) {
            Ok(x) => match x {
                0 => None,
                _ => Some(line_buf),
            },
            Err(_) => None,
        }
    }
}
