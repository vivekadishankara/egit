use thiserror::Error;

#[derive(Debug, Error)]
pub enum EgitError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Unauthorized")]
    Unauthorized,

    #[error("Git error: {0}")]
    Git(String),

    #[error("Internal error: {0}")]
    Internal(String),
}
