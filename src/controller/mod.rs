use std::path::Path;
use std::thread::Builder;

use crossbeam_channel::unbounded;
use log::{warn, error};

pub mod error;
pub mod file;

use crate::controller::error::{Result};
use crate::uploader::Uploader;
use crate::watcher::FileWatcher;


pub struct Controller {
}

impl Controller {
    pub fn run<P: AsRef<Path>>(paths: &[P], duration: u64) -> Result<()> {
        let (watcher_tx, watcher_rx) = unbounded();
        let (uploader_tx, uploader_rx) = unbounded();
        let num_uploaders = 2;

        // There's no need to hold handles to the threads,
        // they are expected to stop when their respectives channels will be closed
        for num in 1..=num_uploaders {
            let uploader = Uploader::new("test-s3-file-sync", "eu-west-3", uploader_rx.clone());
            Builder::new().name(format!("uploader {}", num)).spawn(move || uploader.run())?;
        }

        for watcher in FileWatcher::create_watchers(paths, watcher_tx, duration)? {
            Builder::new().name(watcher.base_path.display().to_string()).spawn(move || watcher.run())?;
        }

        loop {
            match watcher_rx.recv() {
                Err(err) => {
                    error!("Failed to receive file from watcher: {}", err);
                    break;
                },
                Ok(file) => {
                    uploader_tx.send(file)
                        .unwrap_or_else(|err| warn!("Failed to send file to uploader: {}", err));
                }
            }
        }
        Ok(())
    }
}
