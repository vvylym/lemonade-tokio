//! Config Error module
//!
use std::path::PathBuf;

/// Config error enum
#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    /// File not found
    #[error("file not found: {0}")]
    FileNotFound(PathBuf),
    /// IO error
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    /// JSON parse error
    #[error("JSON parse error: {0}")]
    Json(#[from] serde_json::Error),
    /// TOML parse error
    #[error("TOML parse error: {0}")]
    Toml(#[from] toml::de::Error),
    /// YAML parse error
    #[error("YAML parse error: {0}")]
    Yaml(#[from] serde_yaml::Error),
    /// Notify watcher error
    #[error("Notify watcher error: {0}")]
    Notify(#[from] notify::Error),
    /// Unsupported file format
    #[error("Unsupported file format: {0}. Supported formats: .json, .toml, .yaml, .yml")]
    UnsupportedFormat(String),
    /// Parse error
    #[error("Parse error: {0}")]
    Parse(String),
}
