use std::thread::Builder;

use crossbeam_channel::{unbounded, Receiver, Select};
use log::{debug, error, info, warn};

mod database;
pub mod error;
pub mod file;

use crate::config::Config;
use crate::controller::database::{error::Error as DBError, Database};
use crate::controller::error::Result;
use crate::controller::file::File;
use crate::uploader::Uploader;
use crate::watcher::FileWatcher;

pub struct Controller {}

impl Controller {
    pub fn run(config: Config) -> Result<()> {
        let (watcher_tx, watcher_rx) = unbounded();
        let (ctl2upl_tx, ctl2upl_rx) = unbounded();
        let (upl2ctl_tx, upl2ctl_rx) = unbounded();

        let db = Database::open("db.sqlite3")?;

        // There's no need to hold handles to the threads,
        // they are expected to stop when their respective channels will be closed
        for num in 1..=config.num_uploaders {
            let uploader = Uploader::new(
                &config.bucket_name,
                "eu-west-3",
                ctl2upl_rx.clone(),
                upl2ctl_tx.clone(),
            );
            Builder::new()
                .name(format!("uploader {}", num))
                .spawn(move || uploader.run())?;
        }

        for watcher in
            FileWatcher::create_watchers(&config.watched_dirs, watcher_tx, config.watcher_delay)?
        {
            Builder::new()
                .name(watcher.base_path.display().to_string())
                .spawn(move || watcher.run())?;
        }

        let mut sel = Select::new();
        let rcv_from_watcher = sel.recv(&watcher_rx);
        let rcv_from_uploader = sel.recv(&upl2ctl_rx);

        loop {
            let oper = sel.select();

            match oper.index() {
                i if i == rcv_from_watcher => match oper.recv(&watcher_rx) {
                    Err(err) => {
                        error!("Failed to receive file from watcher: {}", err);
                        break;
                    }
                    Ok(file) => match db.add_file(&file) {
                        Ok(_) => ctl2upl_tx.send(file).unwrap_or_else(|err| {
                            warn!("Failed to send file to uploader: {}", err)
                        }),
                        Err(DBError::FileExists(err)) => {
                            warn!("Attempted to insert known file: {}", file);
                            debug!("Failed to add file `{}` to db: {}", file, err);
                        }
                        Err(err) => error!("Unexpected database error: {}", err),
                    },
                },
                i if i == rcv_from_uploader => match oper.recv(&upl2ctl_rx) {
                    Err(err) => {
                        warn!("Failed to receive from uploader: {}", err);
                        break;
                    }
                    Ok((file, result)) => match result {
                        Err(err) => warn!("Failed to upload {}: {}", file, err),
                        Ok(()) => info!("Uploaded {}", file),
                    },
                },
                _ => unreachable!(),
            }
        }
        Ok(())
    }
}
