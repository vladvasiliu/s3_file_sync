use std::{error::Error as StdError, result::Result as StdResult, fmt};

use crate::watcher::error::Error as WatcherError;

pub type Result<T> = StdResult<T, Error>;

#[derive(Debug)]
pub enum Error {
    FileWatcher(WatcherError),
}

impl From<WatcherError> for Error {
    fn from(err: WatcherError) -> Self {
        Self::FileWatcher(err)
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::FileWatcher(watcher_err) => write!(f, "Failed to start file watcher: {}", watcher_err),
        }
    }
}
