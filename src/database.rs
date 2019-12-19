use std::path::{Path, PathBuf};
use rusqlite::{params, Connection, Result};


#[derive(Debug)]
pub struct File {
    filename: String,
}

pub struct Database {
    connection: Connection,
}

impl Database {
    pub fn init_db(&self) -> Result<()> {
        self.connection.execute(
            "CREATE TABLE IF NOT EXISTS File (
            path            TEXT PRIMARY KEY,
            first_seen_date TEXT DEFAULT CURRENT_TIMESTAMP NOT NULL,
            uploaded        TEXT,
            deleted         TEXT
        )",
            params![],
        )?;
        self.connection.execute(
            "CREATE INDEX IF NOT EXISTS file_uploaded ON File (\
                    uploaded
                  )",
            params![],
        )?;
        self.connection.execute(
            "CREATE INDEX IF NOT EXISTS file_not_deleted ON File (\
                    deleted
                  )
                  WHERE deleted is null and uploaded is not null",
            params![],
        )?;
        Ok(())
    }

    pub fn open<P: AsRef<Path>>(path: P) -> Result<Database> {
        let connection = Connection::open(path)?;
        let database = Database { connection };
        database.init_db()?;
        Ok(database)
    }

//    pub fn add_file<P: AsRef<Path>>(&self, paths: &[P]) -> Result<()> {
//        self.connection.execute("INSERT INTO File (path) values ?", paths)?;
//        Ok(())
//    }
}
