mod models;
mod state;
mod broadcast;
mod handlers;
mod websocket;

use std::sync::Arc;
use std::collections::HashMap;
use local_ip_address::local_ip;
use tauri::{AppHandle, State};
use tokio::runtime::Runtime;
use tokio::sync::Mutex as TokioMutex;

use state::AppState;
use broadcast::discover_network_services;

// Tauri命令：获取WebSocket服务器地址
#[tauri::command]
fn get_server_address(state: State<'_, String>) -> String {
    state.inner().clone()
}

// Tauri命令：发现网络上的服务
#[tauri::command]
async fn discover_services() -> Result<Vec<models::BroadcastMessage>, String> {
    let local_ip = local_ip().map_err(|e| format!("Failed to get local IP: {}", e))?;
    discover_network_services(local_ip.to_string(), 8080).await
}

// Tauri命令：获取问候消息
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let rt = Runtime::new().unwrap();

    // 初始化应用状态
    let state = Arc::new(AppState {
        connected_clients: std::sync::Mutex::new(HashMap::new()),
        shared_items: TokioMutex::new(Vec::new()),
    });

    // 启动WebSocket服务器
    let server_addr = rt.block_on(async {
        match websocket::start_websocket_server(state.clone(), 8080).await {
            Ok(addr) => addr,
            Err(e) => {
                eprintln!("Failed to start WebSocket server: {}", e);
                "unknown".to_string()
            }
        }
    });

    // 启动服务发现监听器
    rt.block_on(async {
        match broadcast::start_discovery_service().await {
            Ok(_) => println!("Service discovery listener started"),
            Err(e) => eprintln!("Failed to start service discovery: {}", e),
        }
    });

    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|_app_handle: &AppHandle, argv: Vec<String>, cwd: String| {
            println!("Another instance tried to launch with args: {:?}, cwd: {}", argv, cwd);
            println!("Focus the existing instance.");
        }))
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .manage(state)
        .manage(server_addr)
        .invoke_handler(tauri::generate_handler![
            greet,
            get_server_address,
            discover_services
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
