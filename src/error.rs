use std::error::Error as StdError;
use std::fmt;
use std::result::Result as StdResult;
use std::path::PathBuf;

use notify;

#[derive(Debug)]
pub enum ErrorKind {
    /// Generic / opaque errors
    Generic(String),

    /// The path is not a directory or is not accessible
    NotDir,

    /// The path cannot be converted to UTF-8
    NotUTF8,

    /// The path cannot be canonicalized
    NotCanon,

    /// An error raised by notify-rs
    WatcherErr(notify::Error),
}

#[derive(Debug)]
pub struct Error {
    kind: ErrorKind,
//    path: PathBuf,
}

pub type Result<T> = StdResult<T, Error>;

impl Error {}

impl StdError for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Meh")
    }
}

impl From<notify::Error> for Error {
    fn from(err: notify::Error) -> Self {
        Error { kind: ErrorKind::WatcherErr(err) }
    }
}