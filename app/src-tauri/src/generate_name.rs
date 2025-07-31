// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
use base64::{engine::general_purpose, Engine};
#[cfg(target_os = "macos")]
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
use std::thread;
use std::time::Duration;
use std::{fs, io::Read, path::Path};
use tauri::{AppHandle, Emitter};

use crate::user::UserDevice;
use dotenvy_macro::dotenv;

#[derive(Serialize)]
struct RequestPayload<'a> {
    #[serde(rename = "base64Img")]
    base_64_img: &'a str,
    #[serde(rename = "deviceId")]
    device_id: &'a str,
    #[serde(rename = "appVersion")]
    app_version: &'a str,
}

#[derive(Deserialize)]
struct ApiResponse {
    success: bool,
    message: String,
    data: ScreenshotData,
}

#[derive(Deserialize)]
struct ScreenshotData {
    #[serde(rename = "screenshotName")]
    screenshot_name: String,
}

pub fn generate_screenshot_name(
    image_path: &Path,
    user_device: &UserDevice,
    app_handle: &AppHandle,
) -> String {
    sentry::capture_message("In generate_screenshot_name", sentry::Level::Info);

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
    // Wait for the screenshot to become available
    sentry::capture_message(
        "Waiting for screenhots to become available",
        sentry::Level::Info,
    );

    if wait_until_exist(image_path, 15, Duration::from_secs(1)) {
        let encoded = encode_image_to_base64(&image_path);
        let image_data_url = format!("data:image/png;base64,{}", encoded);

        let payload = RequestPayload {
            base_64_img: image_data_url.as_str(),
            device_id: &user_device.device_id,
            app_version: &user_device.app_version,
        };
        const DEFAULT_NAME: &str = "screenshot";

        dotenvy::dotenv().ok();

        //   let api_url = match env::var("GEN_SCREENSHOT_NAME_URL") {
        //       Ok(val) => val,
        //       Err(_) => {
        //           eprintln!("❌ Error: GEN_SCREENSHOT_NAME_URL not set.");
        //           return "screenshot".to_string();
        //       }
        //   };
        let api_url = dotenv!("GEN_SCREENSHOT_NAME_URL");

        println!("app_version: {}", payload.app_version);
        println!(
            "base_64_img: {}",
            payload.base_64_img.chars().take(50).collect::<String>()
        ); // Print first 50 chars for brevity
        println!("device_id: {}", payload.device_id);
        println!("api_url: {}", api_url);

        let client = Client::new();
        let response = match client.post(api_url).json(&payload).send() {
            Ok(res) => {
                match res.status().as_u16() {
                    429 => {
                        eprintln!("🚫 Quota exceeded (429): Free plan limit reached");
                        app_handle
                            .emit_to("main", "quota-exceeded", "Free plan limit reached")
                            .unwrap();

                        return "screenshot".to_string(); // or whatever you want to display
                    }
                    200..=299 => res,
                    code => {
                        eprintln!("❌ Server returned unexpected status: {}", code);
                        return DEFAULT_NAME.to_string();
                    }
                }
            }
            Err(e) => {
                eprintln!("❌ Error: Failed to send request: {}", e);
                return DEFAULT_NAME.to_string();
            }
        };

        // Parse JSON response
        let api_response: ApiResponse = match response.json() {
            Ok(json) => json,
            Err(e) => {
                eprintln!("❌ Error: Failed to parse JSON response: {}", e);
                return DEFAULT_NAME.to_string();
            }
        };

        // Check API success status
        if !api_response.success {
            eprintln!(
                "❌ Error: API response indicates failure: {}",
                api_response.message
            );
            return DEFAULT_NAME.to_string();
        }

        // Return cleaned filename
        api_response.data.screenshot_name.trim().to_string()
    } else {
        sentry::capture_message(
            "Screenshot did not become available in time",
            sentry::Level::Warning,
        );
        eprintln!("❌ Screenshot did not become available in time.");
        return "screenshot".to_string();
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


fn wait_until_exist(path: &Path, attempts: u8, delay: Duration) -> bool {
    sentry::capture_message("🕵️ Inside wait_until_exist", sentry::Level::Info);
    sentry::capture_message(&format!("🔍 Target path: {}", path.display()), sentry::Level::Info);

    for i in 0..attempts {
        if path.exists() {
            match fs::File::open(path) {
                Ok(mut file) => {
                    let mut buffer = Vec::new();
                    match file.read_to_end(&mut buffer) {
                        Ok(_) => {
                            sentry::capture_message("✅ File exists and is readable", sentry::Level::Info);
                            return true;
                        }
                        Err(e) => {
                            sentry::capture_message(
                                &format!("❌ File exists but read failed (attempt {}): {}", i + 1, e),
                                sentry::Level::Warning,
                            );
                        }
                    }
                }
                Err(e) => {
                    sentry::capture_message(
                        &format!("❌ File exists but could not open (attempt {}): {}", i + 1, e),
                        sentry::Level::Warning,
                    );
                }
            }
        } else {
            sentry::capture_message(
                &format!("🕒 File does not exist yet (attempt {})", i + 1),
                sentry::Level::Info,
            );
        }
        thread::sleep(delay);
    }

    sentry::capture_message("❌ wait_until_exist failed after all attempts", sentry::Level::Error);
    false
}


// fn wait_until_exist(path: &Path, attempts: u8, delay: Duration) -> bool {
//     sentry::capture_message("Inside wait_until_exist", sentry::Level::Info);
//     sentry::capture_message(
//         &format!("image path {}", path.display()),
//         sentry::Level::Info,
//     );
//     for _ in 0..attempts {
//         if path.exists() {
//             match fs::File::open(path) {
//                 Ok(mut file) => {
//                     let mut buffer = Vec::new();
//                     if file.read_to_end(&mut buffer).is_ok() {
//                         return true; // File exists and is readable
//                     }
//                 }
//                 Err(_) => {}
//             }
//         }
//         thread::sleep(delay);
//     }
//     false
// }
