use std::fs::File;
use std::io::Write;
use std::thread;
use std::time::Duration;

use rusqlite::{Connection, Result};
use serde_derive::*;
use thiserror::Error;

#[derive(Error, Debug)]
enum MyError {
    #[error("database error: {0}")]
    DatabaseError(#[from] rusqlite::Error),
    #[error("json error: {0}")]
    JsonError(#[from] serde_json::Error),
    #[error("file error: {0}")]
    RusQLite(#[from] std::io::Error),
}

#[derive(Debug, Serialize, Deserialize)]
struct Record {
   id: i32,
   name: String,
   age: i32,
}

fn main() -> Result<(), MyError> {
   let connection = Connection::open("test.db")?;
   connection.execute(
       "CREATE TABLE IF NOT EXISTS people (
                 id              INTEGER PRIMARY KEY,
                 name            TEXT NOT NULL,
                 age             INTEGER NOT NULL
                 )",
       [],
   )?;

   let mut last_id = 0;

   loop {
       let mut stmt = connection.prepare("SELECT * FROM people WHERE id > ?")?;
       let rows = stmt.query_map([&last_id], |row| {
           Ok(Record {
               id: row.get(0)?,
               name: row.get(1)?,
               age: row.get(2)?,
           })
       })?;

       let mut file = File::create("output.json")?;
       for row in rows {
           let record = row?;
           let serialized = serde_json::to_string(&record)?;
           file.write_all(serialized.as_bytes())?;
           file.write_all(b"\n")?;
           last_id = record.id;
       }

       thread::sleep(Duration::from_secs(1));
   }
}
