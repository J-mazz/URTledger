use rusqlite::{params, Connection, Result};
use crate::model::InventoryBatch;
use std::sync::{Arc, Mutex};
use std::path::Path;

#[derive(Clone)]
pub struct Database {
    conn: Arc<Mutex<Connection>>,
}

impl Database {
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        let conn = Connection::open(path)?;

        conn.execute("PRAGMA foreign_keys = ON;", [])?;
        let _mode: String = conn.query_row("PRAGMA journal_mode = WAL;", [], |row| row.get(0))?;

        // 1. Inventory Batches
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

        // 2. Stages
        conn.execute(
            "CREATE TABLE IF NOT EXISTS stages (
                id INTEGER PRIMARY KEY,
                name TEXT NOT NULL UNIQUE
            )",
            [],
        )?;

        // 3. Grades
        conn.execute(
            "CREATE TABLE IF NOT EXISTS grades (
                id INTEGER PRIMARY KEY,
                name TEXT NOT NULL UNIQUE
            )",
            [],
        )?;

        // 4. Product Types
        conn.execute(
            "CREATE TABLE IF NOT EXISTS product_types (
                id INTEGER PRIMARY KEY,
                name TEXT NOT NULL UNIQUE,
                specs_keys_json TEXT NOT NULL
            )",
            [],
        )?;

        Ok(Database {
            conn: Arc::new(Mutex::new(conn)),
        })
    }

    /// Pre-populates the DB with the user's requested defaults if empty
    pub fn seed_defaults(&self) -> Result<()> {
        let conn = self.conn.lock().unwrap();

        // 1. Seed Stages: Unbucked, Bucked, Rolled, Processed
        let count: i64 = conn.query_row("SELECT COUNT(*) FROM stages", [], |r| r.get(0))?;
        if count == 0 {
            let defaults = vec!["Unbucked", "Bucked", "Rolled", "Processed"];
            let mut stmt = conn.prepare("INSERT INTO stages (name) VALUES (?1)")?;
            for name in defaults {
                stmt.execute(params![name])?;
            }
        }

        // 2. Seed Grades: A, B, C, Trim
        let count: i64 = conn.query_row("SELECT COUNT(*) FROM grades", [], |r| r.get(0))?;
        if count == 0 {
            let defaults = vec!["A", "B", "C", "Trim"];
            let mut stmt = conn.prepare("INSERT INTO grades (name) VALUES (?1)")?;
            for name in defaults {
                stmt.execute(params![name])?;
            }
        }

        Ok(())
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
        let mut stmt = conn.prepare("SELECT COALESCE(SUM(weight), 0.0), COUNT(*) FROM inventory_batches WHERE stage_id = ?1")?;
        Ok(stmt.query_row(params![stage_id], |row| Ok((row.get(0)?, row.get(1)?)))?)
    }

    // --- Config Methods ---
    
    // Stages
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

    // Grades
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

    // Product Types
    pub fn insert_product_type(&self, name: String, specs_keys: Vec<String>) -> Result<i64> {
        let conn = self.conn.lock().unwrap();
        let specs_json = serde_json::to_string(&specs_keys)
            .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;
        
        conn.execute("INSERT OR IGNORE INTO product_types (name, specs_keys_json) VALUES (?1, ?2)", params![name, specs_json])?;
        Ok(conn.last_insert_rowid())
    }
    
    pub fn get_all_product_types(&self) -> Result<Vec<(i64, String, Vec<String>)>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT id, name, specs_keys_json FROM product_types ORDER BY id ASC")?;
        let rows = stmt.query_map([], |row| {
            let json_str: String = row.get(2)?;
            let keys: Vec<String> = serde_json::from_str(&json_str).unwrap_or_default();
            Ok((row.get(0)?, row.get(1)?, keys))
        })?;
        let mut result = Vec::new();
        for r in rows { result.push(r?); }
        Ok(result)
    }
}