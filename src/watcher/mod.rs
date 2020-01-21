use std::path::{Path, PathBuf};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::time::Duration;

use log::{debug, info, warn, error};
use notify::{RecommendedWatcher, RecursiveMode, Watcher, DebouncedEvent};

pub mod error;

use crate::watcher::error::{Result, Error};


/// Watches a directory and sends events for created files
///
/// Only one directory tree is watched.
/// This allows to upload files from each tree to its own directory.
pub struct FileWatcher {
    base_path: PathBuf,
    controller_tx: Sender<PathBuf>,
    watcher_rx: Receiver<DebouncedEvent>,
    _watcher: RecommendedWatcher,
}


impl  FileWatcher {
    pub fn new<P: AsRef<Path>>(path: &P, delay: u64, controller_tx: Sender<PathBuf>) -> Result<FileWatcher> {
        if !path.as_ref().is_dir() {
            return Err(Error::not_dir(path));
        }
        let base_path = path.as_ref().canonicalize()?;

        let (watcher_tx, watcher_rx) = channel();
        let mut _watcher: RecommendedWatcher = Watcher::new(watcher_tx,
                                                            Duration::from_secs(delay))?;

        match _watcher.watch(&base_path, RecursiveMode::Recursive) {
            Ok(()) => info!("Watching path: {}", base_path.display()),
            Err(err) => warn!("Failed to watch path: {}", err),
        }

        Ok(FileWatcher {
            base_path,
            controller_tx,
            watcher_rx,
            _watcher,
        })
    }

    pub fn run(&self) {
        for msg in self.watcher_rx.iter() {
            match msg {
                DebouncedEvent::Create(path) => {
                    self.handle_event(path);
                },
                DebouncedEvent::Error(err, path) => warn!("Error watching files:[{:?}] {:?}", path, err),
                _ => {}
            }
        }
        error!("Watcher channel broken. Stopping watcher.")
    }

    fn handle_event(&self, path: PathBuf) {
        if !path.is_file() {
            debug!("Ignoring non-file or unreadable path: {}", path.display());
            return;
        }

        match path.strip_prefix(&self.base_path) {
            Ok(stripped_path) => debug!("Detected file: {}", stripped_path.display()),
            Err(err) => warn!("Failed to remove base path: {}", err),
        }

        self.controller_tx.send(path).unwrap_or_else(|err| {
            warn!("Failed to notify file detection: {}", err);
        });
    }
}
