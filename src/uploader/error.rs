use std::{
    error::Error as StdError,
    io::Error as IOError,
    result::Result as StdResult,
    fmt,
};

use rusoto_core::RusotoError;
use rusoto_s3::{ CreateMultipartUploadError, CompleteMultipartUploadError, UploadPartError };


pub type Result<T> = StdResult<T, Error>;

#[derive(Debug)]
pub enum Error {
    CreateMultipartUpload(RusotoError<CreateMultipartUploadError>),
    UploadPart { part_number: i64, error: RusotoError<UploadPartError> },
    CompleteMultipartUpload(RusotoError<CompleteMultipartUploadError>),
    Generic(String),
    Read(IOError),
}


impl StdError for Error {}


impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::CreateMultipartUpload(err) => write!(f, "Failed to create multipart upload: {}", err),
            Self::UploadPart{part_number, error} => write!(f, "Failed to upload part {}: {}", part_number, error),
            Self::CompleteMultipartUpload(err) => write!(f, "Failed to complete multipart upload: {}", err),
            Self::Read(io_error) => write!(f, "Failed to read file: {}", io_error),
            Self::Generic(msg) => write!(f, "Failed to upload file: {}", msg)
        }
    }
}

impl From<IOError> for Error {
    fn from(err: IOError) -> Self {
        Self::Read(err)
    }
}

impl From<RusotoError<CompleteMultipartUploadError>> for Error {
    fn from(err: RusotoError<CompleteMultipartUploadError>) -> Self {
        Self::CompleteMultipartUpload(err)
    }
}

impl From<&str> for Error {
    fn from(msg: &str) -> Self {
        Self::Generic(msg.into())
    }
}
