use crate::{db, models::Settings, organizer, scanner};
use notify::{Config, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::collections::HashMap;
use std::fs::OpenOptions;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, Receiver, RecvTimeoutError};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tauri::{AppHandle, Emitter, Manager};

/// Give up on a stubborn file after ~12 seconds of failed attempts so the
/// pending map cannot grow forever when a file stays locked by another app.
const MAX_ATTEMPTS: u32 = 60;

struct WatcherRuntime {
    stop: Arc<AtomicBool>,
}

/// Tauri-managed lifecycle for the single background watcher thread.
#[derive(Default)]
pub struct WatcherManager {
    runtime: Mutex<Option<WatcherRuntime>>,
}

impl WatcherManager {
    pub fn stop(&self) -> Result<(), String> {
        let mut guard = self
            .runtime
            .lock()
            .map_err(|_| "Watcher state lock is poisoned.".to_string())?;
        if let Some(runtime) = guard.take() {
            runtime.stop.store(true, Ordering::Relaxed);
        }
        Ok(())
    }

    /// Start (or restart) watching `folder`. Events are debounced by the
    /// persisted `watch_delay_secs` before a file is auto-organized.
    pub fn start(
        &self,
        app: AppHandle,
        folder: PathBuf,
        settings: Settings,
    ) -> Result<(), String> {
        self.stop()?;

        let (sender, receiver) = mpsc::channel::<Event>();
        let mut watcher = RecommendedWatcher::new(
            move |result: notify::Result<Event>| {
                if let Ok(event) = result {
                    let _ = sender.send(event);
                }
            },
            Config::default(),
        )
        .map_err(|error| format!("Unable to create file watcher: {}", error))?;

        watcher
            .watch(&folder, RecursiveMode::NonRecursive)
            .map_err(|error| format!("Unable to watch '{}': {}", folder.display(), error))?;

        let stop = Arc::new(AtomicBool::new(false));
        let thread_stop = stop.clone();

        std::thread::Builder::new()
            .name("droptidy-watcher".to_string())
            .spawn(move || {
                // The native watcher must stay alive on this thread.
                let _watcher = watcher;
                watch_loop(app, folder, settings.watch_delay_secs, receiver, thread_stop);
            })
            .map_err(|error| format!("Unable to start watcher thread: {}", error))?;

        let mut guard = self
            .runtime
            .lock()
            .map_err(|_| "Watcher state lock is poisoned.".to_string())?;
        *guard = Some(WatcherRuntime { stop });
        Ok(())
    }
}

struct Pending {
    last_event: Instant,
    attempts: u32,
}

fn watch_loop(
    app: AppHandle,
    folder: PathBuf,
    delay_secs: f64,
    receiver: Receiver<Event>,
    stop: Arc<AtomicBool>,
) {
    let data_dir = match app.path().app_data_dir() {
        Ok(path) => path,
        Err(error) => {
            eprintln!("DropTidy watcher: app data dir unavailable: {}", error);
            return;
        }
    };
    let mut connection = match db::open_db(&data_dir) {
        Ok(connection) => connection,
        Err(error) => {
            eprintln!("DropTidy watcher: database unavailable: {}", error);
            return;
        }
    };

    let delay = Duration::from_millis((delay_secs * 1000.0).round().max(100.0) as u64);
    let mut pending: HashMap<PathBuf, Pending> = HashMap::new();

    loop {
        if stop.load(Ordering::Relaxed) {
            break;
        }

        match receiver.recv_timeout(Duration::from_millis(200)) {
            Ok(event) => {
                for path in candidate_paths(&event) {
                    if is_watched_file(&path, &folder) {
                        pending.insert(
                            path,
                            Pending {
                                last_event: Instant::now(),
                                attempts: 0,
                            },
                        );
                    }
                }
            }
            Err(RecvTimeoutError::Timeout) => {}
            Err(RecvTimeoutError::Disconnected) => break,
        }

        let now = Instant::now();
        let ready: Vec<PathBuf> = pending
            .iter()
            .filter(|(_, entry)| {
                now.duration_since(entry.last_event) >= delay && entry.attempts < MAX_ATTEMPTS
            })
            .map(|(path, _)| path.clone())
            .collect();

        for path in ready {
            if !is_watched_file(&path, &folder) {
                pending.remove(&path);
                continue;
            }

            if is_locked(&path) {
                if let Some(entry) = pending.get_mut(&path) {
                    entry.attempts += 1;
                }
                continue;
            }

            match organizer::organize_single(&path, &folder, &mut connection) {
                Ok(()) => {
                    let name = path
                        .file_name()
                        .unwrap_or_default()
                        .to_string_lossy()
                        .into_owned();
                    let _ = app.emit("droptidy-auto-sorted", name);
                }
                Err(error) => {
                    eprintln!("DropTidy watcher: could not sort '{}': {}", path.display(), error);
                    if let Some(entry) = pending.get_mut(&path) {
                        entry.attempts += 1;
                    }
                }
            }
            pending.remove(&path);
        }

        pending.retain(|_, entry| entry.attempts < MAX_ATTEMPTS);
    }
}

/// Only creations, renames and content writes are interesting. A rename into
/// the folder arrives as a `Modify` / `Create` event on most platforms.
fn candidate_paths(event: &Event) -> Vec<PathBuf> {
    match event.kind {
        EventKind::Create(_) | EventKind::Modify(_) => event.paths.clone(),
        _ => Vec::new(),
    }
}

/// Restrict watcher activity to plain files directly inside the target folder.
fn is_watched_file(path: &Path, folder: &Path) -> bool {
    if !path.is_file() || path.parent() != Some(folder) {
        return false;
    }
    match path.file_name() {
        Some(name) => !scanner::is_ignored_file_name(&name.to_string_lossy()),
        None => false,
    }
}

/// A file is considered safe to move once it can be opened for writing.
/// Read-only files are still safe, so fall back to a read-only open.
fn is_locked(path: &Path) -> bool {
    if OpenOptions::new().read(true).write(true).open(path).is_ok() {
        return false;
    }
    OpenOptions::new().read(true).open(path).is_err()
}
