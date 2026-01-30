pub mod db;
pub mod model;

use slint::{ComponentHandle, SharedString, ModelRc, VecModel};
use crate::db::Database;
use crate::model::{InventoryBatch, Model};
use std::collections::HashMap;

slint::include_modules!();

// Helper to convert Rust vectors to Slint models
fn refresh_config_lists(ui: &MainWindow, db: &Database) {
    // 1. Refresh Stages
    if let Ok(stages) = db.get_all_stages() {
        let stage_items: Vec<ConfigItem> = stages.into_iter().map(|(id, name)| {
            ConfigItem { id: id as i32, name: name.into() }
        }).collect();
        ui.set_stages_list(ModelRc::new(VecModel::from(stage_items)));
    }

    // 2. Refresh Grades
    if let Ok(grades) = db.get_all_grades() {
        let grade_items: Vec<ConfigItem> = grades.into_iter().map(|(id, name)| {
            ConfigItem { id: id as i32, name: name.into() }
        }).collect();
        ui.set_grades_list(ModelRc::new(VecModel::from(grade_items)));
    }
}

fn main() -> Result<(), slint::PlatformError> {
    let ui = MainWindow::new()?;
    let db = Database::open("urt_ledger.db").expect("Failed to initialize SQLite");
    let model = Model::new(db);

    // Initial Data Load
    refresh_config_lists(&ui, &model.db);

    // --- Callback: Add New Stage ---
    ui.on_add_new_stage({
        let ui_handle = ui.as_weak();
        let db_clone = model.db.clone();
        move |name: SharedString| {
            if name.is_empty() { return; }
            let _ = db_clone.insert_stage(name.to_string());
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
            if name.is_empty() { return; }
            let _ = db_clone.insert_grade(name.to_string());
            if let Some(ui) = ui_handle.upgrade() {
                refresh_config_lists(&ui, &db_clone);
            }
        }
    });

    // --- Callback: Add Batch ---
    ui.on_add_batch({
        let ui_handle = ui.as_weak();
        let db_clone = model.db.clone();
        move |name: SharedString, weight_str: SharedString, price_str: SharedString, grade_id: i32, stage_id: i32| {
            let weight = weight_str.to_string().parse::<f64>().unwrap_or(0.0);
            let price = price_str.to_string().parse::<f64>().unwrap_or(0.0);

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
                    // Update the summary for the current stage
                    if let Ok((total_w, count)) = db_clone.aggregate_stage_totals(stage_id as i64) {
                        ui.set_stage_total_weight(total_w as f32);
                        ui.set_stage_batch_count(count as i32);
                    }
                }
            }
        }
    });

    // --- Callback: Request Summary ---
    ui.on_request_stage_summary({
        let ui_handle = ui.as_weak();
        let db_clone = model.db.clone();
        move |stage_id: i32| {
            if let Some(ui) = ui_handle.upgrade() {
                if let Ok((total_w, count)) = db_clone.aggregate_stage_totals(stage_id as i64) {
                    ui.set_stage_total_weight(total_w as f32);
                    ui.set_stage_batch_count(count as i32);
                }
            }
        }
    });

    ui.run()
}
