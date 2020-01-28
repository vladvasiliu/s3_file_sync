use fern::colors::{Color, ColoredLevelConfig};
use log::{info, error};
use std::thread;
use crate::controller::Controller;

mod controller;
mod uploader;
mod watcher;

fn main() {
    setup_logger().unwrap();
    info!("Starting S3 File Sync...");

    match controller::Controller::run(&["./toto"], 2) {
        Ok(_) => info!("Running!"),
        Err(err) => error!("Failed to start controller: {}", err),
    }
}

fn setup_logger() -> Result<(), fern::InitError> {
    let mut colors = ColoredLevelConfig::new()
        .debug(Color::Cyan)
        .info(Color::Green)
        .warn(Color::Yellow)
        .error(Color::Red);

    fern::Dispatch::new()
        .format(move |out, message, record| {
            out.finish(format_args!(
                "[ {} ][ {:5} ][ {:25} ][ {} ] {}",
                chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
                colors.color(record.level()),
                record.target(),
                thread::current().name().unwrap(),
                message
            ))
        })
        .level(log::LevelFilter::Debug)
        .chain(std::io::stdout())
//        .chain(fern::log_file("output.log")?)
        .apply()?;
    Ok(())
}
