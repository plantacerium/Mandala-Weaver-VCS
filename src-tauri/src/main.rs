// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod interface;
mod persistence;
mod geometry;
mod ontology;
mod synarchy;
mod weaver;

use persistence::surreal_bridge::connect_embedded;
use interface::projection_api::{
    export_mandala_state, 
    expand_ring,
    init_project,
    distill_from_selection,
    trace_monad_lineage,
    get_monad_detail,
};
use interface::cli_api::{
    cli_bindu,
    cli_telemetry,
    cli_status,
    cli_crystallize,
    cli_distill,
    cli_lineage,
    cli_spectrum,
    cli_inspect,
    cli_echo,
    cli_vector,
    cli_focus,
    cli_dormant,
    cli_synthesize,
    cli_absorb,
    cli_emanate,
};
use interface::synarchy_api::{
    get_projects,
    add_project,
    remove_project,
    get_project_detail,
    rescan_project,
    get_project_mandala,
    SynarchyState,
};
use tauri::Manager;
use std::path::PathBuf;

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
            let handle = app.handle().clone();
            
            let db = tauri::async_runtime::block_on(async {
                let db_conn = connect_embedded().await.expect("Failed to initialize SurrealDB");
                let queries = crate::persistence::schemas::get_initialization_queries();
                for q in queries {
                    let _ = db_conn.query(q).await;
                }
                db_conn
            });
            handle.manage(db);

            let synarchy_state = SynarchyState::new(
                app.path().app_data_dir().unwrap_or_else(|_| PathBuf::from("."))
            );
            handle.manage(synarchy_state);

            let cwd = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
            crate::weaver::watcher::spawn_watcher(handle, cwd);

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            export_mandala_state, 
            expand_ring,
            init_project,
            distill_from_selection,
            trace_monad_lineage,
            get_monad_detail,
            cli_bindu,
            cli_telemetry,
            cli_status,
            cli_crystallize,
            cli_distill,
            cli_lineage,
            cli_spectrum,
            cli_inspect,
            cli_echo,
            cli_vector,
            cli_focus,
            cli_dormant,
            cli_synthesize,
            cli_absorb,
            cli_emanate,
            get_projects,
            add_project,
            remove_project,
            get_project_detail,
            rescan_project,
            get_project_mandala,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}