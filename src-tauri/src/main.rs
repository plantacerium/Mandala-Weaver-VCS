// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![allow(dead_code)]

mod interface;
mod persistence;
mod geometry;
mod ontology;
mod synarchy;
mod weaver;
mod language;
mod collaboration;
mod plugins;
mod template;
mod network;

use persistence::surreal_bridge::connect_embedded;
use interface::projection_api::{
    export_mandala_state, 
    expand_ring,
    init_project,
    open_project,
    distill_from_selection,
    trace_monad_lineage,
    get_monad_detail,
    write_distilled_to_disk,
    contract_outer_ring,
    revert_to_level,
    get_archived_monads_count,
    purge_archived_monads,
    search_monads,
    get_monads_paginated_api,
};
use interface::collaboration_api::{
    export_mandala_archive,
    import_mandala_archive,
    import_git_repository,
    diff_mandala_states,
    merge_mandala_states,
    sync_project,
    get_sync_status,
    remove_sync_project,
    stop_sync,
    set_sync_status,
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
    cli_seed,
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
use interface::template_api::{
    distill_from_template,
    distill_from_template_content,
    validate_template,
    validate_template_content,
    create_template,
    distill_and_write,
    list_templates,
};
use tauri::Manager;
use std::path::PathBuf;

fn main() {
    // CLI Integration
    let args: Vec<String> = std::env::args().collect();
    // If there are more than 1 argument and it's not a Tauri internal flag
    if args.len() > 1 && !args[1].starts_with("--") {
        if let Err(e) = tauri::async_runtime::block_on(crate::interface::cli_commands::run_cli()) {
            eprintln!("❌ CLI Error: {}", e);
            std::process::exit(1);
        }
        return;
    }

    // Initialize Synchronizer (Background background task)
    let synchronizer = crate::synarchy::sync::Synchronizer::new();
    let sync_handle = synchronizer.clone();
    tauri::async_runtime::spawn(async move {
        sync_handle.start().await;
    });

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
            let handle = app.handle().clone();
            
            let db_path = crate::persistence::surreal_bridge::find_project_root();
            let db = tauri::async_runtime::block_on(async {
                let db_conn = if let Some(ref path) = db_path {
                    crate::persistence::surreal_bridge::connect_db(Some(path))
                        .await
                        .unwrap_or_else(|e| {
                            eprintln!("⚠ Failed to connect with project root ({}): falling back to embedded", e);
                            tauri::async_runtime::block_on(connect_embedded()).expect("Failed to initialize SurrealDB")
                        })
                } else {
                    connect_embedded().await.expect("Failed to initialize SurrealDB")
                };
                let queries = crate::persistence::schemas::get_initialization_queries();
                for q in queries {
                    let _ = db_conn.query(q).await;
                }
                db_conn
            });
            handle.manage(std::sync::Arc::new(tokio::sync::RwLock::new(db.clone())));

            let active_project_path: crate::persistence::surreal_bridge::ActiveProjectState = 
                std::sync::Arc::new(tokio::sync::RwLock::new(db_path.clone()));
            handle.manage(active_project_path);

            let synarchy_state = SynarchyState::new(
                app.path().app_data_dir().unwrap_or_else(|_| PathBuf::from("."))
            );

            // Auto-register current project if found and not yet in registry
            if let Some(ref root) = db_path {
                let root_clone = root.clone();
                let synarchy_clone = synarchy_state.registry.clone();
                let db_ref = db.clone();
                tauri::async_runtime::block_on(async {
                    let mut reg = synarchy_clone.write().await;
                    let exists = reg.projects.iter().any(|p| {
                        p.path.canonicalize().unwrap_or(p.path.clone()) == root_clone.canonicalize().unwrap_or(root_clone.clone())
                    });
                    if !exists {
                        if let Ok(entry) = crate::synarchy::ProjectScanner::scan_and_index_with_db(&db_ref, &root_clone).await {
                            reg.add(entry);
                            drop(reg);
                            let _ = synarchy_state.save().await;
                        }
                    }
                });
            }

            handle.manage(synarchy_state);
            handle.manage(synchronizer);

            let plugin_registry = crate::plugins::PluginRegistry::init();
            handle.manage(plugin_registry);

            let cwd = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
            crate::weaver::watcher::spawn_watcher(handle, cwd);

            // Integration: Use ProjectChangeEvent
            let _demo_event = crate::synarchy::sync::ProjectChangeEvent {
                project_id: "test".to_string(),
                project_name: "test".to_string(),
                change_type: crate::synarchy::sync::ChangeType::Modified,
                monads_added: 0,
                rings_changed: vec![],
            };
            
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            export_mandala_state, 
            expand_ring,
            init_project,
            open_project,
            distill_from_selection,
            trace_monad_lineage,
            get_monad_detail,
            write_distilled_to_disk,
            contract_outer_ring,
            revert_to_level,
            get_archived_monads_count,
            purge_archived_monads,
            search_monads,
            get_monads_paginated_api,
            export_mandala_archive,
            import_mandala_archive,
            import_git_repository,
            diff_mandala_states,
            merge_mandala_states,
            sync_project,
            get_sync_status,
            remove_sync_project,
            stop_sync,
            set_sync_status,
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
            cli_seed,
            get_projects,
            add_project,
            remove_project,
            get_project_detail,
            rescan_project,
            get_project_mandala,
            distill_from_template,
            distill_from_template_content,
            validate_template,
            validate_template_content,
            create_template,
            distill_and_write,
            list_templates,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}