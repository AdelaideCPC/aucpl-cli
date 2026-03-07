use std::io;
use std::path::PathBuf;

use owo_colors::OwoColorize;
use thiserror::Error;

/// A simplified error type for the CLI with main message, verbose details, and suggestions.
#[derive(Error, Debug)]
#[allow(dead_code)]
pub enum CliError {
    /// A required file or directory was not found
    #[error("{}", format!("{resource_type} '{name}' not found").red().bold())]
    NotFound {
        resource_type: String,
        name: String,
        verbose: Option<String>,
        suggestions: Vec<String>,
    },

    /// Project root not found (settings.toml not found in current or parent directories)
    #[error("{}", "Could not find project root".red().bold())]
    ProjectNotFound {
        searched_from: PathBuf,
        verbose: String,
        suggestions: Vec<String>,
    },

    /// Invalid user input with descriptive message
    #[error("{}", message.red().bold())]
    InvalidInput {
        message: String,
        verbose: Option<String>,
        suggestions: Vec<String>,
    },

    /// A configuration file error
    #[error("{}", message.red().bold())]
    ConfigurationError {
        message: String,
        verbose: Option<String>,
        suggestions: Vec<String>,
    },

    /// File operation error with context
    #[error("{}", message.red().bold())]
    FileOperationError {
        message: String,
        verbose: Option<String>,
        suggestions: Vec<String>,
        #[source]
        source: Option<io::Error>,
    },
}

/// Inner data structure for CliError to reduce duplication
#[derive(Debug, Clone)]
#[allow(dead_code)]
struct ErrorData {
    message: String,
    verbose: Option<String>,
    suggestions: Vec<String>,
}

impl CliError {
    /// Extract the common error data fields
    fn data(&self) -> ErrorData {
        match self {
            Self::NotFound {
                resource_type,
                name,
                verbose,
                suggestions,
            } => ErrorData {
                message: format!("{resource_type} '{name}' not found"),
                verbose: verbose.clone(),
                suggestions: suggestions.clone(),
            },
            Self::ProjectNotFound {
                verbose,
                suggestions,
                ..
            } => ErrorData {
                message: "Could not find project root".to_owned(),
                verbose: Some(verbose.clone()),
                suggestions: suggestions.clone(),
            },
            Self::InvalidInput {
                message,
                verbose,
                suggestions,
            } => ErrorData {
                message: message.clone(),
                verbose: verbose.clone(),
                suggestions: suggestions.clone(),
            },
            Self::ConfigurationError {
                message,
                verbose,
                suggestions,
            } => ErrorData {
                message: message.clone(),
                verbose: verbose.clone(),
                suggestions: suggestions.clone(),
            },
            Self::FileOperationError {
                message,
                verbose,
                suggestions,
                ..
            } => ErrorData {
                message: message.clone(),
                verbose: verbose.clone(),
                suggestions: suggestions.clone(),
            },
        }
    }

    /// Check if suggestions should be shown
    pub fn has_suggestions(&self) -> bool {
        !self.data().suggestions.is_empty()
    }

    /// Get suggestions for this error
    pub fn get_suggestions(&self) -> Vec<String> {
        self.data().suggestions
    }

    /// Get verbose details for this error (only shown in verbose mode)
    pub fn get_verbose(&self) -> Option<String> {
        self.data().verbose
    }
}

/// Extension trait for adding context to io::Error
#[allow(dead_code)]
pub trait IoErrorExt<T> {
    /// Convert a Result to CliError::FileOperationError with context
    fn with_file_context(
        self,
        path: impl Into<PathBuf>,
        purpose: impl Into<String>,
    ) -> Result<T, CliError>;

    /// Convert a Result to CliError::FileOperationError
    fn with_operation_context(
        self,
        operation: impl Into<String>,
        path: impl Into<PathBuf>,
    ) -> Result<T, CliError>;
}

impl<T> IoErrorExt<T> for Result<T, io::Error> {
    fn with_file_context(
        self,
        path: impl Into<PathBuf>,
        purpose: impl Into<String>,
    ) -> Result<T, CliError> {
        match self {
            Ok(val) => Ok(val),
            Err(e) => {
                let path = path.into();
                let message = if e.kind() == io::ErrorKind::NotFound {
                    format!("{} not found: {}", purpose.into(), path.display())
                } else {
                    format!("Failed to access {}: {}", purpose.into(), path.display())
                };

                Err(CliError::FileOperationError {
                    message,
                    verbose: Some(format!("Path: {}\nError: {}", path.display(), e)),
                    suggestions: if e.kind() == io::ErrorKind::NotFound {
                        vec![
                            "Check that the path is correct".to_owned(),
                            "Create the file/directory if needed".to_owned(),
                        ]
                    } else {
                        vec!["Check file permissions".to_owned()]
                    },
                    source: Some(e),
                })
            }
        }
    }

    fn with_operation_context(
        self,
        operation: impl Into<String>,
        path: impl Into<PathBuf>,
    ) -> Result<T, CliError> {
        match self {
            Ok(val) => Ok(val),
            Err(e) => Err(CliError::FileOperationError {
                message: format!(
                    "Failed to {} on {}",
                    operation.into(),
                    path.into().display()
                ),
                verbose: Some(format!("Error: {e}")),
                suggestions: vec!["Check file permissions".to_owned()],
                source: Some(e),
            }),
        }
    }
}

impl From<io::Error> for CliError {
    fn from(error: io::Error) -> Self {
        CliError::FileOperationError {
            message: "I/O error occurred".to_owned(),
            verbose: Some(format!("Error details: {error}")),
            suggestions: vec!["Check file/directory permissions".to_owned()],
            source: Some(error),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_file_not_found_error() {
        let err = CliError::NotFound {
            resource_type: "file".to_owned(),
            name: "config.toml".to_owned(),
            verbose: Some("Looking in /etc/config".to_owned()),
            suggestions: vec!["Create the file".to_owned()],
        };

        assert!(err.has_suggestions());
        assert_eq!(err.get_suggestions().len(), 1);
        assert_eq!(err.get_verbose(), Some("Looking in /etc/config".to_owned()));
    }

    #[test]
    fn test_not_found_suggestions() {
        let err = CliError::NotFound {
            resource_type: "problem".to_owned(),
            name: "my-problm".to_owned(),
            verbose: Some("Searched in problems/".to_owned()),
            suggestions: vec!["Did you mean: my-problem?".to_owned()],
        };

        assert!(err.has_suggestions());
        assert_eq!(err.get_suggestions().len(), 1);
    }

    #[test]
    fn test_error_without_suggestions() {
        let err = CliError::InvalidInput {
            message: "Invalid input".to_owned(),
            verbose: None,
            suggestions: vec![],
        };

        assert!(!err.has_suggestions());
        assert!(err.get_suggestions().is_empty());
    }
}
