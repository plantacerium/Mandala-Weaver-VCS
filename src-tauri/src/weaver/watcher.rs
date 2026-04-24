use notify_debouncer_mini::{new_debouncer, notify::RecursiveMode, DebounceEventResult};
use std::path::Path;
use std::time::Duration;
use tauri::{AppHandle, Emitter};
use serde::Serialize;
use std::sync::mpsc;

#[derive(Clone, Serialize)]
pub struct FileChangeEvent {
    pub path: String,
}

pub fn spawn_watcher(app_handle: AppHandle, watch_path: impl AsRef<Path>) {
    let (tx, rx) = mpsc::channel::<DebounceEventResult>();
    let watch_path = watch_path.as_ref().to_path_buf();

    std::thread::spawn(move || {
        let mut debouncer = new_debouncer(Duration::from_millis(500), tx).expect("Failed to create debouncer");
        
        debouncer
            .watcher()
            .watch(&watch_path, RecursiveMode::Recursive)
            .expect("Failed to watch path");

        // Relay events to Tauri async runtime
        for res in rx {
            match res {
                Ok(events) => {
                    for event in events {
                        let path_str = event.path.to_string_lossy().to_string();
                        if path_str.ends_with(".rs") || path_str.ends_with(".ts") || path_str.ends_with(".js") || path_str.ends_with(".astro") {
                            let _ = app_handle.emit("mandala://file-changed", FileChangeEvent {
                                path: path_str,
                            });
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Watcher error: {:?}", e);
                }
            }
        }
    });
}
