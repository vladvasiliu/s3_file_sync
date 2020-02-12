use std::path::{Path, PathBuf};

use libsqlite3_sys::{Error as LibSQLError, ErrorCode as LibSQLErrorCode};
use log::{trace, warn};
use rusqlite::{params, Connection, Error as SQLError, OpenFlags, Row, Statement, NO_PARAMS};

use crate::controller::file::File;

pub mod error;
use error::{Error, Result};

pub struct Database {
    connection: Connection,
}

impl Database {
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Database> {
        let connection = Connection::open_with_flags(
            path,
            OpenFlags::SQLITE_OPEN_READ_WRITE
                | OpenFlags::SQLITE_OPEN_CREATE
                | OpenFlags::SQLITE_OPEN_FULL_MUTEX,
        )?;
        let database = Database { connection };
        database.init_db()?;
        Ok(database)
    }

    fn init_db(&self) -> Result<()> {
        self.connection.execute_batch(
            "BEGIN;
                  CREATE TABLE IF NOT EXISTS File (
                          path            TEXT PRIMARY KEY,
                          first_seen_date TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
                          uploaded_date   TEXT,
                          deleted_date    TEXT
                  );
                  CREATE INDEX IF NOT EXISTS file_uploaded ON File ( uploaded_date );
                  CREATE INDEX IF NOT EXISTS file_not_deleted ON File ( deleted_date )
                          WHERE deleted_date IS NULL and uploaded_date IS NOT NULL;
              COMMIT;",
        )?;
        Ok(())
    }

    pub fn add_file(&self, file: &File) -> Result<()> {
        let mut statement = self
            .connection
            .prepare_cached("INSERT INTO File (path) values (?1)")?;
        match statement.insert(&[file.full_path.to_str().unwrap()]) {
            Ok(_) => Ok(()),
            Err(
                err
                @
                SQLError::SqliteFailure(
                    LibSQLError {
                        code: LibSQLErrorCode::ConstraintViolation,
                        ..
                    },
                    ..,
                ),
            ) => Err(Error::FileExists(err)),
            Err(err) => Err(Error::Unhandled(err)),
        }
    }

    //    pub fn populate(&mut self) -> Result<()> {
    //        let tx = self.connection.transaction()?;
    //        {
    //            let mut stmt = tx.prepare("INSERT INTO File (path, first_seen_date) values (?1, ?2)")?;
    //            let mut x = 0;
    //            let update = "2020-02-02T02:02:02Z";
    //            while x < 1_000_000 {
    //                let base_name = format!("toto_{:06}", x);
    //                stmt.execute(params![&base_name, &update])?;
    //                x += 1;
    //                if x % 10_000 == 0 {
    //                    info!("Inserted {}", x);
    //                }
    //            }
    //        }
    //        tx.commit()?;
    //        Ok(())
    //    }

    //    pub fn files_to_upload(&self) -> Result<Vec<File>> {
    //        let mut statement = self
    //            .connection
    //            .prepare_cached("SELECT * FROM File WHERE uploaded_date IS NULL")?;
    //        let mut rows = statement.query(NO_PARAMS)?;
    //
    //        let mut files: Vec<File> = Vec::new();
    //
    //        loop {
    //            match rows.next() {
    //                Ok(None) => break,
    //                Ok(Some(row)) => match file_from_row(row) {
    //                    Ok(file) => {
    //                        trace!("Loaded file: {:?}", file);
    //                        files.push(file);
    //                    }
    //                    Err(err) => {
    //                        let file_path: String = row.get_unwrap("path");
    //                        warn!(
    //                            "Failed to load file '{}' from DB. Error: {:?}.",
    //                            file_path, err
    //                        );
    //                    }
    //                },
    //                Err(err) => warn!("Error looking for files: {:?}", err),
    //            };
    //        }
    //
    //        Ok(files)
    //    }
}
