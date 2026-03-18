//! Error types for the Scope library.
//!
//! Use `ScopeError` for typed errors that need to be matched on.
//! Use `anyhow::Result` for application-level error propagation.
use thiserror::Error;

/// Typed errors for Scope operations that callers may need to handle specifically.
#[derive(Error, Debug)]
#[allow(dead_code)]
pub enum ScopeError {
    /// The `.scope/` directory or index database was not found.
    #[error("No index found. Run 'sc init' to initialise, then 'sc index' to build the index.")]
    IndexNotFound,

    /// A symbol name was not found in the index.
    #[error("Symbol '{0}' not found in index. Run 'sc index' if this is a new file.")]
    SymbolNotFound(String),

    /// tree-sitter failed to parse a source file.
    #[error("Failed to parse {file_path}: {reason}")]
    ParseError {
        /// Path to the file that failed to parse.
        file_path: String,
        /// Description of what went wrong.
        reason: String,
    },

    /// SQLite or LanceDB storage operation failed.
    #[error("Storage error: {0}")]
    StorageError(String),

    /// Configuration file is missing or malformed.
    #[error("Configuration error: {0}")]
    ConfigError(String),

    /// A language is not yet supported by Scope.
    #[error("Unsupported language: {0}")]
    UnsupportedLanguage(String),
}
