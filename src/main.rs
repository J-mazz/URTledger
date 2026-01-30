use slint::{ComponentHandle, ModelRc, SharedString, VecModel};
use crate::db::Database;
use crate::model::{InventoryBatch, Model};
use std::collections::HashMap;

pub mod db;
pub mod model;

slint::include_modules!();

// --- Helper Functions ---

/// Fetches Stages and Grades from the DB and pushes them to the UI
fn refresh_config_lists(ui: &MainWindow, db: &Database) {
    // 1. Refresh Stages
    if let Ok(stages) = db.get_config_items("stage") {
        let stage_items: Vec<ConfigItem> = stages.into_iter().map(|(id, name)| {
            ConfigItem { 
                id: id as i32, 
                name: SharedString::from(name) 
            }
        }).collect();
        ui.set_stages_list(ModelRc::new(VecModel::from(stage_items)));
    }

    // 2. Refresh Grades
    if let Ok(grades) = db.get_config_items("grade") {
        let grade_items: Vec<ConfigItem> = grades.into_iter().map(|(id, name)| {
            ConfigItem { 
                id: id as i32, 
                name: SharedString::from(name) 
            }
        }).collect();
        ui.set_grades_list(ModelRc::new(VecModel::from(grade_items)));
    }
}

/// Updates the dashboard summary for a specific stage
fn refresh_stage_summary(ui: &MainWindow, db: &Database, stage_id: i32) {
    if let Ok((total_w, count)) = db.aggregate_stage_totals(stage_id as i64) {
        ui.set_stage_total_weight(total_w as f32);
        ui.set_stage_batch_count(count as i32);
    }
}

// --- Main Execution ---

fn main() -> Result<(), slint::PlatformError> {
    let ui = MainWindow::new()?;
    let db = Database::open("urt_ledger.db").expect("Failed to initialize SQLite");
    let model = Model::new(db);

    // Initial Data Load (Populate the lists on startup)
    refresh_config_lists(&ui, &model.db);

    // --- Callback: Add New Stage ---
    ui.on_add_new_stage({
        let ui_handle = ui.as_weak();
        let db_clone = model.db.clone();
        move |name: SharedString| {
            let name_str = name.to_string();
            if name_str.trim().is_empty() { return; }
            
            // Insert into Configuration table with kind="stage"
            let _ = db_clone.insert_config("stage", &name_str);
            
            // Refresh UI to show the new item immediately
            if let Some(ui) = ui_handle.upgrade() {
                refresh_config_lists(&ui, &db_clone);
            }
        }
    });

    // --- Callback: Add New Grade ---
    ui.on_add_new_grade({
        let ui_handle = ui.as_weak();
        let db_clone = model.db.clone();
        move |name: SharedString| {
            let name_str = name.to_string();
            if name_str.trim().is_empty() { return; }

            // Insert into Configuration table with kind="grade"
            let _ = db_clone.insert_config("grade", &name_str);

            if let Some(ui) = ui_handle.upgrade() {
                refresh_config_lists(&ui, &db_clone);
            }
        }
    });

    // --- Callback: Add Batch ---
    ui.on_add_batch({
        let ui_handle = ui.as_weak();
        let db_clone = model.db.clone();
        move |name, weight_str, price_str, grade_id, stage_id| {
            let weight = weight_str.parse::<f64>().unwrap_or(0.0);
            let price = price_str.parse::<f64>().unwrap_or(0.0);

            let new_batch = InventoryBatch {
                name: name.to_string(),
                weight,
                price,
                grade_id: grade_id as i64,
                stage_id: stage_id as i64,
                type_id: 1, 
                specs: HashMap::new(),
            };

            if let Ok(_) = db_clone.insert_inventory_batch(&new_batch) {
                if let Some(ui) = ui_handle.upgrade() {
                    refresh_stage_summary(&ui, &db_clone, stage_id);
                }
            } else {
                eprintln!("Failed to insert batch.");
            }
        }
    });

    // --- Callback: Request Summary ---
    ui.on_request_stage_summary({
        let ui_handle = ui.as_weak();
        let db_clone = model.db.clone();
        move |stage_id| {
            if let Some(ui) = ui_handle.upgrade() {
                refresh_stage_summary(&ui, &db_clone, stage_id);
            }
        }
    });

    ui.run()
}