use log::{info, error};
use std::thread;
use std::sync::mpsc::channel;
use crate::watcher::FileWatcher;
use crate::uploader::Uploader;
use std::process::exit;

mod controller;
mod uploader;
mod watcher;

fn main() {
    setup_logger().unwrap();
    info!("Starting S3 File Sync...");

    let (upload_tx, upload_rx) = channel();

    let file_watcher = FileWatcher::new(&"/home/vlad/tmp", 2, upload_tx).unwrap_or_else(|err| {
        error!("Gor an error: {}", err);
        exit(1)
    });

    let watcher_thread = thread::spawn(move || {
        file_watcher.run();
    });

    let uploader = Uploader::new(
        "test-s3-file-sync",
        "eu-west-3",
    );

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
        .level(log::LevelFilter::Debug)
        .chain(std::io::stdout())
//        .chain(fern::log_file("output.log")?)
        .apply()?;
    Ok(())
}
