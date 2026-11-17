mod db;
mod models;
mod organizer;
mod scanner;
mod watcher;

use crate::models::{BatchInfo, OrganizeResult, ScanResult, Settings, UndoResult};
use crate::watcher::WatcherManager;
use rusqlite::Connection;
use std::path::{Path, PathBuf};
use tauri::image::Image;
use tauri::menu::{Menu, MenuItem, PredefinedMenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{AppHandle, Emitter, Manager, State};

/// The user's Downloads folder as reported by the operating system.
#[tauri::command]
fn default_downloads_dir() -> Result<String, String> {
    scanner::default_downloads_dir()
        .map(|path| path.to_string_lossy().into_owned())
        .ok_or_else(|| "Could not determine the Downloads directory for this user.".to_string())
}

/// Scan the top level of a folder and produce a dry-run preview.
#[tauri::command]
fn scan_folder(folder: String) -> Result<ScanResult, String> {
    scanner::scan_folder(Path::new(&folder))
}

/// Move the selected files into category folders and record undo history.
#[tauri::command]
fn organize_selected(
    app: tauri::AppHandle,
    paths: Vec<String>,
    target_folder: String,
) -> Result<OrganizeResult, String> {
    let data_dir = app
        .path()
        .app_data_dir()
        .map_err(|error| format!("Unable to locate app data directory: {}", error))?;
    let mut connection = db::open_db(&data_dir)?;
    organizer::organize_selected(&paths, Path::new(&target_folder), &mut connection)
}

/// Undo the most recent sort batch.
#[tauri::command]
fn undo_last_batch(app: tauri::AppHandle) -> Result<UndoResult, String> {
    let data_dir = app
        .path()
        .app_data_dir()
        .map_err(|error| format!("Unable to locate app data directory: {}", error))?;
    let mut connection = db::open_db(&data_dir)?;
    organizer::undo_last_batch(&mut connection)
}

/// Metadata for the Undo button (how many files the last batch contained).
#[tauri::command]
fn get_last_batch_info(app: tauri::AppHandle) -> Result<Option<BatchInfo>, String> {
    let data_dir = app
        .path()
        .app_data_dir()
        .map_err(|error| format!("Unable to locate app data directory: {}", error))?;
    let connection = db::open_db(&data_dir)?;
    organizer::last_batch_info(&connection)
}

#[tauri::command]
fn get_settings(app: tauri::AppHandle) -> Result<Settings, String> {
    let data_dir = app
        .path()
        .app_data_dir()
        .map_err(|error| format!("Unable to locate app data directory: {}", error))?;
    let connection = db::open_db(&data_dir)?;
    db::get_settings(&connection)
}

/// Shared by the frontend command and the tray menu so both stay in sync.
fn apply_auto_organize(
    app: &AppHandle,
    manager: &WatcherManager,
    connection: &mut Connection,
    enabled: bool,
    folder: &Path,
) -> Result<Settings, String> {
    db::set_auto_organize(connection, enabled)?;
    db::set_target_folder(connection, &folder.to_string_lossy())?;
    if enabled {
        let settings = db::get_settings(connection)?;
        manager.start(app.clone(), folder.to_path_buf(), settings)?;
    } else {
        manager.stop()?;
    }

    let settings = db::get_settings(connection)?;
    app.emit("droptidy-settings-changed", settings.clone())
        .map_err(|error| error.to_string())?;
    Ok(settings)
}

#[tauri::command]
fn set_auto_organize(
    app: tauri::AppHandle,
    manager: State<'_, WatcherManager>,
    enabled: bool,
    folder: String,
) -> Result<Settings, String> {
    let data_dir = app
        .path()
        .app_data_dir()
        .map_err(|error| format!("Unable to locate app data directory: {}", error))?;
    let mut connection = db::open_db(&data_dir)?;
    apply_auto_organize(&app, &manager, &mut connection, enabled, Path::new(&folder))
}

/// Persist the debounce delay and restart the watcher if it is active.
#[tauri::command]
fn set_watch_delay_secs(
    app: tauri::AppHandle,
    manager: State<'_, WatcherManager>,
    seconds: f64,
    folder: String,
) -> Result<Settings, String> {
    let data_dir = app
        .path()
        .app_data_dir()
        .map_err(|error| format!("Unable to locate app data directory: {}", error))?;
    let connection = db::open_db(&data_dir)?;
    db::set_watch_delay_secs(&connection, seconds)?;

    let settings = db::get_settings(&connection)?;
    if settings.auto_organize {
        manager.start(app.clone(), PathBuf::from(&folder), settings.clone())?;
    }
    app.emit("droptidy-settings-changed", settings.clone())
        .map_err(|error| error.to_string())?;
    Ok(settings)
}

fn setup_tray(app: &mut tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    let open_item = MenuItem::with_id(app, "open", "Open DropTidy", true, None::<&str>)?;
    let toggle_item = MenuItem::with_id(
        app,
        "toggle_auto",
        "Toggle Auto-Organize",
        true,
        None::<&str>,
    )?;
    let separator = PredefinedMenuItem::separator(app)?;
    let quit_item = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&open_item, &toggle_item, &separator, &quit_item])?;

    let icon = Image::from_bytes(include_bytes!("../icons/32x32.png"))?;
    let tray = TrayIconBuilder::new()
        .icon(icon)
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| match event.id().as_ref() {
            "open" => {
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.show();
                    let _ = window.set_focus();
                }
            }
            "toggle_auto" => {
                let data_dir = match app.path().app_data_dir() {
                    Ok(path) => path,
                    Err(_) => return,
                };
                let mut connection = match db::open_db(&data_dir) {
                    Ok(connection) => connection,
                    Err(_) => return,
                };
                let settings = match db::get_settings(&connection) {
                    Ok(settings) => settings,
                    Err(_) => return,
                };
                // Follow the same folder the UI was using when possible.
                let folder = db::get_target_folder(&connection)
                    .map(PathBuf::from)
                    .or_else(scanner::default_downloads_dir)
                    .unwrap_or_default();
                let manager = app.state::<WatcherManager>();
                let _ = apply_auto_organize(
                    app,
                    &manager,
                    &mut connection,
                    !settings.auto_organize,
                    &folder,
                );
            }
            "quit" => app.exit(0),
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                let app = tray.app_handle();
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.show();
                    let _ = window.set_focus();
                }
            }
        })
        .build(app)?;

    // Keep the tray icon alive for the lifetime of the app.
    app.manage(tray);

    // Restore a persisted Auto-Organize session at launch.
    let data_dir = app.path().app_data_dir()?;
    let connection = db::open_db(&data_dir)?;
    let settings = db::get_settings(&connection)?;
    if settings.auto_organize {
        let folder = db::get_target_folder(&connection)
            .map(PathBuf::from)
            .or_else(scanner::default_downloads_dir);
        if let Some(folder) = folder {
            app.state::<WatcherManager>()
                .start(app.handle().clone(), folder, settings)?;
        }
    }

    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .manage(WatcherManager::default())
        .setup(|app| {
            setup_tray(app)?;
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            default_downloads_dir,
            scan_folder,
            organize_selected,
            undo_last_batch,
            get_last_batch_info,
            get_settings,
            set_auto_organize,
            set_watch_delay_secs,
        ])
        .run(tauri::generate_context!())
        .expect("error while running DropTidy");
}
