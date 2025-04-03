mod models;
mod state;
mod broadcast;
mod handlers;
mod websocket;

use std::sync::Arc;
use std::collections::HashMap;
use tauri::{AppHandle, State};
use tokio::runtime::Runtime;
use tokio::sync::Mutex as TokioMutex;

use state::AppState;

// Tauri commands
#[tauri::command]
fn get_server_address(state: State<'_, String>) -> String {
    state.inner().clone()
}

#[tauri::command]
async fn discover_services(state: State<'_, Arc<AppState>>) -> Result<Vec<models::ServerInfo>, String> {
    let services = broadcast::discover_network_services().await?;
    
    let broadcast_messages: Vec<models::BroadcastMessage> = services
        .iter()
        .map(|s| models::BroadcastMessage {
            message_type: "service_info".to_string(),
            server_name: s.server_name.clone(),
            server_address: s.server_address.clone(),
            server_port: s.server_port,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        })
        .collect();
    
    handlers::broadcast_discovered_services(&state.inner(), broadcast_messages).await;
    
    Ok(services)
}

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
fn get_local_ip() -> Result<String, String> {
    match local_ip_address::local_ip() {
        Ok(ip) => Ok(ip.to_string()),
        Err(e) => Err(format!("Failed to get local IP address: {}", e))
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Initialize application state
    let state = Arc::new(AppState {
        connected_clients: std::sync::Mutex::new(HashMap::new()),
        shared_items: TokioMutex::new(Vec::new()),
    });

    // Spawn WebSocket server in a separate thread
    let state_clone = state.clone();
    std::thread::spawn(move || {
        let rt = Runtime::new().unwrap();
        rt.block_on(async move {
            match websocket::start_websocket_server(state_clone, 8080).await {
                Ok(addr) => println!("WebSocket server started at: {}", addr),
                Err(e) => eprintln!("Failed to start WebSocket server: {}", e),
            }
        });
    });

    // Spawn service discovery in a separate thread
    std::thread::spawn(move || {
        let rt = Runtime::new().unwrap();
        rt.block_on(async {
            match broadcast::start_discovery_service().await {
                Ok(_) => println!("Service discovery listener started"),
                Err(e) => eprintln!("Failed to start service discovery: {}", e),
            }
        });
    });

    tauri::Builder::default()
        .plugin(tauri_plugin_udp::init())
        .plugin(tauri_plugin_single_instance::init(|_app_handle: &AppHandle, argv: Vec<String>, cwd: String| {
            println!("Another instance tried to launch with args: {:?}, cwd: {}", argv, cwd);
            println!("Focus the existing instance.");
        }))
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .manage(state)
        .invoke_handler(tauri::generate_handler![
            greet,
            get_server_address,
            discover_services,
            get_local_ip
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
