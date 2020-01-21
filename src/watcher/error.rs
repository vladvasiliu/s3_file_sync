use std::error::Error as StdError;
use std::{fmt, io};
use std::result::Result as StdResult;
use std::path::{Path, PathBuf};

use notify;

#[derive(Debug)]
pub enum ErrorKind {
    /// Generic / opaque errors
//    Generic(String),

    /// The path is not a directory or is not accessible
    NotDir,

    /// The path cannot be converted to UTF-8
    NotUTF8,

    /// The path cannot be canonicalized
    NotCanon(io::Error),

    /// An error raised by notify-rs
    WatcherErr(notify::Error),

//    /// An error raised from std::path
//    Io(io::Error),
}

#[derive(Debug)]
pub struct Error {
    pub kind: ErrorKind,
    path: Option<PathBuf>,
}

pub type Result<T> = StdResult<T, Error>;

impl Error {
    pub fn not_dir<P: AsRef<Path>>(path: P) -> Self {
        Self { kind: ErrorKind::NotDir, path: Some(PathBuf::from(path.as_ref())) }
    }
    pub fn not_utf8<P: AsRef<Path>>(path: P) -> Self {
        Self { kind: ErrorKind::NotUTF8, path: Some(PathBuf::from(path.as_ref())) }
    }
    pub fn not_canon<P: AsRef<Path>>(path: P, err: io::Error) -> Self {
//        Self { kind: ErrorKind::NotCanon, path: Some(PathBuf::from(path.as_ref())) }
        Self { kind: ErrorKind::NotCanon(err), path: Some(PathBuf::from(path.as_ref())) }
    }
//    pub fn io<P: AsRef<Path>>(path: P, err: io::Error) -> Self {
//        Self { kind: ErrorKind::Io(err), path: Some(PathBuf::from(path.as_ref())) }
//    }
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
//            ErrorKind::Generic(ref err) => err.clone(),
            ErrorKind::NotDir => "The path is not accessible or not a directory".into(),
            ErrorKind::NotUTF8 => "The path is not UTF-8".into(),
//            ErrorKind::NotCanon => "The path cannot be canonicalized".into(),
            ErrorKind::WatcherErr(ref err) => err.description().into(),
            ErrorKind::NotCanon(ref err) => format!("Cannot canonicalize, I/O Error for path: {:?}", err).into(),
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
