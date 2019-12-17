extern crate notify;

use notify::{RecommendedWatcher, RecursiveMode, Result, Watcher, Event};
use notify::event::{EventKind, CreateKind};
use std::time::Duration;
use crossbeam_channel::Sender;


fn watcher(tx: Sender<Result<Event>>) -> Result<()> {
    // Automatically select the best implementation
    let mut watcher: RecommendedWatcher = Watcher::new(tx, Duration::from_secs(2))?;

    // Add a path to be watched
    watcher.watch(".", RecursiveMode::Recursive)?;

    loop {
        match rx.recv() {
            // Ok(event) =>  println!("changed: {:?}", event),
            Ok(event) => handle_event(event.unwrap()),
            Err(err) => println!("watch error: {:?}", err),
        };
    }
}

fn handle_event(event: Event) {
    match event.kind {
        EventKind::Create(CreateKind::Any) => println!("created: {:?}", event.paths),
        _ => println!("Something else: {:?}", event),
    }
}
