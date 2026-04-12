use thiserror::Error;

#[derive(Error, Debug)]
pub enum LoadLevelError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Parse error: {0}")]
    Parse(#[from] serde_json::Error),
    #[error("Entity types error: {0}")]
    EntityTypes(String),
}

