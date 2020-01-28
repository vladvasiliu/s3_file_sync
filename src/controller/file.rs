use std::fmt;
use std::path::PathBuf;

/// A file as handled by this program
///
/// This is an abstract file, as such the filesystem object it represents does not necessarily
/// exist. One such situation is a file that has been deleted.
///
/// The base_path is the root of the tree being watched
/// The key is the path from the base_path. It will be replicated on the bucket
#[derive(Debug)]
pub struct File {
    pub base_path: PathBuf,
    pub key: PathBuf,
}

impl File {
    pub fn full_path<'a>(self) -> PathBuf {
        self.base_path.join(&self.key)
    }
}


impl fmt::Display for File {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "[{}]{}", self.base_path.display(), self.key.display())
    }
}
