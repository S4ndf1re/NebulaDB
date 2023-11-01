use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("invalid upsert vector length for vector at index {index}. Expected {expected}, found {found}")]
    VectorLengthInvalid { index: usize, expected: usize, found: usize },
    #[error("option with_payload was set to true, meaning all upserted vectors must be paired with a payload")]
    MissingPayload,
    #[error("option with_payload was set to false, meaning all upserted vectors must NOT be paired with a payload")]
    ProvidedPayload,
    #[error("id '{0}' already exists")]
    IdAlreadyExists(usize),
}

pub type Result<T> = std::result::Result<T, Error>;
