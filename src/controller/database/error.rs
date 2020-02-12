use rusqlite::Error as SQLError;
use std::{error::Error as StdError, fmt, result::Result as StdResult};

pub type Result<T> = StdResult<T, Error>;

#[derive(Debug)]
pub enum Error {
    Unhandled(SQLError),
    FileExists(SQLError),
}

impl StdError for Error {}

impl Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Unhandled(err) => write!(f, "Got unhandled SQL Error: {}", err),
            Self::FileExists(err) => write!(f, "File exists in database: {}", err),
        }
    }
}

impl From<SQLError> for Error {
    fn from(err: SQLError) -> Self {
        Self::Unhandled(err)
    }
}
