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
    let sidecar_state_window = sidecar_state.clone();

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

            // Create menu with "Connected" and "Exit" items
            let connected_item = MenuItem::with_id(app, "connected", "Connected", true, None::<&str>)?;
            let exit_item = MenuItem::with_id(app, "exit", "Exit", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&connected_item, &exit_item])?;
            
            let _tray = TrayIconBuilder::new()
                .icon(tauri::image::Image::from_path(app.path().resolve(ICON_PATH, BaseDirectory::Resource).unwrap()).unwrap())
                .menu(&menu)
                .on_menu_event(move |app, event| {
                    match event.id.as_ref() {
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
        .on_window_event(move |_window, event| {
            if let tauri::WindowEvent::CloseRequested { .. } = event {
                // Kill the sidecar process when the window is closed
                let state = sidecar_state_window.lock().unwrap();
                if let Some(_pid) = state.process_id {
                    // Note: We can't access the process plugin here, so we'll rely on the tray exit handler
                    println!("Window closing, sidecar process will be killed on app exit");
                }
            }
        })
        .invoke_handler(tauri::generate_handler![greet])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
