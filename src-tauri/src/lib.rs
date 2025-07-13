// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
use tauri::tray::TrayIconBuilder;
use tauri::menu::{Menu, MenuItem};
use tauri::path::BaseDirectory;
use tauri::Manager;
use std::sync::Arc;
use std::sync::Mutex;
use std::process::Command;

const CROWDLLAMA_BIN_PATH: &str = "crowdllama/crowdllama";
const ICON_PATH: &str = "icons/icon.png";
// Global state to hold the sidecar process
struct SidecarState {
    process_id: Option<u32>,
}

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let sidecar_state = Arc::new(Mutex::new(SidecarState { process_id: None }));
    let sidecar_state_clone = sidecar_state.clone();

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(move |app| {
            // Get the path to the crowdllama binary
            let binary_path = app.path().resolve(CROWDLLAMA_BIN_PATH, BaseDirectory::Resource)
                .expect("Failed to find crowdllama binary");

            // Spawn the sidecar process using std::process::Command
            let child = Command::new(binary_path)
                .spawn()
                .expect("Failed to spawn crowdllama process");

            // Store the process ID for cleanup
            {
                let mut state = sidecar_state.lock().unwrap();
                state.process_id = Some(child.id());
            }

            println!("Spawned crowdllama sidecar process with ID: {}", child.id());

            // Create menu with "Show", "Connected" and "Exit" items
            let show_item = MenuItem::with_id(app, "show", "Show", true, None::<&str>)?;
            let connected_item = MenuItem::with_id(app, "connected", "Connected", true, None::<&str>)?;
            let exit_item = MenuItem::with_id(app, "exit", "Exit", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&show_item, &connected_item, &exit_item])?;
            
            let _tray = TrayIconBuilder::new()
                .icon(tauri::image::Image::from_path(app.path().resolve(ICON_PATH, BaseDirectory::Resource).unwrap()).unwrap())
                .menu(&menu)
                .on_menu_event(move |app, event| {
                    match event.id.as_ref() {
                        "show" => {
                            // Show the main window
                            if let Some(window) = app.get_webview_window("main") {
                                window.show().unwrap();
                                window.set_focus().unwrap();
                            }
                        }
                        "exit" => {
                            // Kill the sidecar process before exiting
                            let state = sidecar_state_clone.lock().unwrap();
                            if let Some(pid) = state.process_id {
                                // Use kill command to terminate the process
                                let _ = Command::new("kill").arg(pid.to_string()).output();
                                println!("Killed crowdllama sidecar process with ID: {}", pid);
                            }
                            app.exit(0);
                        }
                        "connected" => {
                            println!("Connected status clicked");
                        }
                        _ => {}
                    }
                })
                .build(app)?;
            Ok(())
        })
        .on_window_event(move |window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                // Prevent the default close behavior
                api.prevent_close();
                // Hide the window instead of closing it
                window.hide().unwrap();
                println!("Window hidden, app continues running in system tray");
            }
        })
        .invoke_handler(tauri::generate_handler![greet])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
