use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Error, Debug)]
pub enum Error {
    #[error("encountered runtime error!\n\t{0}")]
    Runtime(String),
    #[error("encountered compile-time error!\n\t{0}")]
    CompileTime(String),
}
