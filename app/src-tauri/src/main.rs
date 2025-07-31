// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    let _guard = sentry::init(("https://9c3e3a3343b8945302f367eec2d92d48@o4509742887010304.ingest.us.sentry.io/4509742893498368", sentry::ClientOptions {
    release: sentry::release_name!(),
    // Capture user IPs and potentially sensitive headers when using HTTP server integrations
    // see https://docs.sentry.io/platforms/rust/data-management/data-collected for more info
    send_default_pii: true,
    ..Default::default()
  }));

    // Sentry will capture this
    //   panic!("Everything is on fire!");

    app_lib::run();
}
