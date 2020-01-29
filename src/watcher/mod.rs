use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::mpsc::{channel, Receiver};
use std::time::Duration;

use crossbeam_channel::Sender;
use log::{debug, error, info, warn};
use notify::{DebouncedEvent, RecommendedWatcher, RecursiveMode, Watcher};

pub mod error;

use crate::controller::file::File;
use crate::watcher::error::{Error, Result};

/// Watches a directory and sends events for created files
///
/// Only one directory tree is watched.
/// This allows to upload files from each tree to its own directory.
pub struct FileWatcher {
    pub base_path: PathBuf,
    controller_tx: Sender<File>,
    watcher_rx: Receiver<DebouncedEvent>,
    _watcher: RecommendedWatcher,
}

impl FileWatcher {
    pub fn create_watchers<P: AsRef<Path>>(
        paths: &[P],
        controller_tx: Sender<File>,
        watcher_duration: u64,
    ) -> Result<Vec<FileWatcher>> {
        let mut canonical_paths = Vec::new();

        for path in paths {
            canonical_paths.push(
                path.as_ref()
                    .canonicalize()
                    .or_else(|err| Err(Error::not_canon(path, err)))?,
            );
        }

        let filtered_paths = get_paths(&canonical_paths);

        let mut watchers = Vec::new();

        for path in filtered_paths {
            watchers.push(Self::new(&path, watcher_duration, controller_tx.clone())?)
        }

        Ok(watchers)
    }

    pub fn new<P: AsRef<Path>>(
        path: &P,
        delay: u64,
        controller_tx: Sender<File>,
    ) -> Result<FileWatcher> {
        if !path.as_ref().is_dir() {
            return Err(Error::not_dir(path));
        }
        let base_path = path.as_ref().canonicalize()?;

        let (watcher_tx, watcher_rx) = channel();

        let mut _watcher: RecommendedWatcher =
            Watcher::new(watcher_tx, Duration::from_secs(delay))?;

        _watcher.watch(&base_path, RecursiveMode::Recursive)?;

        Ok(FileWatcher {
            base_path,
            controller_tx,
            watcher_rx,
            _watcher,
        })
    }

    pub fn run(&self) {
        info!("Started watcher");
        for msg in self.watcher_rx.iter() {
            match msg {
                DebouncedEvent::Create(path) => {
                    self.handle_event(path);
                }
                DebouncedEvent::Error(err, path) => {
                    warn!("Error watching files:[{:?}] {:?}", path, err)
                }
                _ => {}
            }
        }
        error!(
            "Watcher channel broken. Stopping watcher for {}",
            self.base_path.display()
        )
    }

    fn handle_event(&self, path: PathBuf) {
        if !path.is_file() {
            debug!("Ignoring non-file or unreadable path: {}", path.display());
            return;
        }

        match path.strip_prefix(&self.base_path) {
            Ok(stripped_path) => {
                debug!("Detected file: {}", stripped_path.display());
                let file = File {
                    base_path: self.base_path.to_owned(),
                    key: stripped_path.into(),
                };
                self.controller_tx.send(file).unwrap_or_else(|err| {
                    warn!("Failed to notify file detection: {}", err);
                });
            }
            Err(err) => warn!("Failed to remove base path: {}", err),
        }
    }
}

/// Gets the tree roots of the provided paths
///
/// If any one of the trees is a subtree of another, it is ignored. Ex:
/// Given `/a/b/c` and `/a/b` only `/a/b` will be watched.
/// For this to work, paths have to be canonical.
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
    use super::FileWatcher;
    use crossbeam_channel::unbounded;
    use std::path::Path;

    #[test]
    fn test_create_watchers_fails_with_missing_path() {
        let paths = [Path::new("/some/missing/path/")];
        let (watcher_tx, _) = unbounded();

        assert!(FileWatcher::create_watchers(&paths, watcher_tx, 2).is_err());
    }

    #[test]
    fn test_get_paths() {
        use super::get_paths;
        use std::collections::HashSet;

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
