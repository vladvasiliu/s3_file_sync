use std::path::{Path, PathBuf};
use rusqlite::{params, Connection, Result, ToSql};
use rusqlite::types::ToSqlOutput;


#[derive(Debug)]
pub struct File {
    pub path: PathBuf,
}

impl File {
    pub fn new(path: &Path) -> File {
        File { path: path.canonicalize().unwrap() }
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

    pub fn add_files(&self, paths: &[File]) -> Result<usize> {
        self.connection.execute("INSERT INTO File (path) values (?)", paths)
    }
}

impl ToSql for File {
    fn to_sql(&self) -> Result<ToSqlOutput> {
        // The path must be convertible to UTF-8 as we're storing this in a DB.
        Ok(ToSqlOutput::from(self.path.to_str().unwrap()))
    }
}
