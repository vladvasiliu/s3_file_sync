//use std::thread;

mod database;
//mod watcher;


fn main() {
//    let watcher_thread = thread::spawn(|| {
//        let db = database::Database::open("db.sqlite3").unwrap();
//        watcher::FileWatcher::run(db, &["."], 2);
//    });
//    watcher_thread.join();

    let db = database::Database::open("db.sqlite3").unwrap();
    match db.files_to_upload() {
        Ok(_) => println!("ok"),
        Err(err) => println!("Error: {:?}", err)
    }
}
