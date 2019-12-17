use std::path::Path;
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
        connection.init_db?;
        Ok(Database{ connection })
    }
}
