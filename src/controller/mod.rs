use std::path::Path;
use std::thread::Builder;

use crossbeam_channel::unbounded;
use log::{error, warn};

pub mod error;
pub mod file;

use crate::controller::error::Result;
use crate::uploader::Uploader;
use crate::watcher::FileWatcher;

pub struct Controller {}

impl Controller {
    pub fn run<P: AsRef<Path>>(paths: &[P], duration: u64) -> Result<()> {
        let (watcher_tx, watcher_rx) = unbounded();
        let (ctl2upl_tx, ctl2upl_rx) = unbounded();
        let (upl2ctl_tx, upl2ctl_rx) = unbounded();
        let num_uploaders = 2;

        // There's no need to hold handles to the threads,
        // they are expected to stop when their respective channels will be closed
        for num in 1..=num_uploaders {
            let uploader = Uploader::new(
                "test-s3-file-sync",
                "eu-west-3",
                ctl2upl_rx.clone(),
                upl2ctl_tx.clone(),
            );
            Builder::new()
                .name(format!("uploader {}", num))
                .spawn(move || uploader.run())?;
        }

        for watcher in FileWatcher::create_watchers(paths, watcher_tx, duration)? {
            Builder::new()
                .name(watcher.base_path.display().to_string())
                .spawn(move || watcher.run())?;
        }

        loop {
            select! {
                            recv(watcher_rx) -> msg => match msg {
                                Err(err) => {
                                    error!("Failed to receive file from watcher: {}", err);
                                    break;
                                }
                                Ok(file) => {
                                    warn!("Got file: {}", file)
            //                        ctl2upl_tx
            //                            .send(file)
            //                            .unwrap_or_else(|err| warn!("Failed to send file to uploader: {}", err));
                                }
                            }

                        }
        }
        Ok(())
    }
}
