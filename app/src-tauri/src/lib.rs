// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
use std::sync::{Arc, Mutex};
use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    App, AppHandle, Manager, WebviewWindow, Window,
};
use crate::watch::watch_screenshots;
use std::env;
#[cfg(target_os = "macos")]
use tauri::Emitter;
use tauri_plugin_positioner::{Position, WindowExt};
mod state;
mod watch;
use state::AppState;

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
fn trigger_test_event_from_rust(app: AppHandle, msg: &str) {

    app.emit_to("SnapName", "screenshot-renamed", msg).unwrap();
}

#[tauri::command]
fn set_watcher_paused(state: tauri::State<AppState>, paused: bool) {
    let mut guard = state.paused.lock().unwrap();
    *guard = paused;
    println!("✅ Watcher paused state updated: {}", paused);
}

// https://github.com/tauri-apps/window-vibrancy/tree/dev
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_log::Builder::new().build())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_positioner::init())
        .plugin(tauri_plugin_notification::init())
        .setup(|app: &mut App| {
            // Initialize the application state
            let state = AppState {
                paused: Arc::new(Mutex::new(false)),
            };
            app.manage(state.clone());


            trigger_test_event_from_rust(app.app_handle().clone(), "Hello from Rust!");
            let app_handle = app.app_handle().clone(); // clone app handle for thread
            std::thread::spawn({
                let paused_state = state.paused.clone(); // clone Arc for thread
                move || {
                    if let Err(e) = watch_screenshots(paused_state, app_handle) {
                        eprintln!("❌ Error in watcher: {:?}", e);
                    }
                }
            });

            use tauri_plugin_notification::NotificationExt;
            app.notification()
                .builder()
                .title("Tauri")
                .body("Tauri is awesome")
                .show()
                .unwrap();


            ///HudWindow
            /// Popover
            /// Menu
            // FullScreenUI
            use window_vibrancy::{apply_blur, apply_vibrancy, NSVisualEffectMaterial};
            let window = app.get_webview_window("main").unwrap();

            #[cfg(target_os = "macos")]

            apply_vibrancy(&window, NSVisualEffectMaterial::Popover, None, Some(6.0))
                .expect("Unsupported platform! 'apply_vibrancy' is only supported on macOS");

            #[cfg(target_os = "windows")]
            apply_blur(&window, Some((18, 18, 18, 125)))
                .expect("Unsupported platform! 'apply_blur' is only supported on Windows");

            let quit_i = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&quit_i])?;
            let _ = TrayIconBuilder::new()
                .icon(app.default_window_icon().unwrap().clone())
                .tooltip("Screenshot Renamer")
                .on_tray_icon_event(|tray_handle, event| {
                    tauri_plugin_positioner::on_tray_event(tray_handle.app_handle(), &event);
                    if let TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        ..
                    } = event
                    {
                        let webview = tray_handle.app_handle().get_webview_window("main").unwrap();
                        let window = webview.as_ref().window();

                        if !window.is_visible().unwrap() {
                            let _ = window.move_window(Position::TrayBottomCenter);
                            let _ = window.show();
                            let _ = window.set_focus();
                        } else {
                            let _ = window.hide();
                        }
                    }
                })
                .menu(&menu)
                .show_menu_on_left_click(false)
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "quit" => {
                        println!("quit menu item was clicked");
                        app.exit(0);
                    }
                    _ => {
                        println!("menu item {:?} not handled", event.id);
                    }
                })
                .build(app)?;

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            greet,
            set_watcher_paused,
            trigger_test_event_from_rust,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
