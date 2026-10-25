use std::io;
use thiserror::Error;

/// Result alias for rust-pdf-box operations.
pub type PdfResult<T> = Result<T, PdfError>;

/// Unified error type for all PDF operations.
#[derive(Debug, Error)]
pub enum PdfError {
    #[error("I/O error: {0}")]
    Io(#[from] io::Error),

    #[error("Parse error at offset {offset}: {message}")]
    Parse { offset: usize, message: String },

    #[error("Invalid PDF format: {0}")]
    InvalidFormat(String),

    #[error("Missing required dictionary entry: {0}")]
    MissingEntry(String),

    #[error("Unexpected object type: expected {expected}, found {found}")]
    TypeMismatch {
        expected: &'static str,
        found: &'static str,
    },

    #[error("Invalid object reference: {0:?}")]
    InvalidReference(crate::cos::ObjectId),

    #[error("Stream decompression or compression error: {0}")]
    StreamCompression(String),

    #[error("Font error: {0}")]
    Font(String),

    #[error("Image processing error: {0}")]
    Image(String),

    #[error("Cryptography/Security error: {0}")]
    Crypto(String),

    #[error("Digital signature error: {0}")]
    Signature(String),

    #[error("Page operation error: {0}")]
    PageOperation(String),

    #[error("Unsupported feature: {0}")]
    Unsupported(String),
}
