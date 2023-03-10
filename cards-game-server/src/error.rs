use thiserror::Error;

#[derive(Error, Debug)]
pub enum GameError {
    #[error("Couldn't serialize to JSON")]
    MessageSerialization(#[from] serde_json::error::Error),
}
