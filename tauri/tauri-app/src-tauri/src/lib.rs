use std::sync::Arc;

use crate::file_server::{FileServer, FileServerStatus};
use moduels::udp_broadcast::UdpBroadcast;
use tauri::{AppHandle, State};
use tokio::runtime::Builder as RtBuilder;

mod file_server;
mod moduels;

// Application state for sharing between commands
struct AppState {
    file_server: Arc<FileServer>,
    server_address: String,
}

// Tauri commands
#[tauri::command]
fn get_server_address(state: State<'_, AppState>) -> String {
    state.server_address.clone()
}

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
fn get_local_ip() -> Result<String, String> {
    match local_ip_address::local_ip() {
        Ok(ip) => Ok(ip.to_string()),
        Err(e) => Err(format!("Failed to get local IP address: {}", e)),
    }
}

// Only keep status query command, fix return type to Result
#[tauri::command]
async fn get_file_server_status(state: State<'_, AppState>) -> Result<FileServerStatus, String> {
    Ok(state.file_server.status().await)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let rt = RtBuilder::new_multi_thread()
        .worker_threads(2)
        .enable_all()
        .build()
        .unwrap();

    // Get local IP address for the server
    let server_ip = local_ip_address::local_ip()
        .map(|ip| ip.to_string())
        .unwrap_or_else(|_| "127.0.0.1".to_string());

    // Initialize file server with downloads directory
    let downloads_dir = dirs::download_dir().unwrap_or_else(|| {
        let home = dirs::home_dir().unwrap();
        home.join("Downloads")
    });

    let file_server = Arc::new(FileServer::new(server_ip.clone(), 8090, downloads_dir));

    // Create app state to share between commands
    let app_state = AppState {
        file_server: file_server.clone(),
        server_address: server_ip.clone(),
    };

    // Start HTTP file server in background automatically
    let http_server = file_server.clone();
    rt.spawn(async move {
        println!("Starting HTTP file server...");
        match http_server.start().await {
            Ok(status) => println!("HTTP file server started at: {}", status.url),
            Err(e) => eprintln!("Failed to start HTTP file server: {:?}", e),
        }
    });

    // Start UDP broadcast in background
    rt.spawn(async move {
        let udp_broadcast = UdpBroadcast::new("255.255.255.255".to_string(), 12345).await;
        tokio::select! {
            _ = udp_broadcast.start_broadcasting() => {},
            _ = udp_broadcast.listen_for_responses() => {},
        }
    });

    tauri::Builder::default()
        .manage(app_state)
        .plugin(tauri_plugin_udp::init())
        .plugin(tauri_plugin_single_instance::init(
            |_app_handle: &AppHandle, argv: Vec<String>, cwd: String| {
                println!(
                    "Another instance tried to launch with args: {:?}, cwd: {}",
                    argv, cwd
                );
                println!("Focus the existing instance.");
            },
        ))
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            greet,
            get_server_address,
            get_local_ip,
            get_file_server_status
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
