// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
use std::{
    fs,
    io::Read,
    path::Path,
    sync::{Arc, Mutex},
};
use tauri::{AppHandle, Emitter, Manager};
// use tauri::tray::{TrayIconEvent, MouseButton, MouseButtonState};

use base64::{engine::general_purpose, Engine};
use tauri_plugin_positioner::{Position, WindowExt};
#[cfg(target_os = "macos")]
use window_vibrancy::NSVisualEffectState;

use dotenvy::dotenv;
use notify::{recommended_watcher, Event, RecursiveMode, Result, Watcher};
use reqwest::blocking::Client;
use serde_json::json;
use std::collections::HashMap;
use std::env;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;
use std::time::Instant;

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

pub fn encode_image_to_base64(path: &Path) -> String {
    let mut file = match fs::File::open(path) {
        Ok(file) => file,
        Err(e) => {
            eprintln!("❌ Error opening file {}: {}", path.display(), e);
            return String::new();
        }
    };
    let mut buffer = Vec::new();
    if let Err(e) = file.read_to_end(&mut buffer) {
        eprintln!("❌ Error reading file {}: {}", path.display(), e);
        return String::new();
    }
    let base64_string = general_purpose::STANDARD.encode(&buffer);
    base64_string
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
    pub fn watch_screenshots(paused_state: Arc<Mutex<bool>>, app_handle: AppHandle) -> notify::Result<()> {

    // pub fn watch_screenshots<F>(
    //     paused_state: Arc<Mutex<bool>>,
    //     mut on_rename: F,
    // ) -> notify::Result<()>
    // where
    //     F: FnMut(&str) + Send + 'static,
    // {
    dotenv().ok();
    let key = env::var("OPENAI_API_KEY").expect("API key not found");
    println!("Using OpenAI key: {}", &key[..6]);

    let (tx, rx) = mpsc::channel::<Result<Event>>();
    let mut watcher = recommended_watcher(tx)?;

    let screenshot_dir = std::env::var("HOME").unwrap() + "/Desktop";
    watcher.watch(Path::new(&screenshot_dir), RecursiveMode::NonRecursive)?;

    println!("📸 Watching screenshots in: {}", screenshot_dir);

    let mut recently_handled: HashMap<String, Instant> = HashMap::new();
    let cooldown = Duration::from_secs(5);
    // let state = app.state::<AppState>();

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

                    let name = generate_screenshot_name(&path);
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

fn wait_until_exist(path: &Path, attempts: u8, delay: Duration) -> bool {
    for _ in 0..attempts {
        if path.exists() {
            match fs::File::open(path) {
                Ok(mut file) => {
                    let mut buffer = Vec::new();
                    if file.read_to_end(&mut buffer).is_ok() {
                        return true; // File exists and is readable
                    }
                }
                Err(_) => {}
            }
        }
        thread::sleep(delay);
    }
    false
}

pub fn generate_screenshot_name(image_path: &Path) -> String {
    dotenvy::dotenv().ok();

    let api_key = match env::var("OPENAI_API_KEY") {
        Ok(val) => val,
        Err(_) => {
            eprintln!("❌ Error: OPENAI_API_KEY not set.");
            return "screenshot".to_string();
        }
    };

    if !image_path.exists() {
        eprintln!(
            "❌ Error: Screenshot file does not exist at path: {}",
            image_path.display()
        );
        return "screenshot".to_string();
    }
    if !image_path.is_file() {
        eprintln!("❌ Error: Path is not a file: {}", image_path.display());
        return "screenshot".to_string();
    }

    println!(
        "📸 Generating name for screenshot: {}",
        image_path.display()
    );

    if wait_until_exist(image_path, 10, Duration::from_secs(1)) {
        let encoded = encode_image_to_base64(&image_path);
        let image_data_url = format!("data:image/png;base64,{}", encoded);

        let payload = json!({
            "model": "gpt-4.1",
            "messages": [
                {
                    "role": "user",
                    "content": [
                        {
                            "type": "image_url",
                            "image_url": { "url": image_data_url }
                        },
                        {
                            "type": "text",
                            "text": "Return a short and descriptive filename for this screenshot, lowercase with no spaces or extension."
                        }
                    ]
                }
            ],
            "max_tokens": 50
        });

        let client = Client::new();
        let response = match client
            .post("https://api.openai.com/v1/chat/completions")
            .bearer_auth(api_key)
            .json(&payload)
            .send()
        {
            Ok(res) => res,
            Err(_) => {
                eprintln!("❌ Error: Failed to send request to OpenAI.");
                return "screenshot".to_string();
            }
        };

        let response = match response.error_for_status_ref() {
            Ok(_) => response,
            Err(e) => {
                eprintln!("❌ OpenAI API error: {}", e);

                // Try to read the body even if it's an error
                match response.text() {
                    Ok(body) => eprintln!("💬 OpenAI error body: {}", body),
                    Err(_) => eprintln!("⚠️ Could not read error body"),
                }

                return "screenshot".to_string();
            }
        };

        let json: serde_json::Value = match response.json() {
            Ok(j) => j,
            Err(_) => {
                eprintln!("❌ Error: Failed to parse OpenAI response.");
                return "screenshot".to_string();
            }
        };

        let filename = json["choices"][0]["message"]["content"]
            .as_str()
            .unwrap_or("screenshot")
            .trim()
            .to_string();

        filename
    } else {
        eprintln!("❌ Screenshot did not become available in time.");
        return "screenshot".to_string();
    }
}

// https://github.com/tauri-apps/window-vibrancy/tree/dev
