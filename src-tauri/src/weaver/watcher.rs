use notify_debouncer_mini::{new_debouncer, notify::RecursiveMode, DebounceEventResult};
use std::path::{Path, PathBuf};
use std::sync::{mpsc, Arc, Mutex};
use std::time::Duration;
use tauri::{AppHandle, Emitter, Manager};
use serde::Serialize;

#[derive(Clone, Serialize)]
pub struct FileChangeEvent {
    pub path: String,
    pub ring_expanded: Option<u32>,
}

pub struct WatcherController {
    tx_switch: Mutex<mpsc::Sender<PathBuf>>,
}

impl WatcherController {
    pub fn watch_directory(&self, new_path: PathBuf) {
        if let Ok(sender) = self.tx_switch.lock() {
            let _ = sender.send(new_path);
        }
    }
}

pub type SharedWatcher = Arc<WatcherController>;

pub fn spawn_watcher(app_handle: AppHandle, watch_path: impl AsRef<Path>) -> SharedWatcher {
    let (tx_events, rx_events) = mpsc::channel::<DebounceEventResult>();
    let (tx_switch, rx_switch) = mpsc::channel::<PathBuf>();
    let initial_path = watch_path.as_ref().to_path_buf();

    std::thread::spawn(move || {
        let mut debouncer = match new_debouncer(Duration::from_millis(500), tx_events) {
            Ok(d) => d,
            Err(e) => {
                eprintln!("Watcher debouncer init error: {:?}", e);
                return;
            }
        };

        let mut current_watched = initial_path.clone();
        let _ = debouncer
            .watcher()
            .watch(&current_watched, RecursiveMode::Recursive);

        let valid_exts = ["rs", "ts", "tsx", "js", "jsx", "py", "go", "astro"];

        loop {
            // Check if a project switch was requested
            while let Ok(new_dir) = rx_switch.try_recv() {
                if new_dir.exists() && new_dir != current_watched {
                    let _ = debouncer.watcher().unwatch(&current_watched);
                    if debouncer
                        .watcher()
                        .watch(&new_dir, RecursiveMode::Recursive)
                        .is_ok()
                    {
                        current_watched = new_dir;
                    }
                }
            }

            // Wait for file events with timeout so we can check rx_switch periodically
            match rx_events.recv_timeout(Duration::from_millis(400)) {
                Ok(Ok(events)) => {
                    for event in events {
                        let path_buf = event.path;
                        let path_str = path_buf.to_string_lossy().to_string();

                        // Ignore internal build/db directories
                        if path_str.contains(".mandala")
                            || path_str.contains("node_modules")
                            || path_str.contains("target")
                            || path_str.contains(".git")
                            || path_str.contains("dist")
                        {
                            continue;
                        }

                        let is_watched_ext = path_buf
                            .extension()
                            .and_then(|e| e.to_str())
                            .map(|ext| valid_exts.contains(&ext))
                            .unwrap_or(false);

                        if is_watched_ext && path_buf.is_file() {
                            let handle_clone = app_handle.clone();
                            let path_clone = path_str.clone();

                            tauri::async_runtime::spawn(async move {
                                let mut ring_expanded = None;
                                if let Some(db_state) =
                                    handle_clone.try_state::<crate::persistence::surreal_bridge::SharedDb>()
                                {
                                    let db_guard = db_state.read().await;
                                    if let Ok(ring) =
                                        crate::weaver::sync_file_in_place(&*db_guard, &path_clone).await
                                    {
                                        ring_expanded = Some(ring);
                                    }
                                }

                                let payload = FileChangeEvent {
                                    path: path_clone,
                                    ring_expanded,
                                };
                                let _ = handle_clone.emit("mandala:file-changed", payload);
                            });
                        }
                    }
                }
                Ok(Err(e)) => {
                    eprintln!("Watcher error: {:?}", e);
                }
                Err(mpsc::RecvTimeoutError::Timeout) => {}
                Err(mpsc::RecvTimeoutError::Disconnected) => break,
            }
        }
    });

    Arc::new(WatcherController {
        tx_switch: Mutex::new(tx_switch),
    })
}
