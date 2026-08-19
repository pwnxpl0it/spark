use std::path::PathBuf;
use thiserror::Error;

/// Represents all errors that can occur when using Spark as a library or CLI.
#[derive(Debug, Error)]
pub enum Error {
    /// Standard I/O failure (file read, directory creation, etc.).
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// Failed to deserialize a TOML template file or string.
    #[error("Failed to parse TOML template: {0}")]
    TomlDe(#[from] toml::de::Error),

    /// Failed to serialize a template structure into TOML.
    #[error("Failed to serialize TOML template: {0}")]
    TomlSer(#[from] toml::ser::Error),

    /// JSON parsing or serialization error.
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    /// Placeholder regex compilation error.
    #[error("Regex error: {0}")]
    Regex(#[from] regex::Error),

    /// Liquid templating engine rendering error.
    #[error("Liquid template error: {0}")]
    Liquid(#[from] liquid::Error),

    /// `jaq` JSON path filter evaluation error.
    #[error("JSON filter error: {0}")]
    JsonFilter(String),

    /// Required variable or project name was missing in non-interactive mode.
    #[error("Missing required variable '{0}'")]
    MissingVariable(String),

    /// Interactive terminal prompt failed.
    #[error("Interactive prompt failed: {0}")]
    Prompt(String),

    /// Failed to dispatch rendered output to a target sink (file, stdout, stderr, clipboard).
    #[error("Output write error for path '{path}': {message}")]
    OutputWrite {
        /// Target path / URI that caused the error.
        path: String,
        /// Description of the error.
        message: String,
    },

    /// Specified template path does not exist or is invalid.
    #[error("Invalid template path: {0:?}")]
    InvalidPath(PathBuf),

    /// Generic error message.
    #[error("{0}")]
    Custom(String),
}

/// Convenience type alias for `Result<T, spark::Error>`.
pub type Result<T> = std::result::Result<T, Error>;

impl From<String> for Error {
    fn from(msg: String) -> Self {
        Error::Custom(msg)
    }
}

impl From<&str> for Error {
    fn from(msg: &str) -> Self {
        Error::Custom(msg.to_string())
    }
}
