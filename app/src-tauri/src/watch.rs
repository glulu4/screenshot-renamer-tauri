// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
use std::{
    fs,
    path::Path,
    sync::{Arc, Mutex},
};
use tauri::{AppHandle, Emitter, Manager};
// use tauri::tray::{TrayIconEvent, MouseButton, MouseButtonState};

#[cfg(target_os = "macos")]
use dotenvy::dotenv;
use notify::{recommended_watcher, Event, RecursiveMode, Result, Watcher};
use std::collections::HashMap;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;
use std::time::Instant;

use crate::{generate_name::generate_screenshot_name, user::UserDevice};

pub fn is_new_screenshot(path: &Path) -> bool {
    if path.is_dir() {
        return false; // Ignore directories
    }
    if !path.exists() {
        return false; // Ignore non-existent paths
    }

    if path.try_exists().is_err() {
        return false; // Ignore paths that cannot be checked
    }

    path.extension().map_or(false, |ext| ext == "png")
        && path.file_name().map_or(false, |name| {
            name.to_string_lossy().to_lowercase().contains("screenshot")
        })
}

pub fn get_file_extension(file_path: &Path) -> &str {
    // flatening the first match with and then
    match file_path.extension().and_then(|ext| ext.to_str()) {
        Some(ext) => ext,
        None => "png",
    }
}

pub fn rename_file(file_to_edit: &Path, new_file_name: &String) {
    println!("Renaming file: {}", file_to_edit.display());

    let parent_dir = match file_to_edit.parent() {
        Some(val) => val,
        None => {
            eprintln!("Can't determine parent directory");
            return;
        }
    };

    let file_ext = get_file_extension(file_to_edit);

    let mut new_file_path = parent_dir.join(&new_file_name); // giving this a reference so i can use it below as well

    new_file_path.set_extension(file_ext);

    println!("paths {}", new_file_path.display());
    let res = fs::rename(file_to_edit, new_file_path);

    match res {
        Ok(_) => println!("Successfully renamed file to '{}'", new_file_name),
        Err(e) => eprintln!("Error renaming file: {}", e),
    }
}

// pub fn watch_screenshots(paused_state: Arc<Mutex<bool>>) -> notify::Result<()> {
pub fn watch_screenshots(
    paused_state: Arc<Mutex<bool>>,
    app_handle: AppHandle,
    user_device: UserDevice,
    directory_to_watch: Arc<Mutex<String>>,
) -> notify::Result<()> {
    sentry::capture_message("In watch screenshots", sentry::Level::Info);
    dotenv().ok();

    let (tx, rx) = mpsc::channel::<Result<Event>>();
    let mut watcher = recommended_watcher(tx)?;

    // let screenshot_dir = std::env::var("HOME").unwrap() + "/Desktop";

    println!(
        "📸 Watching screenshots in {}",
        directory_to_watch.lock().expect("uh oh").clone()
    );

    // let screenshot_dir = {
    //     let guard = directory_to_watch.lock().unwrap();
    //     let path = guard.clone();

    //     // Handle case where no directory was selected
    //     if path.is_empty() {
    //         return Err(notify::Error::generic("No directory selected for watching"));
    //     }

    //     // Validate that the path exists
    //     if !std::path::Path::new(&path).exists() {
    //         panic!("Directory does not exist: {}", path);
    //         // return Err(notify::Error::generic(&format!(
    //         //     "Directory does not exist: {}",
    //         //     path
    //         // )));
    //     }

    //     path
    // };
    // let guard = directory_to_watch.lock().unwrap();
    // *guard
    // };
    let screenshot_dir:String = match app_handle.path().desktop_dir() {
        Ok(path) => match path.to_str() {
            Some(s) => s.to_string(),
            None => {
                eprintln!("❌ Failed to convert desktop path to UTF-8 string");
                return Ok(());
            }
        },
        Err(e) => {
            eprintln!("❌ Failed to get desktop directory: {}", e);
            panic!("Failed to get desktop directory");
        }
    };

    watcher.watch(Path::new(&screenshot_dir), RecursiveMode::NonRecursive)?;

    println!("📸 Watching screenshots in: {}", screenshot_dir);
    sentry::capture_message("Watching screenshots in desktop", sentry::Level::Info);

    let mut recently_handled: HashMap<String, Instant> = HashMap::new();
    let cooldown = Duration::from_secs(5);
    // let state = app.state::<AppState>();
    sentry::capture_message("Entering loop", sentry::Level::Info);
    loop {
        let is_paused = {
            let guard = paused_state.lock().unwrap();
            *guard
        };

        if is_paused {
            // Draining the event queue to avoid blocking
            // This will clear any pending events while paused
            while rx.try_recv().is_ok() {}
            thread::sleep(Duration::from_millis(100));
            continue; // skip processing while paused
        }

        let now = Instant::now();

        match rx.recv_timeout(Duration::from_secs(1)) {
            Ok(Ok(event)) => {
                if event.paths.is_empty() {
                    continue; // No paths in the event
                }
                if !event.kind.is_modify() && !event.kind.is_create() && !event.kind.is_access() {
                    println!("⚠️ Skipping non-relevant event kind: {:?}", event.kind);
                    continue;
                }

                for path in event.paths {
                    if !is_new_screenshot(&path) {
                        continue;
                    }

                    let path_str = path.to_string_lossy().to_string();

                    // Skip if this file was processed too recently
                    if let Some(&last_seen) = recently_handled.get(&path_str) {
                        if now.duration_since(last_seen) < cooldown {
                            continue;
                        }
                    }

                    // Record the file as handled
                    recently_handled.insert(path_str.clone(), now);

                    println!("🖼️ Processing new screenshot: {}", path.display());

                    let name = generate_screenshot_name(&path, &user_device, &app_handle);
                    println!("📁 Suggested name: {}", name);

                    rename_file(&path, &name);

                    if let Err(e) = app_handle.emit_to("main", "screenshot-renamed", name.clone()) {
                        eprintln!("❌ Failed to emit event: {:?}", e);
                    }

                    // on_rename(&name);
                    // notify_user(&name, &app_handle);
                }
            }
            Ok(Err(e)) => println!("❌ Watch error: {:?}", e),
            Err(_) => {} // Timeout — no new events
        }

        // Prune old entries
        recently_handled.retain(|_, &mut t| now.duration_since(t) < Duration::from_secs(30));
    }
}
