use std::path::Path;
use std::sync::mpsc::{channel, Receiver};


pub mod error;
pub mod file;

use crate::controller::file::File;
use crate::controller::error::{Result};
use crate::uploader::Uploader;
use crate::watcher::FileWatcher;
use std::thread::Builder;


pub struct Controller {
}

impl Controller {
    pub fn run<P: AsRef<Path>>(paths: &[P]) -> Result<()> {
        let (watcher_tx, watcher_rx) = channel();

        let uploaders = vec![
            Uploader::new("test-s3-file-sync", "eu-west-3"),
        ];

        let mut watchers = vec![];

        for watcher in FileWatcher::create_watchers(paths, watcher_tx, 2)? {
            let watcher_thread = Builder::new().name(watcher.base_path.display().to_string()).spawn(move || watcher.run())?;
            watchers.push(watcher_thread);
        }

        for thread in watchers {
            thread.join();
        }

        Ok(())
    }
}
