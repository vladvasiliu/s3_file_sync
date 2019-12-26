extern crate crossbeam_channel;
extern crate notify;

use std::path::Path;
use std::time::Duration;

use crossbeam_channel::{unbounded, Receiver};
use notify::{RecommendedWatcher, RecursiveMode, Result, Watcher, Event};
use notify::event::{EventKind, CreateKind};

use crate::database;


pub struct FileWatcher {
    pub rx: Receiver<Result<Event>>,
    pub watcher : RecommendedWatcher,
}


impl  FileWatcher {
    pub fn run<P: AsRef<Path>>(db: database::Database, paths: &[P], delay: u64 ) {
        let (tx, rx) = unbounded();
        // Automatically select the best implementation
        let mut watcher: RecommendedWatcher = Watcher::new(tx, Duration::from_secs(delay))
                                                            .unwrap();

        // Add a path to be watched
        for path in paths {
            match watcher.watch(path, RecursiveMode::Recursive) {
                Ok(()) => {},
                Err(err) => println!("Cannot watch path : {:?}", err),
            }
        }

        loop {
            match rx.recv() {
                // Ok(event) =>  println!("changed: {:?}", event),
                Ok(event) => {
                    let event = event.unwrap();
                    FileWatcher::handle_event(event, &db)
                }
                Err(err) => {
                    println!("watch error: {:?}", err);
                },
            };
        }
    }

    pub fn handle_event(event: Event, db: &database::Database) {
        match event.kind {
            EventKind::Create(CreateKind::Any) => {
                // File creation should only return one path, hence we can safely use the first element.
                let file = File::new(&event.paths[0]);
                println!("files: {:?}", file);
                match db.add_file(&file) {
                    Ok(_) => {},
                    Err(err) => println!("Failed to add file to database: {:?}", err),
                }
            },
            _ => {},
        }
    }
}
