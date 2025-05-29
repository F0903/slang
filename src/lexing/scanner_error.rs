use std::{error::Error, fmt::Display};

#[derive(Debug)]
pub struct ScannerError {
    message: String,
}

impl ScannerError {
    pub fn new(message: impl ToString) -> Self {
        Self {
            message: message.to_string(),
        }
    }

    pub fn get_message(&self) -> &str {
        &self.message
    }
}

impl Error for ScannerError {}

impl Display for ScannerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("SCANNER ERROR: {}", self.message))
    }
}

impl From<String> for ScannerError {
    fn from(value: String) -> Self {
        Self::new(value)
    }
}

impl From<&str> for ScannerError {
    fn from(value: &str) -> Self {
        Self::new(value)
    }
}
