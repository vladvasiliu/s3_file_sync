use std::collections::HashSet;
use std::path::Path;
use std::sync::mpsc::{channel, Receiver, Sender};

use log::{debug, warn, error};

pub mod error;
pub mod file;

use crate::controller::file::File;
use crate::controller::error::{Error, Result};
use crate::uploader::Uploader;
use crate::watcher::FileWatcher;


struct Controller {
    uploaders: Vec<Uploader>,
    watchers: Vec<FileWatcher>,
    watcher_rx: Receiver<File>,
}

impl Controller {
    pub fn new<P: AsRef<Path>>(paths: &[P]) -> Result<Self> {
        let (watcher_tx, watcher_rx) = channel();

        let watchers = create_watchers(paths, watcher_tx)?;

        Ok(Self {
            uploaders: Vec::new(),
            watchers,
            watcher_rx,
        })
    }

}


/// Creates a vector of Watchers for all the directory trees specified in the list.
/// If any one of the trees is a subtree of another, it is ignored. Ex:
/// Given `/a/b/c` and `/a/b` only `/a/b` will be watched.
fn create_watchers<P: AsRef<Path>>(paths: &[P], watcher_tx: Sender<File>) -> Result<Vec<FileWatcher>> {
    let actual_paths = get_paths(paths);

    let mut watchers = Vec::new();

    for path in actual_paths {
        match FileWatcher::new(&path, 2, watcher_tx.clone()) {
            Ok(watcher) => watchers.push(watcher),
            Err(err) => warn!("Failed to create watcher for path [{}]: {}", path.display(), err),
        }
    }

    Ok(watchers)
}

fn get_paths<P: AsRef<Path>>(paths: &[P]) -> HashSet<&Path> {
    let mut result = HashSet::new();

    'outer: for cur_path in paths.iter().map(|x| x.as_ref()) {
        for check_path in paths.iter().map(|x| x.as_ref()) {
            if cur_path != check_path && cur_path.starts_with(check_path) {
                continue 'outer;
            }
        }
        result.insert(cur_path);
    }

    result
}


#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    #[test]
    fn test_get_paths() {
        use super::get_paths;
        use std::path::Path;

        let paths = vec![
            Path::new("/home/toto/tata"),
            Path::new("/home/toto/"),
            Path::new("/home/toto/titi"),
            Path::new("/home/toto/titi"),
            Path::new("/home/tutu/titi"),
        ];

        let mut expected_result = HashSet::new();
        expected_result.insert(Path::new("/home/toto/"));
        expected_result.insert(Path::new("/home/tutu/titi"));

        let actual_result = get_paths(paths.as_ref());
        assert_eq!(actual_result, expected_result);
    }
}
