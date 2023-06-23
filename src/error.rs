use glob::{GlobError, PatternError};

#[derive(thiserror::Error, Debug)]
pub enum DotrError {
    #[error("Generic {0}")]
    Generic(String),

    #[error("Path not found {0}")]
    PathNotFound(String),

    #[error("Could not access path from pattern {0}")]
    PathAccess(String, #[source] GlobError),

    #[error("Expected {0} to be a file")]
    NotFile(String),

    #[error("Expected {0} to be a directory")]
    NotDir(String),

    #[error("Bad glob {0}")]
    BadGlob(String, #[source] PatternError),

    #[error("Could not remove file {0}")]
    RemoveFile(String, #[source] std::io::Error),

    #[error("Could not create directory {0}")]
    CreateDir(String, #[source] std::io::Error),

    #[error("Could not copy file {0} to destination {1}")]
    CopyFile(String, String, #[source] std::io::Error),

    #[error(transparent)]
    IO(#[from] std::io::Error),
}
