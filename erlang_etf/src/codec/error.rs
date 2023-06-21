use crate::term::Term;

/// Errors which can occur when decoding a term
#[derive(Debug, thiserror::Error)]
pub enum DecodeError {
    #[error("I/O error")]
    Io(#[from] std::io::Error),

    // #[error("BitReader error")]
    // BitReader(#[from] bitreader::BitReaderError),

    // #[error("atom cache error")]
    // AtomCacheError(#[from] AtomCacheError),
    #[error("the format version {version} is unsupported")]
    UnsupportedVersion { version: u8 },

    #[error("unknown tag {tag}")]
    UnknownTag { tag: u8 },

    #[error("{value} is not a {expected}")]
    UnexpectedType { value: Term, expected: String },

    #[error("{value} is out of range {range:?}")]
    OutOfRange {
        value: i32,
        range: std::ops::Range<i32>,
    },

    #[error("tried to convert non-finite float")]
    NonFiniteFloat,
}
