use std::thread;

use log::{error, info, trace};
use std::time::Duration;

mod database;
mod watcher;


fn main() {
    setup_logger().unwrap();
    info!("Starting S3 File Sync...");
    let watcher_thread = thread::spawn(|| {
        let db = database::Database::open("db.sqlite3").unwrap();
        watcher::FileWatcher::run(db, &["."], 2);
    });

    let uploader_thread = thread::spawn( || {
        let db = database::Database::open("db.sqlite3").unwrap();
        loop {
            match db.files_to_upload() {
                Ok(files) => trace!("Files to upload: {}", files.len()),
                Err(err) => error!("Couldn't get files to upload: {:?}", err)
            }
            thread::sleep(Duration::from_secs(2))
        }
    });

    watcher_thread.join();
    uploader_thread.join();
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
        .level(log::LevelFilter::Trace)
        .chain(std::io::stdout())
//        .chain(fern::log_file("output.log")?)
        .apply()?;
    Ok(())
}
