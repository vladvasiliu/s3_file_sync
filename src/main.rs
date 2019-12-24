use std::thread;

mod database;
mod file;
mod watcher;


fn main() {
    let watcher_thread = thread::spawn(|| {
        let db = database::Database::open("db.sqlite3").unwrap();
        watcher::FileWatcher::run(db, &["."], 2);
    });
    watcher_thread.join();
}

