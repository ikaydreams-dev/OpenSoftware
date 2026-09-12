use rusqlite::{Connection, Result as SqliteResult};
use std::path::PathBuf;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum DatabaseError {
    #[error("Database error: {0}")]
    SqliteError(#[from] rusqlite::Error),

    #[error("Collection '{0}' already exists")]
    CollectionExists(String),

    #[error("Failed to create database: {0}")]
    CreationError(String),

    #[error("Query error: {0}")]
    QueryError(String),
}

pub struct Database {
    connection: Connection,
    name: String,
}

impl Database {
    pub fn new(name: &str) -> Result<Self, DatabaseError> {
        // Create databases directory if it doesn't exist
        let db_dir = PathBuf::from("databases");
        std::fs::create_dir_all(&db_dir)
            .map_err(|e| DatabaseError::CreationError(e.to_string()))?;

        // Create database file
        let db_path = db_dir.join(format!("{}.db", name));
        let connection = Connection::open(&db_path)?;

        Ok(Self {
            connection,
            name: name.to_string(),
        })
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn create_collection(&mut self, collection_name: &str) -> Result<(), DatabaseError> {
        // In SQLite, collections are tables
        // Create a simple table with id and data columns
        let query = format!(
            "CREATE TABLE IF NOT EXISTS {} (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                data TEXT NOT NULL,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP
            )",
            sanitize_table_name(collection_name)
        );

        self.connection.execute(&query, [])?;
        Ok(())
    }

    pub fn reset_collection(&mut self, collection_name: &str) -> Result<(), DatabaseError> {
        let name = sanitize_table_name(collection_name);
        self.connection.execute(&format!("DELETE FROM {}", name), [])?;
        self.connection.execute(&format!("DELETE FROM sqlite_sequence WHERE name = '{}'", name), [])?;
        Ok(())
    }

    pub fn insert(&mut self, collection_name: &str, data: &str) -> Result<(), DatabaseError> {
        let query = format!(
            "INSERT INTO {} (data) VALUES (?)",
            sanitize_table_name(collection_name)
        );

        self.connection.execute(&query, [data])?;
        Ok(())
    }

    pub fn select_all(&self, collection_name: &str) -> Result<Vec<String>, DatabaseError> {
        let query = format!(
            "SELECT data FROM {}",
            sanitize_table_name(collection_name)
        );

        let mut stmt = self.connection.prepare(&query)?;
        let rows = stmt
            .query_map([], |row| row.get(0))?
            .collect::<SqliteResult<Vec<String>>>()?;

        Ok(rows)
    }

    pub fn select_rows(&self, collection_name: &str) -> Result<Vec<(i64, String)>, DatabaseError> {
        let query = format!(
            "SELECT id, data FROM {}",
            sanitize_table_name(collection_name)
        );

        let mut stmt = self.connection.prepare(&query)?;
        let rows = stmt
            .query_map([], |row| Ok((row.get::<_, i64>(0)?, row.get::<_, String>(1)?)))?
            .collect::<SqliteResult<Vec<(i64, String)>>>()?;

        Ok(rows)
    }

    pub fn update_row(&self, collection_name: &str, id: i64, data: &str) -> Result<(), DatabaseError> {
        let query = format!(
            "UPDATE {} SET data = ? WHERE id = ?",
            sanitize_table_name(collection_name)
        );

        self.connection.execute(&query, rusqlite::params![data, id])?;
        Ok(())
    }

    pub fn delete_rows_by_ids(&self, collection_name: &str, ids: &[i64]) -> Result<(), DatabaseError> {
        if ids.is_empty() {
            return Ok(());
        }

        let table = sanitize_table_name(collection_name);
        let condition = ids.iter().map(|id| format!("id = {}", id)).collect::<Vec<_>>().join(" OR ");
        let query = format!("DELETE FROM {} WHERE {}", table, condition);

        self.connection.execute(&query, [])?;
        Ok(())
    }

    pub fn list_collections(&self) -> Result<Vec<String>, DatabaseError> {
        let mut stmt = self.connection
            .prepare("SELECT name FROM sqlite_master WHERE type='table' ORDER BY name")?;

        let collections = stmt
            .query_map([], |row| row.get(0))?
            .collect::<SqliteResult<Vec<String>>>()?;

        Ok(collections)
    }

    pub fn insert_with_id(&mut self, collection_name: &str, data: &str) -> Result<i64, DatabaseError> {
        let query = format!(
            "INSERT INTO {} (data) VALUES (?)",
            sanitize_table_name(collection_name)
        );
        self.connection.execute(&query, [data])?;
        Ok(self.connection.last_insert_rowid())
    }

    // Returns rows (id, parsed JSON) matching an equality filter, optionally sorted.
    pub fn select_filtered(
        &self,
        collection_name: &str,
        filter: &serde_json::Value,
        sort_field: Option<&str>,
        desc: bool,
    ) -> Result<Vec<(i64, serde_json::Value)>, DatabaseError> {
        let mut rows: Vec<(i64, serde_json::Value)> = Vec::new();
        for (id, data) in self.select_rows(collection_name)? {
            let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&data) else {
                continue;
            };
            if matches_condition(&parsed, filter) {
                rows.push((id, parsed));
            }
        }

        if let Some(field) = sort_field {
            rows.sort_by(|a, b| compare_values(&a.1, &b.1, field));
            if desc {
                rows.reverse();
            }
        }

        Ok(rows)
    }

    pub fn get_row_by_id(&self, collection_name: &str, id: i64) -> Result<Option<serde_json::Value>, DatabaseError> {
        for (row_id, data) in self.select_rows(collection_name)? {
            if row_id == id {
                return Ok(serde_json::from_str::<serde_json::Value>(&data).ok());
            }
        }
        Ok(None)
    }

    pub fn delete_by_id(&mut self, collection_name: &str, id: i64) -> Result<bool, DatabaseError> {
        let query = format!(
            "DELETE FROM {} WHERE id = ?",
            sanitize_table_name(collection_name)
        );
        let affected = self.connection.execute(&query, rusqlite::params![id])?;
        Ok(affected > 0)
    }

    // Starts a database transaction. All writes made after this call are
    // rolled back if rollback() is called, or made permanent on commit().
    pub fn begin_transaction(&mut self) -> Result<(), DatabaseError> {
        self.connection.execute("BEGIN", [])?;
        Ok(())
    }

    pub fn commit_transaction(&mut self) -> Result<(), DatabaseError> {
        self.connection.execute("COMMIT", [])?;
        Ok(())
    }

    pub fn rollback_transaction(&mut self) -> Result<(), DatabaseError> {
        self.connection.execute("ROLLBACK", [])?;
        Ok(())
    }
}

// Sanitize table name to prevent SQL injection
fn sanitize_table_name(name: &str) -> String {
    name.chars()
        .filter(|c| c.is_alphanumeric() || *c == '_')
        .collect()
}

pub fn create_database(name: &str) -> Result<Database, DatabaseError> {
    Database::new(name)
}

pub fn create_collection(db: &mut Database, name: &str) -> Result<(), DatabaseError> {
    db.create_collection(name)
}

pub fn reset_collection(db: &mut Database, name: &str) -> Result<(), DatabaseError> {
    db.reset_collection(name)
}

pub fn insert_data(db: &mut Database, collection: &str, data: &str) -> Result<(), DatabaseError> {
    db.insert(collection, data)
}

pub fn select_all(db: &Database, collection: &str) -> Result<Vec<String>, DatabaseError> {
    db.select_all(collection)
}

pub fn select_rows(db: &Database, collection: &str) -> Result<Vec<(i64, String)>, DatabaseError> {
    db.select_rows(collection)
}

pub fn update_row(db: &Database, collection: &str, id: i64, data: &str) -> Result<(), DatabaseError> {
    db.update_row(collection, id, data)
}

pub fn delete_rows_by_ids(db: &Database, collection: &str, ids: &[i64]) -> Result<(), DatabaseError> {
    db.delete_rows_by_ids(collection, ids)
}

pub fn insert_with_id(db: &mut Database, collection: &str, data: &str) -> Result<i64, DatabaseError> {
    db.insert_with_id(collection, data)
}

pub fn select_filtered(
    db: &Database,
    collection: &str,
    filter: &serde_json::Value,
    sort_field: Option<&str>,
    desc: bool,
) -> Result<Vec<(i64, serde_json::Value)>, DatabaseError> {
    db.select_filtered(collection, filter, sort_field, desc)
}

pub fn get_row_by_id(db: &Database, collection: &str, id: i64) -> Result<Option<serde_json::Value>, DatabaseError> {
    db.get_row_by_id(collection, id)
}

pub fn delete_by_id(db: &mut Database, collection: &str, id: i64) -> Result<bool, DatabaseError> {
    db.delete_by_id(collection, id)
}

pub fn begin_transaction(db: &mut Database) -> Result<(), DatabaseError> {
    db.begin_transaction()
}

pub fn commit_transaction(db: &mut Database) -> Result<(), DatabaseError> {
    db.commit_transaction()
}

pub fn rollback_transaction(db: &mut Database) -> Result<(), DatabaseError> {
    db.rollback_transaction()
}

pub fn query_data(db: &Database, collection: &str, condition: &str) -> Result<Vec<String>, DatabaseError> {
    if condition.trim().is_empty() {
        return db.select_all(collection);
    }

    let cond: serde_json::Value = serde_json::from_str(condition)
        .map_err(|e| DatabaseError::QueryError(e.to_string()))?;

    let matching = db
        .select_rows(collection)?
        .into_iter()
        .map(|(_, data)| data)
        .filter(|data| {
            let Ok(row) = serde_json::from_str::<serde_json::Value>(data) else {
                return false;
            };
            matches_condition(&row, &cond)
        })
        .collect();

    Ok(matching)
}

pub fn update_data(db: &Database, collection: &str, update: &str, condition: &str) -> Result<(), DatabaseError> {
    let update_obj: serde_json::Map<String, serde_json::Value> = serde_json::from_str(update)
        .map_err(|e| DatabaseError::QueryError(e.to_string()))?;

    let cond: serde_json::Value = serde_json::from_str(condition)
        .map_err(|e| DatabaseError::QueryError(e.to_string()))?;

    let rows = db.select_rows(collection)?;

    for (id, data) in rows {
        let Ok(mut row) = serde_json::from_str::<serde_json::Value>(&data) else {
            continue;
        };

        if !matches_condition(&row, &cond) {
            continue;
        }

        if let Some(obj) = row.as_object_mut() {
            for (k, v) in &update_obj {
                obj.insert(k.clone(), v.clone());
            }
        }

        db.update_row(collection, id, &row.to_string())?;
    }

    Ok(())
}

fn matches_condition(row: &serde_json::Value, cond: &serde_json::Value) -> bool {
    let Some(cond_obj) = cond.as_object() else {
        return false;
    };

    cond_obj.iter().all(|(key, expected)| {
        match (row.get(key), expected) {
            (Some(serde_json::Value::String(a)), serde_json::Value::String(b)) => a == b,
            (Some(serde_json::Value::Number(a)), serde_json::Value::Number(b)) => a == b,
            (Some(serde_json::Value::Bool(a)), serde_json::Value::Bool(b)) => a == b,
            _ => false,
        }
    })
}

// Compares two JSON rows by the given field, for sorting.
fn compare_values(a: &serde_json::Value, b: &serde_json::Value, field: &str) -> std::cmp::Ordering {
    match (a.get(field), b.get(field)) {
        (Some(serde_json::Value::Number(x)), Some(serde_json::Value::Number(y))) => {
            x.as_f64().partial_cmp(&y.as_f64()).unwrap_or(std::cmp::Ordering::Equal)
        }
        (Some(x), Some(y)) => {
            let xs = match x {
                serde_json::Value::String(s) => s.clone(),
                other => other.to_string(),
            };
            let ys = match y {
                serde_json::Value::String(s) => s.clone(),
                other => other.to_string(),
            };
            xs.cmp(&ys)
        }
        (None, None) => std::cmp::Ordering::Equal,
        (Some(_), None) => std::cmp::Ordering::Greater,
        (None, Some(_)) => std::cmp::Ordering::Less,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_create_database() {
        let db = Database::new("test_db").unwrap();
        assert_eq!(db.name(), "test_db");

        // Cleanup
        let _ = fs::remove_file("databases/test_db.db");
    }

    #[test]
    fn test_create_collection() {
        let mut db = Database::new("test_collections").unwrap();

        assert!(db.create_collection("users").is_ok());
        assert!(db.create_collection("posts").is_ok());

        let collections = db.list_collections().unwrap();
        assert!(collections.contains(&"users".to_string()));
        assert!(collections.contains(&"posts".to_string()));

        // Cleanup
        let _ = fs::remove_file("databases/test_collections.db");
    }

    #[test]
    fn test_sanitize_table_name() {
        assert_eq!(sanitize_table_name("users"), "users");
        assert_eq!(sanitize_table_name("user_data"), "user_data");
        assert_eq!(sanitize_table_name("users; DROP TABLE"), "usersDROPTABLE");
    }
}
