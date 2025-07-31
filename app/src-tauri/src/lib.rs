// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
use std::sync::{Arc, Mutex};
use tauri::{
    image::Image,
    menu::{MenuBuilder, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    App, AppHandle, Manager,
};

use crate::{
    user::{Tier, UserDevice},
    watch::watch_screenshots,
};
use std::env;
#[cfg(target_os = "macos")]
use tauri_plugin_positioner::{Position, WindowExt};
#[cfg(target_os = "macos")]
use window_vibrancy::NSVisualEffectState;
mod generate_name;
mod state;
mod user;
mod watch;
use state::AppState;
use std::sync::mpsc;
// use tauri_plugin_dialog::{DialogExt, FilePath};
use tauri_plugin_notification::NotificationExt;
use user::register;
// was in the icons tauri config file
// "icons/32x32.png",
// "icons/128x128.png",
// "icons/128x128@2x.png",
// "icons/icon.icns",
// "icons/icon.ico"

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
fn set_watcher_paused(state: tauri::State<AppState>, paused: bool) {
    let mut guard = state.paused.lock().unwrap();
    *guard = paused;
    println!("✅ Watcher paused state updated: {}", paused);
}

#[tauri::command]
fn get_device_id() -> String {
    return machine_uid::get().unwrap_or_else(|_| "unknown-device".into());
}
#[tauri::command]
fn get_user_tier(state: tauri::State<AppState>) -> Tier {
    let user_device = state.user_device.clone();
    match user_device.tier {
        Tier::Free => Tier::Free,
        Tier::Pro => Tier::Pro,
    }
}

// #[tauri::command]
// async fn select_folder(app: tauri::AppHandle) -> Option<String> {

//     let mut path = None;
//     app.dialog().file().pick_folder(move |folder_path| {

//         if let Some(folder) = folder_path {

//             let folder_str = folder.to_string();
//             println!("📂 Selected folder: {}", folder_str);
//             path = Some(folder_str);
//         }
//         else{
//             println!("❌ No folder selected");
//         }
//         // do something with the optional folder path here
//         // the folder path is `None` if the user closed the dialog
//     });
//     return path;
// }



// tauri-plugin-dialog = "2.3.2"

// #[tauri::command]
// async fn select_folder(app: AppHandle) -> Option<String> {

//     let folder_path = app.dialog().file().blocking_pick_folder();

//     let res = match folder_path {
//         Some(path) => {
//             println!("📂 Selected folder: {}", path.to_string());
//             // let state = app.state::<AppState>();
//             // let mut selected_path = state.selected_path.lock().unwrap();
//             // *selected_path = path.to_string_lossy().to_string();
//             Some(path.to_string())
//         }
//         None => {
//             println!("❌ No folder selected");
//             None
//         }
//     };
//     res

// }

fn spawn_watcher_thread(
    paused_state: Arc<Mutex<bool>>,
    app_handle: AppHandle,
    user_device: UserDevice,
    directory_to_watch: Arc<Mutex<String>>,
) {
    std::thread::spawn(move || {
        if let Err(e) = watch_screenshots(paused_state, app_handle, user_device, directory_to_watch)
        {
            eprintln!("❌ Error in watcher: {:?}", e);
        }
    });
}

// https://github.com/tauri-apps/window-vibrancy/tree/dev
// https://docs.rs/tauri-plugin-dialog/2.3.2/tauri_plugin_dialog/struct.FileDialogBuilder.html#method.pick_folder
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    sentry::capture_message("Starting at the run function", sentry::Level::Info);
    tauri::Builder::default()
        .plugin(tauri_plugin_log::Builder::new().build())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_positioner::init())
        .plugin(tauri_plugin_notification::init())
        .setup(|app: &mut App| {
            sentry::capture_message("About to get user device", sentry::Level::Info);
            let user_device: UserDevice = register().expect("Failed to register user device");

            sentry::capture_message(
                &format!(
                    "got user user device. device id {}",
                    user_device.device_id.to_owned()
                ),
                sentry::Level::Info,
            );

            let state = AppState {
                paused: Arc::new(Mutex::new(false)),
                user_device: user_device,
                selected_path: Arc::new(Mutex::new(String::new())),
            };
            app.manage(state.clone());

            let app_handle = app.app_handle().clone(); // clone app handle for thread

            spawn_watcher_thread(
                state.paused.clone(),
                app_handle,
                state.user_device.clone(),
                state.selected_path.clone(),
            );

            sentry::capture_message("About to get user device", sentry::Level::Info);
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
            use window_vibrancy::{apply_vibrancy, NSVisualEffectMaterial};
            let window = app.get_webview_window("main").unwrap();

            #[cfg(target_os = "macos")]
            let state: Option<NSVisualEffectState> = {
                use window_vibrancy::NSVisualEffectState;
                // Set the state to `Active` for a more vibrant effect
                Some(NSVisualEffectState::Active)
            };

            apply_vibrancy(&window, NSVisualEffectMaterial::Popover, state, Some(6.0))
                .expect("Unsupported platform! 'apply_vibrancy' is only supported on macOS");

            #[cfg(target_os = "windows")]
            apply_blur(&window, Some((18, 18, 18, 125)))
                .expect("Unsupported platform! 'apply_blur' is only supported on Windows");

            let window_for_blur = window.clone();
            window.on_window_event(move |event| {
                if let tauri::WindowEvent::Focused(false) = event {
                    let window_clone = window_for_blur.clone();

                    // Trigger fade out animation via JavaScript
                    let _ = window_for_blur.eval(
                        "
                        document.body.style.transition = 'opacity 0.2s ease-out';
                        document.body.style.opacity = '0';
                    ",
                    );

                    // Hide window after animation completes
                    std::thread::spawn(move || {
                        std::thread::sleep(std::time::Duration::from_millis(200));
                        let _ = window_clone.hide();

                        // Reset opacity for next show
                        let _ = window_clone.eval(
                            "
                            document.body.style.opacity = '1';
                        ",
                        );
                    });
                }
            });

            let quit_i = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
            let settings_i = MenuItem::with_id(app, "settings", "Settings...", true, None::<&str>)?;

            // let open_i = MenuItem::with_id(app, "open", "Open Coco", true, None::<&str>).unwrap();
            // let about_i = MenuItem::with_id(app, "about", "About Coco", true, None::<&str>).unwrap();
            // let hide_i = MenuItem::with_id(app, "hide", "Hide Coco", true, None::<&str>).unwrap();

            // let menu = Menu::with_items(app, &[&quit_i])?;
            let menu = MenuBuilder::new(app)
                // .item(&open_i)
                // .separator()
                // .item(&hide_i)
                // .item(&about_i)
                .item(&settings_i)
                .separator()
                .item(&quit_i)
                .build()
                .unwrap();

            sentry::capture_message("Set up window", sentry::Level::Info);

            sentry::capture_message("Setting up tray icon ", sentry::Level::Info);
            let _ = TrayIconBuilder::new() //tray-icon.png
                .icon(
                    Image::from_bytes(include_bytes!("../icons/tray-iconTemplate.png"))
                        .expect("Failed to load icon"),
                )
                .tooltip("Screenshot Renamer")
                .icon_as_template(true)
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
                    // "settings" => {
                    //     let app_handle = app.app_handle().clone();
                    //     std::thread::spawn(move || {
                    //         if let Some(folder_path) = select_folder(app_handle.clone()) {

                    //             // https://docs.rs/tauri-plugin-dialog/2.3.2/tauri_plugin_dialog/struct.FileDialogBuilder.html
                    //             let state = app_handle.state::<AppState>();
                    //             let paused = state.paused.clone();
                    //             let user_device = state.user_device.clone();

                    //             if let Err(e) = watch_screenshots(paused, app_handle, user_device) {
                    //                 eprintln!("❌ Error in watcher: {:?}", e);
                    //             }
                    //         }
                    //     });
                    // }
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
            get_device_id,
            get_user_tier,
            // select_folder
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
