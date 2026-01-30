use rusqlite::{params, Connection, Result};
use crate::model::{InventoryBatch, ProductTemplate};
use std::sync::{Arc, Mutex};
use std::path::Path;
use serde_json;

#[derive(Clone)]
pub struct Database {
    pub conn: Arc<Mutex<Connection>>,
}

impl Database {
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        let conn = Connection::open(path)?;

        // 1. Enable Foreign Keys
        conn.execute("PRAGMA foreign_keys = ON;", [])?;

        // 2. Enable WAL Mode (Consume result to avoid panic)
        let _mode: String = conn.query_row("PRAGMA journal_mode = WAL;", [], |row| row.get(0))?;

        // 3. Initialize Tables matching migrations/initial.sql
        
        // Table: Configuration (Stores 'grade' and 'stage' definitions)
        conn.execute(
            "CREATE TABLE IF NOT EXISTS Configuration (
                id INTEGER PRIMARY KEY,
                kind TEXT NOT NULL, -- 'grade' or 'stage'
                name TEXT NOT NULL
            )",
            [],
        )?;

        // Table: ProductTemplates (Stores User-defined types like 'Blue Dream')
        conn.execute(
            "CREATE TABLE IF NOT EXISTS ProductTemplates (
                id INTEGER PRIMARY KEY,
                name TEXT NOT NULL,
                specs_json TEXT NOT NULL -- e.g. [\"THC\", \"Terpenes\"]
            )",
            [],
        )?;

        // Table: Inventory Batches
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

        Ok(Database {
            conn: Arc::new(Mutex::new(conn)),
        })
    }

    // --- Inventory Methods ---

    pub fn insert_inventory_batch(&self, b: &InventoryBatch) -> Result<i64> {
        let conn = self.conn.lock().unwrap();
        let specs_str = serde_json::to_string(&b.specs)
            .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;

        conn.execute(
            "INSERT INTO inventory_batches (name, type_id, grade_id, stage_id, weight, price, specs_json)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![b.name, b.type_id, b.grade_id, b.stage_id, b.weight, b.price, specs_str],
        )?;

        Ok(conn.last_insert_rowid())
    }

    pub fn aggregate_stage_totals(&self, stage_id: i64) -> Result<(f64, usize)> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT COALESCE(SUM(weight), 0.0), COUNT(*) 
             FROM inventory_batches 
             WHERE stage_id = ?1"
        )?;

        let result = stmt.query_row(params![stage_id], |row| {
            let total_weight: f64 = row.get(0)?;
            let count: usize = row.get(1)?;
            Ok((total_weight, count))
        })?;

        Ok(result)
    }

    // --- Configuration Methods ---

    /// Inserts a new configuration item (e.g., Kind="stage", Name="Drying")
    pub fn insert_config(&self, kind: &str, name: &str) -> Result<i64> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO Configuration (kind, name) VALUES (?1, ?2)",
            params![kind, name],
        )?;
        Ok(conn.last_insert_rowid())
    }

    /// Retrieves all items of a specific kind (e.g., all "stage" items)
    pub fn get_config_items(&self, kind: &str) -> Result<Vec<(i64, String)>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT id, name FROM Configuration WHERE kind = ?1 ORDER BY id ASC")?;
        
        let rows = stmt.query_map(params![kind], |row| {
            Ok((row.get(0)?, row.get(1)?))
        })?;

        let mut result = Vec::new();
        for r in rows {
            result.push(r?);
        }
        Ok(result)
    }
}