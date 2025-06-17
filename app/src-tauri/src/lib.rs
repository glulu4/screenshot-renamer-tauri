// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
use std::{sync::{Arc, Mutex}};
use tauri::{
    menu::{Menu, MenuItem}, tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent}, App, AppHandle, Manager, WebviewWindow, Window
};
// use tauri::tray::{TrayIconEvent, MouseButton, MouseButtonState};
use tauri_plugin_positioner::{Position, WindowExt};
#[cfg(target_os = "macos")]
use tauri::Emitter;
use std::env;
use crate::watch::watch_screenshots;
use serde_json::json;
mod watch;
mod state;
use state::AppState;



#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}




// static WATCHER_PAUSED: Lazy<Arc<Mutex<bool>>> = Lazy::new(|| Arc::new(Mutex::new(false)));


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
    .plugin(tauri_plugin_opener::init())
    .plugin(tauri_plugin_positioner::init())
    .plugin(tauri_plugin_notification::init())
    .setup(|app: &mut App| {

        // Initialize the application state
        let state = AppState {
            paused: Arc::new(Mutex::new(false)),
        };
        app.manage(state.clone());

        // Spawn the screenshot watcher on a separate thread. Whenever a file is
        // renamed we emit an event so the frontend can display a notification.
        std::thread::spawn({
            let paused_state = state.paused.clone();
            let app_handle = app.handle();
            move || {
                if let Err(e) = watch_screenshots(paused_state, |name| {
                    if let Err(e) = app_handle.emit_all(
                        "screenshot-renamed",
                        json!({ "name": name }),
                    ) {
                        eprintln!("❌ Failed to emit screenshot-renamed event: {}", e);
                    }
                }) {
                    eprintln!("❌ Error in watcher: {:?}", e);
                }
            }
        });

        // std::thread::spawn({
        //     let paused_state = state.paused.clone();
        //     move || {
        //         if let Err(e) = watch_screenshots(paused_state, |name| {
        //             // Emit event instead of showing notification directly
        //             if let Err(e) = app.emit("screenshot-renamed", name) {
        //                 eprintln!("❌ Failed to emit screenshot-renamed event: {}", e);
        //             }
        //         }) {
        //             eprintln!("❌ Error in watcher: {:?}", e);
        //         }
        //     }
        // });



        use tauri_plugin_notification::NotificationExt;
        app.notification()
            .builder()
            .title("Tauri")
            .body("Tauri is awesome")
            .show()
            .unwrap();

        // Titlebar. 

            ///HudWindow
            /// Popover
            /// Menu
            // FullScreenUI
        use window_vibrancy::{apply_blur, apply_vibrancy, NSVisualEffectMaterial};
        let window = app.get_webview_window("main").unwrap();

        #[cfg(target_os = "macos")]
        // let state: Option<NSVisualEffectState> = {
        //     use window_vibrancy::NSVisualEffectState;
        //     Some(NSVisualEffectState::Inactive)

        // };
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

            .on_tray_icon_event( |tray_handle, event| {
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
                        // let _ = window.set_size(tauri::Size::Logical(tauri::LogicalSize {
                        //     width: 400.0,
                        //     height: 300.0,
                        // }));
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
    .invoke_handler(tauri::generate_handler![greet, set_watcher_paused])
    // .invoke_handler(tauri::generate_handler![set_watcher_paused])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");

}


    // tauri::Builder::default()
    //     .plugin(tauri_plugin_opener::init())
    //     .invoke_handler(tauri::generate_handler![greet])
    //     .run(tauri::generate_context!())
    //     .expect("error while running tauri application");

