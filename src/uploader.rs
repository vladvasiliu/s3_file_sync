use crossbeam_channel::Receiver;

use crate::database::{Database, File};


pub struct Uploader {
    bucket_name: String,
    rx: Receiver<File>,
    db: Database,
}


impl Uploader {
    pub fn run() {
        let mut uploader = {}

    }
}