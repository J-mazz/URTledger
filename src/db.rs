use rusqlite::{params, Connection, Result};
use crate::model::InventoryBatch;
use std::sync::{Arc, Mutex};
use std::path::Path;

// Deriving Clone here allows us to pass a pointer to the SAME connection
// to multiple UI callbacks without fighting the borrow checker.
#[derive(Clone)]
pub struct Database {
    conn: Arc<Mutex<Connection>>,
}

impl Database {
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        let conn = Connection::open(path)?;

        // 1. Enable Foreign Keys (returns no rows)
        conn.execute("PRAGMA foreign_keys = ON;", [])?;

        // 2. Enable WAL Mode - this PRAGMA returns the new mode ("wal"), so consume with query_row
        let _mode: String = conn.query_row("PRAGMA journal_mode = WAL;", [], |row| row.get(0))?;

        // 3. Inventory table
        conn.execute(
            "CREATE TABLE IF NOT EXISTS inventory_batches (
                id INTEGER PRIMARY KEY,
                name TEXT NOT NULL,
                type_id INTEGER,
                grade_id INTEGER,
                stage_id INTEGER,
                weight REAL NOT NULL,
                price REAL NOT NULL,
                specs_json TEXT NOT NULL,
                created_at TEXT DEFAULT CURRENT_TIMESTAMP
            )",
            [],
        )?;

        // 4. Stages table
        conn.execute(
            "CREATE TABLE IF NOT EXISTS stages (
                id INTEGER PRIMARY KEY,
                name TEXT NOT NULL UNIQUE
            )",
            [],
        )?;

        // 5. Grades table
        conn.execute(
            "CREATE TABLE IF NOT EXISTS grades (
                id INTEGER PRIMARY KEY,
                name TEXT NOT NULL UNIQUE
            )",
            [],
        )?;

        Ok(Database {
            conn: Arc::new(Mutex::new(conn)),
        })
    }

    pub fn insert_inventory_batch(&self, b: &InventoryBatch) -> Result<i64> {
        // Lock the mutex to get exclusive access to the connection
        let conn = self.conn.lock().unwrap();

        let specs_str = serde_json::to_string(&b.specs)
            .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;

        conn.execute(
            "INSERT INTO inventory_batches (name, type_id, grade_id, stage_id, weight, price, specs_json)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                b.name,
                b.type_id,
                b.grade_id,
                b.stage_id,
                b.weight,
                b.price,
                specs_str
            ],
        )?;

        Ok(conn.last_insert_rowid())
    }

    pub fn aggregate_stage_totals(&self, stage_id: i64) -> Result<(f64, usize)> {
        let conn = self.conn.lock().unwrap();

        let mut stmt = conn.prepare(
            "SELECT COALESCE(SUM(weight), 0.0), COUNT(*) 
             FROM inventory_batches 
             WHERE stage_id = ?1",
        )?;

        let result = stmt.query_row(params![stage_id], |row| {
            let total_weight: f64 = row.get(0)?;
            let count: usize = row.get(1)?;
            Ok((total_weight, count))
        })?;

        Ok(result)
    }

    // --- Config (stages/grades) methods ---
    pub fn insert_stage(&self, name: String) -> Result<i64> {
        let conn = self.conn.lock().unwrap();
        conn.execute("INSERT OR IGNORE INTO stages (name) VALUES (?1)", params![name])?;
        Ok(conn.last_insert_rowid())
    }

    pub fn get_all_stages(&self) -> Result<Vec<(i64, String)>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT id, name FROM stages ORDER BY id ASC")?;
        let rows = stmt.query_map([], |row| Ok((row.get(0)?, row.get(1)?)))?;
        let mut result = Vec::new();
        for r in rows { result.push(r?); }
        Ok(result)
    }

    pub fn insert_grade(&self, name: String) -> Result<i64> {
        let conn = self.conn.lock().unwrap();
        conn.execute("INSERT OR IGNORE INTO grades (name) VALUES (?1)", params![name])?;
        Ok(conn.last_insert_rowid())
    }

    pub fn get_all_grades(&self) -> Result<Vec<(i64, String)>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT id, name FROM grades ORDER BY id ASC")?;
        let rows = stmt.query_map([], |row| Ok((row.get(0)?, row.get(1)?)))?;
        let mut result = Vec::new();
        for r in rows { result.push(r?); }
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::InventoryBatch;
    use tempfile::NamedTempFile;
    use std::collections::HashMap;

    #[test]
    fn insert_batch_and_aggregate() {
        let f = NamedTempFile::new().unwrap();
        let db = Database::open(f.path()).unwrap();
        // Insert a batch with stage_id = 5
        let mut specs = HashMap::new();
        specs.insert("THC".into(), 12.5);
        let b = InventoryBatch {
            name: "Batch 1".into(),
            type_id: 1,
            grade_id: 1,
            stage_id: 5,
            weight: 10.0,
            price: 2.5,
            specs,
        };
        let _ = db.insert_inventory_batch(&b).unwrap();
        let (w, c) = db.aggregate_stage_totals(5).unwrap();
        assert_eq!(w, 10.0);
        assert_eq!(c, 1);
    }
}
