use std::path::Path;
use std::sync::mpsc::{channel, Receiver, Sender};


pub mod error;
pub mod file;

use crate::controller::file::File;
use crate::controller::error::{Result};
use crate::uploader::Uploader;
use crate::watcher::FileWatcher;


pub struct Controller {
    uploaders: Vec<Uploader>,
    watchers: Vec<FileWatcher>,
    watcher_rx: Receiver<File>,
}

impl Controller {
    pub fn new<P: AsRef<Path>>(paths: &[P]) -> Result<Self> {
        let (watcher_tx, watcher_rx) = channel();

        let watchers = FileWatcher::create_watchers(paths, watcher_tx, 2)?;

        Ok(Self {
            uploaders: Vec::new(),
            watchers,
            watcher_rx,
        })
    }
}
