use thiserror::Error;

#[derive(Error, Debug, Clone)]
pub enum LoadLevelError {
    #[error("IO error: {0}")]
    Io(String),
    #[error("Parse error: {0}")]
    Parse(String),
    #[error("Entity types error: {0}")]
    EntityTypes(String),
}

impl From<std::io::Error> for LoadLevelError {
    fn from(e: std::io::Error) -> Self { LoadLevelError::Io(e.to_string()) }
}

impl From<serde_json::Error> for LoadLevelError {
    fn from(e: serde_json::Error) -> Self { LoadLevelError::Parse(e.to_string()) }
}

