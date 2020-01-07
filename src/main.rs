use log::{info};
use std::thread;
use crate::watcher::FileWatcher;
use crossbeam_channel::unbounded;
use crate::uploader::Uploader;

mod database;
mod error;
mod uploader;
mod watcher;

fn main() {
    setup_logger().unwrap();
    info!("Starting S3 File Sync...");

    let (upload_tx, upload_rx) = unbounded();
    let watcher_db = database::Database::open("db.sqlite3").unwrap();

    let file_watcher = FileWatcher::new(&["."], 2, watcher_db, upload_tx).unwrap();

    let watcher_thread = thread::spawn(move || {
        file_watcher.run();
    });

    let uploader_db = database::Database::open("db.sqlite3").unwrap();
    let uploader = Uploader::new(
        "test-s3-file-sync",
        "eu-west-3",
        upload_rx,
        uploader_db,
    ).unwrap();

    watcher_thread.join().expect("Failed to join watcher thread.");

}

fn setup_logger() -> Result<(), fern::InitError> {
    fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "[ {} ][ {:22} ][ {:5} ] {}",
                chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
                record.target(),
                record.level(),
                message
            ))
        })
        .level(log::LevelFilter::Info)
        .chain(std::io::stdout())
//        .chain(fern::log_file("output.log")?)
        .apply()?;
    Ok(())
}
