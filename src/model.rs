use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use crate::db::Database;

/// Represents a single unit of inventory. Deriving Serialize/Deserialize allows
/// the 'specs' map to be converted to JSON automatically when persisted.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InventoryBatch {
    pub name: String,
    pub type_id: i64,      // References a specific user-defined Product Type
    pub grade_id: i64,     // References a user-defined Grade
    pub stage_id: i64,     // References a user-defined Stage
    pub weight: f64,       // High precision float for weight
    pub price: f64,        // User-defined price per unit
    pub specs: HashMap<String, f64>, // Dynamic key-value pairs (e.g., "THC": 22.5)
}

impl InventoryBatch {
    /// Helper to calculate the total value of this specific batch.
    /// Pure arithmetic: Weight * Price.
    pub fn total_value(&self) -> f64 {
        self.weight * self.price
    }
}

/// The high-level application state. Holds the database connection and exposes
/// business logic to the UI.
pub struct Model {
    pub db: Database,
}

impl Model {
    /// Initialize the model with an active database connection.
    pub fn new(db: Database) -> Self {
        Model { db }
    }

    /// Aggregates total weight and count for a given stage id. This is the
    /// primary function used to populate the dashboard tiles.
    pub fn aggregate_stage(&self, stage_id: i64) -> (f64, usize) {
        match self.db.aggregate_stage_totals(stage_id) {
            Ok((weight, count)) => (weight, count),
            Err(e) => {
                eprintln!("Error aggregating stage {}: {}", stage_id, e);
                (0.0, 0)
            }
        }
    }
}

// Optional: typed ProductTemplate for future UI bindings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProductTemplate {
    pub id: i64,
    pub name: String,
    pub required_specs: Vec<String>, // List of keys (e.g., ["THC", "Moisture"])
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::Database;
    use tempfile::NamedTempFile;
    use std::collections::HashMap;

    #[test]
    fn batch_total_value() {
        let b = InventoryBatch { name: "B".into(), type_id: 1, grade_id: 1, stage_id: 1, weight: 2.0, price: 3.0, specs: HashMap::new() };
        assert_eq!(b.total_value(), 6.0);
    }
}
