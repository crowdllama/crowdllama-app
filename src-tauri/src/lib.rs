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
use serde::{Deserialize, Serialize};
use prost::Message;

#[cfg(unix)]
use tokio::net::UnixStream;
#[cfg(unix)]
use tokio::io::{AsyncReadExt, AsyncWriteExt};

const CROWDLLAMA_BIN_PATH: &str = "crowdllama/crowdllama";
const ICON_PATH: &str = "icons/icon.png";

// JSON structures for React-Rust communication
#[derive(Serialize, Deserialize, Debug)]
pub struct JsonGenerateRequest {
    pub model: String,
    pub prompt: String,
    pub stream: bool,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct JsonGenerateResponse {
    pub model: String,
    pub response: String,
    pub done: bool,
    pub worker_id: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type")]
pub enum JsonBaseMessage {
    GenerateRequest { data: JsonGenerateRequest },
    GenerateResponse { data: JsonGenerateResponse },
}

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
async fn send_ipc_message(message: JsonBaseMessage) -> Result<String, String> {
    println!("🔄 Received message from React: {:?}", message);
    
    // Convert JSON to protobuf
    let pb_message = match message {
        JsonBaseMessage::GenerateRequest { data } => {
            let request = crowdllama_pb_llama::GenerateRequest {
                model: data.model,
                prompt: data.prompt,
                stream: data.stream,
            };
            crowdllama_pb_llama::BaseMessage {
                message: Some(crowdllama_pb_llama::base_message::Message::GenerateRequest(request)),
            }
        }
        JsonBaseMessage::GenerateResponse { data } => {
            let response = crowdllama_pb_llama::GenerateResponse {
                model: data.model,
                response: data.response,
                done: data.done,
                worker_id: data.worker_id,
            };
            crowdllama_pb_llama::BaseMessage {
                message: Some(crowdllama_pb_llama::base_message::Message::GenerateResponse(response)),
            }
        }
    };
    
    // Encode to protobuf bytes
    let encoded_message = pb_message.encode_to_vec();
    println!("📦 Encoded protobuf message: {} bytes", encoded_message.len());
    
    // Send to IPC socket (Unix only)
    #[cfg(unix)]
    {
        match send_to_ipc_socket(&encoded_message).await {
            Ok(_) => {
                println!("✅ Successfully sent message to IPC socket");
                Ok("Message sent successfully".to_string())
            }
            Err(e) => {
                println!("❌ Failed to send message to IPC socket: {}", e);
                Err(format!("Failed to send message: {}", e))
            }
        }
    }
    
    #[cfg(windows)]
    {
        println!("⚠️ IPC socket not supported on Windows - message logged only");
        Ok("Message logged (Windows stub)".to_string())
    }
}

#[cfg(unix)]
async fn send_to_ipc_socket(message: &[u8]) -> Result<(), Box<dyn std::error::Error>> {
    use tokio::time::{timeout, Duration};
    
    // Try to connect to the IPC socket
    let mut stream = timeout(
        Duration::from_secs(2),
        UnixStream::connect(ipc::SOCKET_PATH)
    ).await??;
    
    // Write length prefix (4 bytes, big-endian)
    let length = message.len() as u32;
    stream.write_all(&length.to_be_bytes()).await?;
    
    // Write the message
    stream.write_all(message).await?;
    stream.flush().await?;
    
    println!("📤 Sent {} bytes to IPC socket", message.len());
    Ok(())
}

#[tauri::command]
async fn simulate_ipc_message() -> Result<JsonBaseMessage, String> {
    // Simulate receiving a message from IPC and converting to JSON
    let pb_response = crowdllama_pb_llama::GenerateResponse {
        model: "simulated-model".to_string(),
        response: "This is a simulated response from the IPC system".to_string(),
        done: true,
        worker_id: "worker-sim-123".to_string(),
    };
    
    let pb_message = crowdllama_pb_llama::BaseMessage {
        message: Some(crowdllama_pb_llama::base_message::Message::GenerateResponse(pb_response)),
    };
    
    // Convert protobuf to JSON
    let json_message = match pb_message.message {
        Some(crowdllama_pb_llama::base_message::Message::GenerateRequest(req)) => {
            JsonBaseMessage::GenerateRequest {
                data: JsonGenerateRequest {
                    model: req.model,
                    prompt: req.prompt,
                    stream: req.stream,
                }
            }
        }
        Some(crowdllama_pb_llama::base_message::Message::GenerateResponse(resp)) => {
            JsonBaseMessage::GenerateResponse {
                data: JsonGenerateResponse {
                    model: resp.model,
                    response: resp.response,
                    done: resp.done,
                    worker_id: resp.worker_id,
                }
            }
        }
        None => {
            return Err("Empty message".to_string());
        }
    };
    
    println!("📨 Simulated IPC message: {:?}", json_message);
    Ok(json_message)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let sidecar_state = sidecar::new_sidecar_state();
    let sidecar_state_clone = sidecar_state.clone();

    // Create a Tokio runtime for the socket listener
    let _rt = tokio::runtime::Runtime::new().expect("Failed to create Tokio runtime");
    
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
        .invoke_handler(tauri::generate_handler![greet, send_ipc_message, simulate_ipc_message])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
