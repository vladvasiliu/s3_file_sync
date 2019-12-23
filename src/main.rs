extern crate chrono;
use notify::Event;
use notify::event::{EventKind, CreateKind};

use crate::database::File;

mod database;
mod watcher;


fn main() {
    let db = database::Database::open("db.sqlite3").unwrap();
    let paths =  ["."];
    let file_watcher = watcher::FileWatcher::new(&paths, 2).unwrap();

    loop {
        match file_watcher.rx.recv() {
            // Ok(event) =>  println!("changed: {:?}", event),
            Ok(event) => {
                let event = event.unwrap();
                handle_event(event, &db)
            }
            Err(err) => {
                println!("watch error: {:?}", err);
            },
        };
    }
}

fn handle_event(event: Event, db: &database::Database) {
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
//        _ => println!("Something else: {:?}", event),
        _ => {},
    }
}
