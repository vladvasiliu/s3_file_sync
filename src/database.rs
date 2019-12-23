use std::path::{Path, PathBuf};
use rusqlite::{params, Connection, Result, ToSql};
use rusqlite::types::ToSqlOutput;
use chrono::{DateTime, Utc};


#[derive(Debug)]
pub struct File {
    pub path: PathBuf,
    pub first_seen_date: DateTime<Utc>,
    pub uploaded_date: Option<DateTime<Utc>>,
    pub deleted_date: Option<DateTime<Utc>>,
}

impl File {
    pub fn new(path: &Path) -> File {
        File {
            path: path.canonicalize().unwrap(),
            first_seen_date: Utc::now(),
            uploaded_date: None,
            deleted_date: None,
        }
    }
}

pub struct Database {
    connection: Connection,
}

impl Database {
    pub fn init_db(&self) -> Result<()> {
        self.connection.execute(
            "CREATE TABLE IF NOT EXISTS File (
                    path            TEXT PRIMARY KEY,
                    first_seen_date TEXT NOT NULL,
                    uploaded_date   TEXT,
                    deleted_date    TEXT
        )",
            params![],
        )?;
        self.connection.execute(
            "CREATE INDEX IF NOT EXISTS file_uploaded ON File (\
                    uploaded_date
                  )",
            params![],
        )?;
        self.connection.execute(
            "CREATE INDEX IF NOT EXISTS file_not_deleted ON File (\
                    deleted_date
                  )
                  WHERE deleted_date IS NULL and uploaded_date IS NOT NULL",
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

    pub fn add_file(&self, file: &File) -> Result<usize> {
        self.connection.execute("INSERT INTO File (path, first_seen_date) values (?1, ?2)",
                                params![&file.path.to_str().unwrap(), &file.first_seen_date])
    }
}
