// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod interface;
mod persistence;
mod geometry;
mod ontology;
mod weaver;

use persistence::surreal_bridge::{connect_embedded, Db};
use interface::projection_api::{export_mandala_state, expand_ring};
use surrealdb::Surreal;
use tauri::Manager;

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![
            export_mandala_state,
            expand_ring
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
