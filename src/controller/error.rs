use std::{fmt, io, result::Result as StdResult};

use crate::controller::database::error::Error as DBError;
use crate::watcher::error::Error as WatcherError;

pub type Result<T> = StdResult<T, Error>;

#[derive(Debug)]
pub enum Error {
    FileWatcher(WatcherError),
    IO(io::Error),
    Database(DBError),
}

impl From<WatcherError> for Error {
    fn from(err: WatcherError) -> Self {
        Self::FileWatcher(err)
    }
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Self {
        Self::IO(err)
    }
}

impl From<DBError> for Error {
    fn from(err: DBError) -> Self {
        Self::Database(err)
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::FileWatcher(watcher_err) => {
                write!(f, "Failed to start file watcher: {}", watcher_err)
            }
            Self::IO(err) => write!(f, "I/O Error: {}", err),
            Self::Database(err) => write!(f, "Database Error: {}", err),
        }
    }
}
