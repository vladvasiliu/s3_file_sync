use std::error::Error as StdError;
use std::path::{Path, PathBuf};
use std::result::Result as StdResult;
use std::{fmt, io};

use notify;

#[derive(Debug)]
pub enum ErrorKind {
    /// Generic / opaque errors
    //    Generic(String),

    /// The path is not a directory or is not accessible
    NotDir,

    /// The path cannot be canonicalized
    NotCanon(io::Error),

    /// An error raised by notify-rs
    WatcherErr(notify::Error),

    //    /// An error raised from std::path
    Io(io::Error),
}

#[derive(Debug)]
pub struct Error {
    pub kind: ErrorKind,
    path: Option<PathBuf>,
}

pub type Result<T> = StdResult<T, Error>;

impl Error {
    pub fn not_dir<P: AsRef<Path>>(path: P) -> Self {
        Self {
            kind: ErrorKind::NotDir,
            path: Some(PathBuf::from(path.as_ref())),
        }
    }
    pub fn not_canon<P: AsRef<Path>>(path: P, err: io::Error) -> Self {
        Self {
            kind: ErrorKind::NotCanon(err),
            path: Some(PathBuf::from(path.as_ref())),
        }
    }
}

impl StdError for Error {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self.kind {
            ErrorKind::WatcherErr(ref err) => Some(err),
            _ => None,
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let msg: String = match self.kind {
            ErrorKind::NotDir => "The path is not accessible or not a directory".into(),
            ErrorKind::WatcherErr(ref err) => err.description().into(),
            ErrorKind::NotCanon(ref err) => {
                format!("Cannot canonicalize, I/O Error for path: {:?}", err)
            }
            ErrorKind::Io(ref err) => format!("IO error: {}", err),
        };

        if let Some(path) = &self.path {
            write!(f, "{}: {}", msg, path.display())
        } else {
            write!(f, "{}", msg)
        }
    }
}

impl From<notify::Error> for Error {
    fn from(err: notify::Error) -> Self {
        Error {
            kind: ErrorKind::WatcherErr(err),
            path: None,
        }
    }
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Self {
        Self {
            kind: ErrorKind::Io(err),
            path: None,
        }
    }
}
