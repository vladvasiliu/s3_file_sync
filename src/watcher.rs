extern crate crossbeam_channel;
extern crate notify;

use std::path::{Path, PathBuf};
use std::time::Duration;

use crossbeam_channel::{unbounded, Sender, Receiver};
use log::{debug, info, warn, error};
use notify::{RecommendedWatcher, RecursiveMode, Watcher, Result as NotifyResult, Event};
use notify::event::{EventKind, CreateKind};

use crate::database::{File, Database};
use crate::error::{Result, Error};


pub struct FileWatcher {
    db: Database,
    upload_tx: Sender<File>,
    watcher_rx: Receiver<NotifyResult<Event>>,
    _watcher: RecommendedWatcher,
}


impl  FileWatcher {
    pub fn new<P: AsRef<Path>>(paths: &[P],
                               delay: u64,
                               db: Database,
                               upload_tx: Sender<File>
    ) -> Result<FileWatcher> {
        let (watcher_tx, watcher_rx) = unbounded();
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
            db,
            upload_tx,
            watcher_rx,
            _watcher,
        })
    }

    pub fn run(&self) {
        while let Some(msg) = self.watcher_rx.recv().ok() {
            match msg {
                Ok(event) if event.kind == EventKind::Create(CreateKind::Any) => {
                    self.handle_event(event);
                },
                Ok(_) => (),
                Err(err) => warn!("Error watching files: {:?}", err),
            }
        }
        error!("Watcher channel broken. Stopping watcher.")
    }

    fn handle_event(&self, event: Event) {
        // File creation should only return one path, hence we can safely use the first element.
        let file = File::new(&event.paths[0]);
        debug!("Detected file: {:?}", event.paths[0].display());
        match self.db.add_file(&file) {
            Ok(_) => {},
            Err(err) => warn!("Failed to add file to database: {:?}", err),
        }
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
