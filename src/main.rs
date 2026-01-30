use slint::{ComponentHandle, SharedString, ModelRc, VecModel, Model as SlintModel}; 
use crate::db::Database;
// We alias your 'Model' struct to 'AppModel' to avoid conflict with Slint's 'Model' trait
use crate::model::{InventoryBatch, Model as AppModel}; 
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

pub mod db;
pub mod model;

slint::include_modules!();

// Cache to manage IDs and Spec Keys
struct AppCache {
    stages: Vec<i64>,
    grades: Vec<i64>,
    types: Vec<(i64, Vec<String>)>, 
}

fn refresh_lists(ui: &MainWindow, db: &Database, cache: &Arc<Mutex<AppCache>>) {
    let mut cache = cache.lock().unwrap();

    // 1. Stages
    if let Ok(items) = db.get_all_stages() {
        cache.stages = items.iter().map(|(id, _)| *id).collect();
        let ui_items: Vec<ConfigItem> = items.iter().map(|(id, name)| ConfigItem { 
            id: *id as i32, name: SharedString::from(name), display_text: "".into() 
        }).collect();
        let names: Vec<SharedString> = items.iter().map(|(_, name)| SharedString::from(name)).collect();

        ui.set_stages_list(ModelRc::new(VecModel::from(ui_items)));
        ui.set_stage_names(ModelRc::new(VecModel::from(names)));
    }

    // 2. Grades
    if let Ok(items) = db.get_all_grades() {
        cache.grades = items.iter().map(|(id, _)| *id).collect();
        let ui_items: Vec<ConfigItem> = items.iter().map(|(id, name)| ConfigItem { 
            id: *id as i32, name: SharedString::from(name), display_text: "".into() 
        }).collect();
        let names: Vec<SharedString> = items.iter().map(|(_, name)| SharedString::from(name)).collect();

        ui.set_grades_list(ModelRc::new(VecModel::from(ui_items)));
        ui.set_grade_names(ModelRc::new(VecModel::from(names)));
    }

    // 3. Product Types
    if let Ok(items) = db.get_all_product_types() {
        cache.types = items.iter().map(|(id, _, keys)| (*id, keys.clone())).collect();
        
        let ui_items: Vec<ConfigItem> = items.iter().map(|(id, name, keys)| {
            let spec_str = keys.join(", ");
            let display = format!("{} [{}]", name, spec_str);
            ConfigItem { 
                id: *id as i32, 
                name: name.clone().into(), 
                display_text: display.into() 
            }
        }).collect();
        ui.set_types_list(ModelRc::new(VecModel::from(ui_items)));

        let names: Vec<SharedString> = items.into_iter().map(|(_, name, _)| name.into()).collect();
        ui.set_type_names(ModelRc::new(VecModel::from(names)));
    }
}

fn main() -> Result<(), slint::PlatformError> {
    let ui = MainWindow::new()?;
    let db = Database::open("urt_ledger.db").expect("Failed to initialize SQLite");
    
    // Seed defaults (Unbucked, Bucked, A, B, C...)
    let _ = db.seed_defaults();

    let model = AppModel::new(db);
    let cache = Arc::new(Mutex::new(AppCache { stages: vec![], grades: vec![], types: vec![] }));

    refresh_lists(&ui, &model.db, &cache);

    // --- CONFIG CALLBACKS ---
    ui.on_add_new_stage({
        let ui_handle = ui.as_weak();
        let db = model.db.clone();
        let c = cache.clone();
        move |name| {
            if !name.is_empty() { let _ = db.insert_stage(name.to_string()); }
            if let Some(ui) = ui_handle.upgrade() { refresh_lists(&ui, &db, &c); }
        }
    });

    ui.on_add_new_grade({
        let ui_handle = ui.as_weak();
        let db = model.db.clone();
        let c = cache.clone();
        move |name| {
            if !name.is_empty() { let _ = db.insert_grade(name.to_string()); }
            if let Some(ui) = ui_handle.upgrade() { refresh_lists(&ui, &db, &c); }
        }
    });

    ui.on_add_new_type({
        let ui_handle = ui.as_weak();
        let db = model.db.clone();
        let c = cache.clone();
        move |name, specs_str| {
            if !name.is_empty() {
                let keys: Vec<String> = specs_str.split(',')
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect();
                let _ = db.insert_product_type(name.to_string(), keys);
            }
            if let Some(ui) = ui_handle.upgrade() { refresh_lists(&ui, &db, &c); }
        }
    });

    ui.on_type_selected({
        let ui_handle = ui.as_weak();
        let c = cache.clone();
        move |index| {
            let cache = c.lock().unwrap();
            if let Some((_, keys)) = cache.types.get(index as usize) {
                let fields: Vec<SpecField> = keys.iter().map(|k| SpecField { 
                    name: k.into(), 
                    value: "".into() 
                }).collect();
                if let Some(ui) = ui_handle.upgrade() {
                    ui.set_active_specs(ModelRc::new(VecModel::from(fields)));
                }
            }
        }
    });

    ui.on_add_batch({
        let ui_handle = ui.as_weak();
        let db = model.db.clone();
        let c = cache.clone();
        move |type_idx, name, w_str, p_str, grade_idx, stage_idx, spec_values| {
            let cache = c.lock().unwrap();
            
            let type_data = cache.types.get(type_idx as usize);
            let type_id = type_data.map(|(id, _)| *id).unwrap_or(0);
            
            // Explicitly type the HashMap to fix inference error
            let mut specs_map: HashMap<String, f64> = HashMap::new();
            
            if let Some((_, keys)) = type_data {
                // Now .iter() works because we imported SlintModel
                for (i, field_struct) in spec_values.iter().enumerate() {
                    if let Some(key) = keys.get(i) {
                         let v = field_struct.value.parse::<f64>().unwrap_or(0.0);
                         specs_map.insert(key.clone(), v);
                    }
                }
            }

            // Safe lookup for Grade/Stage
            let grade_id = if cache.grades.len() > 0 {
                *cache.grades.get(grade_idx as usize).unwrap_or(&0)
            } else { 0 };

            let stage_id = if cache.stages.len() > 0 {
                *cache.stages.get(stage_idx as usize).unwrap_or(&0)
            } else { 0 };

            let new_batch = InventoryBatch {
                name: name.to_string(),
                weight: w_str.parse().unwrap_or(0.0),
                price: p_str.parse().unwrap_or(0.0),
                grade_id,
                stage_id,
                type_id,
                specs: specs_map,
            };

            if let Ok(_) = db.insert_inventory_batch(&new_batch) {
                if let Some(ui) = ui_handle.upgrade() {
                    if let Ok((w, count)) = db.aggregate_stage_totals(new_batch.stage_id) {
                         ui.set_stage_total_weight(w as f32);
                         ui.set_stage_batch_count(count as i32);
                    }
                }
            }
        }
    });

    ui.run()
}