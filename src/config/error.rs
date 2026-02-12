// Configuration error types

use thiserror::Error;

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("Failed to parse configuration: {0}")]
    ParseError(String),

    #[error("Configuration validation failed with {0} error(s)")]
    ValidationError(usize, Vec<ValidationError>),

    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),
}

impl ConfigError {
    /// Get validation errors if this is a ValidationError
    #[allow(dead_code)]
    pub fn validation_errors(&self) -> Option<&[ValidationError]> {
        match self {
            ConfigError::ValidationError(_, errors) => Some(errors),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ValidationError {
    pub field: String,
    pub error: String,
    pub location: Option<String>,
}

impl std::fmt::Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(loc) = &self.location {
            write!(f, "{}: {} ({})", self.field, self.error, loc)
        } else {
            write!(f, "{}: {}", self.field, self.error)
        }
    }
}
