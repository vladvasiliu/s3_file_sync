use std::{
    error::Error as StdError,
    result::Result as StdResult,
};

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
