extern crate crossbeam_channel;
use crossbeam_channel::unbounded;

mod database;
mod watcher;


fn main() {
    let (tx, rx) = unbounded();

    let db = database::Database::open("db.sqlite3").unwrap();

}
