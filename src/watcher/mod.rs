use std::path::{Path, PathBuf};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::time::Duration;

use log::{debug, info, warn, error};
use notify::{RecommendedWatcher, RecursiveMode, Watcher, DebouncedEvent};

pub mod error;

use crate::database::{File};
use crate::watcher::error::{Result, Error};


pub struct FileWatcher {
    upload_tx: Sender<File>,
    watcher_rx: Receiver<DebouncedEvent>,
    _watcher: RecommendedWatcher,
}


impl  FileWatcher {
    pub fn new<P: AsRef<Path>>(paths: &[P],
                               delay: u64,
                               upload_tx: Sender<File>
    ) -> Result<FileWatcher> {
        let (watcher_tx, watcher_rx) = channel();
        let mut _watcher: RecommendedWatcher = Watcher::new(watcher_tx,
                                                           Duration::from_secs(delay))?;

        for path in paths {
            match is_path_valid(path) {
                Ok(p) => {
                    match _watcher.watch(&p, RecursiveMode::Recursive) {
                        Ok(()) => info!("Watching path: {}", p.display()),
                        Err(err) => warn!("Failed to watch path: {}", err),
                    }
                }
                Err(err) => warn!("Ignoring path. {}", err)
            }
        }

        Ok(FileWatcher {
            upload_tx,
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
        // File creation should only return one path, hence we can safely use the first element.
        let file = File::new(path.as_ref());
        debug!("Detected file: {}", path.display());
        self.upload_tx.send(file).unwrap_or_else(|err| {
            warn!("Failed to notify file detection: {}", err);
        });
    }
}


/// A helper method to check whether a watched path is valid
///
/// A path is valid if:
/// * It is a directory and it is readable
/// * Its canonical path is UTF-8
fn is_path_valid<P: AsRef<Path>>(path: &P) -> Result<PathBuf> {
    if !path.as_ref().is_dir() {
        return Err(Error::not_dir(path));
    }

    match path.as_ref().canonicalize() {
        Ok(canonical_path) => {
            match canonical_path.to_str() {
                Some(_) => Ok(canonical_path),
                None => Err(Error::not_utf8(path)),
            }
        },
        Err(err) => Err(Error::not_canon(path, err)),
    }
}
