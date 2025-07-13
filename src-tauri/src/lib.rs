// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
mod crowdllama_pb {
    include!("../src/crowdllama-pb/rust/ipc.v1.rs");
}

mod crowdllama_pb_llama {
    include!("../src/crowdllama-pb/rust/llama.v1.rs");
}

pub mod ipc;
mod sidecar;



use tauri::tray::TrayIconBuilder;
use tauri::menu::{Menu, MenuItem};
use tauri::path::BaseDirectory;
use tauri::Manager;
use std::sync::Arc;
use std::sync::Mutex;

const CROWDLLAMA_BIN_PATH: &str = "crowdllama/crowdllama";
const ICON_PATH: &str = "icons/icon.png";

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let sidecar_state = sidecar::new_sidecar_state();
    let sidecar_state_clone = sidecar_state.clone();

    // Create a Tokio runtime for the socket listener
    let rt = tokio::runtime::Runtime::new().expect("Failed to create Tokio runtime");
    
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(move |app| {
            // Start the socket listener in a dedicated background thread
            std::thread::spawn(|| {
                let rt = tokio::runtime::Runtime::new().expect("Failed to create Tokio runtime");
                rt.block_on(async {
                    ipc::start_socket_listener().await;
                });
            });

            // Get the path to the crowdllama binary
            let binary_path = app.path().resolve(CROWDLLAMA_BIN_PATH, BaseDirectory::Resource)
                .expect("Failed to find crowdllama binary");

            // Spawn the sidecar process
            {
                let mut state = sidecar_state.lock().unwrap();
                state.spawn_sidecar(binary_path.to_str().unwrap());
            }

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
                            state.kill_sidecar();
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
