use std::path::Path;
use std::sync::mpsc;
use crossbeam_channel::unbounded;
use crossbeam_utils::thread;

use log::{warn, info};


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
    pub fn run<P: AsRef<Path>>(paths: &[P], duration: u64) -> Result<()> {
        let (watcher_tx, watcher_rx) = unbounded();
        let num_uploaders = 2;
        let uploader = Uploader::new("test-s3-file-sync", "eu-west-3");

        thread::scope(|s| {
            for num in 1..=num_uploaders {
                s.builder().name(format!("uploader {}", num)).spawn(|_| uploader.run()).unwrap();
            }

            for watcher in FileWatcher::create_watchers(paths, watcher_tx, duration).unwrap() {
                s.builder().name(watcher.base_path.display().to_string()).spawn(move |_| watcher.run()).unwrap();
            }

            loop {
                match watcher_rx.recv() {
                    Err(err) => warn!("Failed to receive file from watcher: {}", err),
                    Ok(file) => {
                        info!("Received file from watcher: {}", file);
                    }
                }
            }
        });

        Ok(())
    }
}
