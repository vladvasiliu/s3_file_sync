use std::path::{Path, PathBuf};
use chrono::{DateTime, Utc};

#[derive(Debug)]
pub struct File {
    pub path: PathBuf,
    pub first_seen_date: DateTime<Utc>,
    pub uploaded_date: Option<DateTime<Utc>>,
    pub deleted_date: Option<DateTime<Utc>>,
}

impl File {
    pub fn new(path: &Path) -> File {
        File {
            path: path.canonicalize().unwrap(),
            first_seen_date: Utc::now(),
            uploaded_date: None,
            deleted_date: None,
        }
    }
}
