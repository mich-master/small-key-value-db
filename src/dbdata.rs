use std::collections::HashMap;
use std::sync::atomic::{ AtomicU32, Ordering };
use std::sync::Mutex;

use crate::error::DbError;

struct DbMap {
    data: HashMap<String,String>,
    modified: bool,
}

impl DbMap {
    fn init(filepath: &str) -> DbMap {
        DbMap {
            data: file_load(filepath).unwrap_or_default(),
            modified: false,
        }
    }
}

pub struct DbData {
    db: Mutex<DbMap>,
    inserts_successful: AtomicU32,
    inserts_failed:     AtomicU32,
    updates_successful: AtomicU32,
    updates_failed:     AtomicU32,
    deletes_successful: AtomicU32,
    deletes_failed:     AtomicU32,
    gets_successful:    AtomicU32,
    gets_failed:        AtomicU32,
}

impl DbData {
    pub fn create(filepath: &str) -> DbData {
        DbData {
            db: Mutex::new(DbMap::init(filepath)),
            inserts_successful: AtomicU32::new(0),
            inserts_failed:     AtomicU32::new(0),
            updates_successful: AtomicU32::new(0),
            updates_failed:     AtomicU32::new(0),
            deletes_successful: AtomicU32::new(0),
            deletes_failed:     AtomicU32::new(0),
            gets_successful:    AtomicU32::new(0),
            gets_failed:        AtomicU32::new(0),
        }
    }
    pub fn dump(&self, filepath: &str) -> Result<(),()> {
        let result: Result<Option<String>,()> =
            if let Ok(mut dbmap) = self.db.lock() {
                if dbmap.modified {
                    serde_json::to_string_pretty(&dbmap.data)
                        .map(|s| { dbmap.modified = false; Some(s) } )
                        .map_err(|_|())
                } else {
                    Ok(None)
                }
            } else {
                Result::Err(())
            };
        result.map(|opt_s| {
            if let Some(s) = opt_s {
                file_dump(s, filepath).unwrap();
            };
            ()
        })
    }
    pub fn insert(&self, key: String, value: String) -> Result<(),DbError> {
        let db_result: Result<(),DbError> =
          self.db.lock()
              .map_or(Result::Err(DbError::DatabaseError), |mut dbmap| {
                  if dbmap.data.contains_key(&key) {
                    Result::Err(DbError::InsertKeyExist)
                  } else {
                    dbmap.data.insert(key,value);
                    dbmap.modified = true;
                    Ok(())
                  }
              }
          );
        match &db_result {
            Ok(_)   =>
              self.inserts_successful.fetch_add(1,Ordering::Relaxed),
            Err(_)  =>
              self.inserts_failed.fetch_add(1,Ordering::Relaxed),
        };
        db_result
    }
    pub fn update(&self, key: String, value: String) -> Result<(),DbError> {
        let db_result: Result<(),DbError> =
          self.db.lock()
              .map_or(Result::Err(DbError::DatabaseError), |mut dbmap| {
                  if let Some(v) = dbmap.data.get_mut(&key) {
                    if *v == value {
                        Result::Err(DbError::UpdateValueMatch)
                    } else {
                        *v = value;
                        dbmap.modified = true;
                        Ok(())
                    }
                  } else {
                      Result::Err(DbError::UpdateKeyNotExist)
                  }
              }
          );
        match &db_result {
            Ok(_)   =>
              self.updates_successful.fetch_add(1,Ordering::Relaxed),
            Err(_)  =>
              self.updates_failed.fetch_add(1,Ordering::Relaxed),
        };
        db_result
    }
    pub fn delete(&self, key: String) -> Result<(),DbError> {
        let db_result: Result<(),DbError> =
          self.db.lock()
              .map_or(Result::Err(DbError::DatabaseError), |mut dbmap| {
                  if let Some(_) = dbmap.data.remove(&key) {
                    dbmap.modified = true;
                    Ok(())
                  } else {
                    Result::Err(DbError::DeleteKeyNotExist)
                  }
              }
          );
        match &db_result {
            Ok(_)   =>
              self.deletes_successful.fetch_add(1,Ordering::Relaxed),
            Err(_)  =>
              self.deletes_failed.fetch_add(1,Ordering::Relaxed),
        };
        db_result
    }
    pub fn get(&self, key: String) -> Result<String,DbError> {
        let db_result: Result<String,DbError> =
          self.db.lock()
              .map_or(Result::Err(DbError::DatabaseError), |dbmap| {
                  if let Some(value) = dbmap.data.get(&key) {
                    Ok(value.clone())
                  } else {
                    Result::Err(DbError::GetKeyNotExist)
                  }
              }
          );
        match &db_result {
            Ok(_)   =>
              self.gets_successful.fetch_add(1,Ordering::Relaxed),
            Err(_)  =>
              self.gets_failed.fetch_add(1,Ordering::Relaxed),
        };
        db_result
    }
    pub fn statistics(&self) -> Result<String,()> {
        let mut text: String = String::new();
        text.push_str(
            &format!("\nThe number of records in database: {}\n",
                self.db.lock()
                    .map(|dbmap| {
                        dbmap.data.len()
                    })
                    .map_err(|_|())?
            )
        );
        text.push_str(
            &format!("INSERT operations: {} successful / {} unsuccessful\n",
                self.inserts_successful.load(Ordering::Relaxed),
                self.inserts_failed.load(Ordering::Relaxed)
            )
        );
        text.push_str(
            &format!("UPDATE operations: {} successful / {} unsuccessful\n",
                self.updates_successful.load(Ordering::Relaxed),
                self.updates_failed.load(Ordering::Relaxed)
            )
        );
        text.push_str(
            &format!("DELETE operations: {} successful / {} unsuccessful\n",
                self.deletes_successful.load(Ordering::Relaxed),
                self.deletes_failed.load(Ordering::Relaxed)
            )
        );
        text.push_str(
            &format!("GET operations: {} successful / {} unsuccessful\n",
                self.gets_successful.load(Ordering::Relaxed),
                self.gets_failed.load(Ordering::Relaxed)
            )
        );
        Ok(text)
    }
}

fn file_load(filepath: &str) -> Result<HashMap<String,String>,std::io::Error> {
    std::fs::File::open(filepath).and_then(|file| {
        let reader = std::io::BufReader::new(file);
        serde_json::from_reader(reader).map_err(|e|e.into())
    })
}

fn file_dump(dump: String, filepath: &str) -> Result<(),std::io::Error> {
    std::fs::write(filepath, dump)
}
