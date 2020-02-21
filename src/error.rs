//! When serializing or deserializing from Postgres rows goes wrong.
use serde::{de, ser};
use std::{error, fmt};

/// Alias for a `Result` with the error type `serde_postgres::Error`.
pub type Result<T> = ::std::result::Result<T, Error>;

/// This type represents all possible error that can occur when deserializing
/// postgres rows.
#[derive(Clone, Debug, PartialEq)]
pub enum Error {
    /// A custom defined error occurred. Typically coming from `serde`.
    Message(String),
    /// Row contained a field unknown to the data structure.
    UnknownField,
    /// Row's column type was different from the Rust data structure.
    InvalidType(String),
    /// Rust data structure contained a type unsupported by `serde_postgres`.
    UnsupportedType,
}

impl de::Error for Error {
    fn custom<T: fmt::Display>(msg: T) -> Self {
        Error::Message(msg.to_string())
    }
}

impl ser::Error for Error {
    fn custom<T: fmt::Display>(msg: T) -> Self {
        Error::Message(msg.to_string())
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(error::Error::description(self))
    }
}

impl error::Error for Error {
    fn description(&self) -> &str {
        match self {
            Error::Message(ref msg) => msg,
            Error::UnknownField => "Unknown field",
            Error::InvalidType(_) => "Invalid type",
            Error::UnsupportedType => "Type unsupported",
        }
    }
}
