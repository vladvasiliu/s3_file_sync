use crate::controller::Controller;
use fern::colors::{Color, ColoredLevelConfig};
use log::{error, info};
use std::thread;

mod config;
mod controller;
mod uploader;
mod watcher;

fn main() {
    let config = config::Config::from_args();
    setup_logger().unwrap();
    info!("Starting S3 File Sync...");
    info!("{}", config.pretty_string());

    match Controller::run(config) {
        Ok(_) => info!("Running!"),
        Err(err) => error!("Failed to start controller: {}", err),
    }
}

fn setup_logger() -> Result<(), fern::InitError> {
    let colors = ColoredLevelConfig::new()
        .debug(Color::Cyan)
        .info(Color::Blue)
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
