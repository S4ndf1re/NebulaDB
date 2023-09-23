use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("invalid upsert vector length for vector at index {index}. Expected {expected}, found {found}")]
    VectorLengthInvalid { index: usize, expected: usize, found: usize },
}

pub type Result<T> = std::result::Result<T, Error>;
