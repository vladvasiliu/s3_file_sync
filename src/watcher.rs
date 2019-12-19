extern crate crossbeam_channel;
extern crate notify;

use notify::{RecommendedWatcher, RecursiveMode, Result, Watcher, Event, ReadDirectoryChangesWatcher};
use std::path::Path;
use std::time::Duration;
use crossbeam_channel::{unbounded, Receiver};


pub struct FileWatcher {
    pub rx: Receiver<Result<Event>>,
    watcher: ReadDirectoryChangesWatcher,
}


impl  FileWatcher {
    pub fn new<P: AsRef<Path>>(paths: &[P], delay: u64) -> Result<FileWatcher> {
        let (tx, rx) = unbounded();
        // Automatically select the best implementation
        let mut watcher: RecommendedWatcher = Watcher::new(tx, Duration::from_secs(delay))?;

        // Add a path to be watched
        for path in paths {
            watcher.watch(path, RecursiveMode::Recursive)?;
        }
        Ok(FileWatcher { rx, watcher })
    }
}
