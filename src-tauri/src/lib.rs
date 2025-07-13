// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
use tauri::tray::TrayIconBuilder;
use tauri::menu::{Menu, MenuItem};

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            // Load the PNG file and convert to RGBA
            let img = image::open("icons/icon.png").unwrap();
            let rgba = img.to_rgba8();
            let (width, height) = rgba.dimensions();
            
            // Create menu with "Connected" and "Exit" items
            let connected_item = MenuItem::with_id(app, "connected", "Connected", true, None::<&str>)?;
            let exit_item = MenuItem::with_id(app, "exit", "Exit", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&connected_item, &exit_item])?;
            
            let tray = TrayIconBuilder::new()
                .icon(tauri::image::Image::new_owned(rgba.into_raw(), width, height))
                .menu(&menu)
                .on_menu_event(|app, event| {
                    match event.id.as_ref() {
                        "exit" => {
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
        .invoke_handler(tauri::generate_handler![greet])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
