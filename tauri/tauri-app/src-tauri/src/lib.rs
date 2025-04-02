use futures_util::{SinkExt, StreamExt};
use local_ip_address::local_ip;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tauri::{AppHandle, State};
use tokio::net::{TcpListener, TcpStream, UdpSocket};
use tokio::sync::mpsc;
use tokio::sync::Mutex as TokioMutex;
use tokio::time::sleep;
use tokio_tungstenite::tungstenite::Message;
use uuid::Uuid;

// 广播常量
const BROADCAST_PORT: u16 = 45678;
const SERVICE_DISCOVERY_MSG: &str = "FILE_SHARE_SERVICE";
const BROADCAST_INTERVAL: Duration = Duration::from_secs(5);

// 共享状态结构体
struct AppState {
    connected_clients: Mutex<HashMap<SocketAddr, mpsc::Sender<Message>>>,
    shared_items: TokioMutex<Vec<SharedItem>>,
}

// 广播消息结构体
#[derive(Serialize, Deserialize, Clone)]
struct BroadcastMessage {
    message_type: String,
    server_address: String,
    server_port: u16,
    server_name: String,
    timestamp: u64,
}

// 共享项结构体
#[derive(Serialize, Deserialize, Clone)]
struct SharedItem {
    id: String,
    name: String,
    #[serde(rename = "type")]
    item_type: String, // "text" or "file"
    content: String,
    path: Option<String>,    // Add path field for files
    username: String,
    upload_time: u64,
    size: Option<u64>,      // Add size field for files
    file_type: Option<String>, // Add file_type field
}

// 获取文件类型
fn get_file_type(path: &str) -> Option<String> {
    let extension = std::path::Path::new(path)
        .extension()
        .and_then(|ext| ext.to_str())?
        .to_lowercase();
    
    match extension.as_str() {
        "jpg" | "jpeg" | "png" | "gif" | "bmp" | "webp" => Some("image".to_string()),
        "mp4" | "avi" | "mov" | "wmv" | "flv" | "webm" => Some("video".to_string()),
        "mp3" | "wav" | "ogg" | "m4a" | "aac" => Some("audio".to_string()),
        "pdf" => Some("pdf".to_string()),
        "txt" | "md" | "json" | "xml" | "csv" => Some("text".to_string()),
        _ => Some("other".to_string()),
    }
}

// 启动UDP广播服务
async fn start_broadcast_service(server_addr: String, server_port: u16) -> Result<(), String> {
    let socket = UdpSocket::bind("0.0.0.0:0")
        .await
        .map_err(|e| format!("Failed to bind UDP socket: {}", e))?;
    
    // 启用广播
    socket.set_broadcast(true)
        .map_err(|e| format!("Failed to set broadcast option: {}", e))?;
    
    // 设置广播地址
    let broadcast_addr = format!("255.255.255.255:{}", BROADCAST_PORT);
    
    // 创建广播消息
    let broadcast_msg = BroadcastMessage {
        message_type: SERVICE_DISCOVERY_MSG.to_string(),
        server_address: server_addr.clone(),
        server_port,
        server_name: format!("Service Discovery Server ({})", server_addr),
        timestamp: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64, // 使用毫秒时间戳
    };
    
    let broadcast_json = serde_json::to_string(&broadcast_msg)
        .map_err(|e| format!("Failed to serialize broadcast message: {}", e))?;
    
    println!("Starting broadcast service for server: {}", server_addr);
    
    // 定期发送广播
    tokio::spawn(async move {
        loop {
            match socket.send_to(broadcast_json.as_bytes(), &broadcast_addr).await {
                Ok(_) => println!("Broadcast message sent to {}", broadcast_addr),
                Err(e) => eprintln!("Failed to send broadcast message: {}", e),
            }
            sleep(BROADCAST_INTERVAL).await;
        }
    });
    
    Ok(())
}

// 启动UDP监听服务 (发现其他服务)
async fn start_discovery_service() -> Result<mpsc::Receiver<BroadcastMessage>, String> {
    let socket = UdpSocket::bind(format!("0.0.0.0:{}", BROADCAST_PORT))
        .await
        .map_err(|e| format!("Failed to bind discovery socket: {}", e))?;
    
    // 创建通道用于发送发现的服务
    let (tx, rx) = mpsc::channel::<BroadcastMessage>(32);
    
    // 启动监听任务
    tokio::spawn(async move {
        let mut buf = [0u8; 1024];
        
        loop {
            match socket.recv_from(&mut buf).await {
                Ok((size, _addr)) => {
                    if let Ok(json_str) = std::str::from_utf8(&buf[..size]) {
                        if let Ok(message) = serde_json::from_str::<BroadcastMessage>(json_str) {
                            if message.message_type == SERVICE_DISCOVERY_MSG {
                                println!("Discovered service: {} at {}:{}", 
                                         message.server_name, message.server_address, message.server_port);
                                
                                // 发送到接收方
                                if tx.try_send(message).is_err() {
                                    eprintln!("Failed to send discovery message to channel");
                                }
                            }
                        }
                    }
                },
                Err(e) => eprintln!("Failed to receive discovery message: {}", e),
            }
        }
    });
    
    Ok(rx)
}

// 处理WebSocket连接
async fn handle_connection(state: Arc<AppState>, socket: TcpStream, addr: SocketAddr) {
    println!("WebSocket connection established: {}", addr);

    // 升级连接为WebSocket
    let ws_stream = tokio_tungstenite::accept_async(socket)
        .await
        .expect("Error during WebSocket handshake");

    // 拆分读写流
    let (ws_sender, mut ws_receiver) = ws_stream.split();
    
    // 创建websocket发送器
    let ws_sender = Arc::new(tokio::sync::Mutex::new(ws_sender));

    // 为该客户端创建一个消息通道
    let (tx, mut rx) = mpsc::channel::<Message>(100);

    // 将发送端存储在共享状态中
    state.connected_clients.lock().unwrap().insert(addr, tx);

    // 发送连接成功消息
    let connect_msg = serde_json::to_string(&serde_json::json!({
        "event": "connected",
        "message": "Connection established"
    })).unwrap();
    ws_sender.lock().await.send(Message::Text(connect_msg)).await.unwrap();

    // 处理从服务器到客户端的消息
    let ws_sender_clone = ws_sender.clone();
    tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            if ws_sender_clone.lock().await.send(msg).await.is_err() {
                break;
            }
        }
    });

    // 处理从客户端到服务器的消息
    while let Some(result) = ws_receiver.next().await {
        match result {
            Ok(msg) => {
                match msg {
                    Message::Text(text) => {
                        if let Ok(request) = serde_json::from_str::<serde_json::Value>(&text) {
                            if let Some(action) = request.get("action").and_then(|a| a.as_str()) {
                                match action {
                                    "getSharedItems" => {
                                        // 发送共享项列表
                                        let items = state.shared_items.lock().await.clone();
                                        let response = serde_json::to_string(&serde_json::json!({
                                            "sharedItems": items
                                        })).unwrap();
                                        ws_sender.lock().await.send(Message::Text(response)).await.unwrap();
                                    }
                                    "shareFile" => {
                                        // 处理文件分享
                                        if let Some(path) = request.get("path").and_then(|p| p.as_str()) {
                                            // 检查文件是否存在
                                            if !std::path::Path::new(path).exists() {
                                                let error_response = serde_json::to_string(&serde_json::json!({
                                                    "error": "File not found"
                                                })).unwrap();
                                                ws_sender.lock().await.send(Message::Text(error_response)).await.unwrap();
                                                continue;
                                            }

                                            // 获取文件基本信息
                                            let metadata = match std::fs::metadata(path) {
                                                Ok(meta) => meta,
                                                Err(_) => {
                                                    let error_response = serde_json::to_string(&serde_json::json!({
                                                        "error": "Failed to read file metadata"
                                                    })).unwrap();
                                                    ws_sender.lock().await.send(Message::Text(error_response)).await.unwrap();
                                                    continue;
                                                }
                                            };

                                            let file_name = std::path::Path::new(path)
                                                .file_name()
                                                .and_then(|n| n.to_str())
                                                .unwrap_or("unknown");

                                            let timestamp = std::time::SystemTime::now()
                                                .duration_since(std::time::UNIX_EPOCH)
                                                .unwrap_or_default()
                                                .as_millis() as u64;

                                            // 创建新的共享项
                                            let new_item = SharedItem {
                                                id: Uuid::new_v4().to_string(),
                                                name: file_name.to_string(),
                                                item_type: "file".to_string(),
                                                content: String::new(), // 文件内容按需加载
                                                path: Some(path.to_string()),
                                                username: format!("User {}", addr),
                                                upload_time: timestamp,
                                                size: Some(metadata.len()),
                                                file_type: get_file_type(path),
                                            };

                                            // 添加到共享列表
                                            state.shared_items.lock().await.push(new_item.clone());

                                            // 通知所有客户端
                                            let notification = serde_json::to_string(&serde_json::json!({
                                                "itemAdded": new_item
                                            })).unwrap();

                                            for (client_addr, client_tx) in state.connected_clients.lock().unwrap().iter() {
                                                if *client_addr != addr {  // 不发送给自己
                                                    if client_tx.try_send(Message::Text(notification.clone())).is_err() {
                                                        println!("Failed to notify client {}", client_addr);
                                                    }
                                                }
                                            }

                                            // 发送成功响应给发送者
                                            let response = serde_json::to_string(&serde_json::json!({
                                                "status": "success",
                                                "message": "File shared successfully",
                                                "itemAdded": new_item
                                            })).unwrap();
                                            ws_sender.lock().await.send(Message::Text(response)).await.unwrap();
                                        }
                                    }
                                    "shareText" => {
                                        // 处理文本分享
                                        if let Some(content) = request.get("content").and_then(|c| c.as_str()) {
                                            if !content.trim().is_empty() {
                                                // 创建新的共享项
                                                let id = Uuid::new_v4().to_string();
                                                let timestamp = std::time::SystemTime::now()
                                                    .duration_since(std::time::UNIX_EPOCH)
                                                    .unwrap_or_default()
                                                    .as_millis() as u64; // 使用毫秒时间戳
                                                
                                                let new_item = SharedItem {
                                                    id: id.clone(),
                                                    name: format!("Text {}", timestamp),
                                                    item_type: "text".to_string(),
                                                    content: content.to_string(),
                                                    path: None,              // 文本类型没有文件路径
                                                    username: format!("User {}", addr),
                                                    upload_time: timestamp,
                                                    size: None,              // 文本类型没有文件大小
                                                    file_type: None,         // 文本类型没有文件类型
                                                };
                                                
                                                // 添加到共享列表
                                                state.shared_items.lock().await.push(new_item.clone());
                                                
                                                // 通知所有客户端
                                                let notification = serde_json::to_string(&serde_json::json!({
                                                    "itemAdded": new_item
                                                })).unwrap();
                                                
                                                for (client_addr, client_tx) in state.connected_clients.lock().unwrap().iter() {
                                                    if *client_addr != addr {  // 不发送给自己
                                                        if client_tx.try_send(Message::Text(notification.clone())).is_err() {
                                                            println!("Failed to notify client {}", client_addr);
                                                        }
                                                    }
                                                }
                                                
                                                // 发送成功响应给发送者
                                                let response = serde_json::to_string(&serde_json::json!({
                                                    "status": "success",
                                                    "message": "Text shared successfully",
                                                    "itemAdded": new_item
                                                })).unwrap();
                                                ws_sender.lock().await.send(Message::Text(response)).await.unwrap();
                                            }
                                        }
                                    }
                                    "deleteSharedItem" => {
                                        // 处理删除共享项
                                        if let Some(id) = request.get("id").and_then(|id| id.as_str()) {
                                            let mut items = state.shared_items.lock().await;
                                            let initial_len = items.len();
                                            items.retain(|item| item.id != id);
                                            
                                            if items.len() < initial_len {
                                                // 有项目被删除，通知所有客户端
                                                let notification = serde_json::to_string(&serde_json::json!({
                                                    "itemRemoved": id
                                                })).unwrap();
                                                
                                                for (client_addr, client_tx) in state.connected_clients.lock().unwrap().iter() {
                                                    if client_tx.try_send(Message::Text(notification.clone())).is_err() {
                                                        println!("Failed to notify client {}", client_addr);
                                                    }
                                                }
                                                
                                                // 发送成功响应给发送者
                                                let response = serde_json::to_string(&serde_json::json!({
                                                    "status": "success",
                                                    "message": "Item deleted successfully"
                                                })).unwrap();
                                                ws_sender.lock().await.send(Message::Text(response)).await.unwrap();
                                            }
                                        }
                                    }
                                    "getItemContent" => {
                                        // 获取项目内容
                                        if let Some(id) = request.get("id").and_then(|id| id.as_str()) {
                                            let items = state.shared_items.lock().await;
                                            if let Some(item) = items.iter().find(|item| item.id == id) {
                                                let content = if item.item_type == "text" {
                                                    item.content.clone()
                                                } else if let Some(path) = &item.path {
                                                    // 读取文件内容
                                                    match std::fs::read_to_string(path) {
                                                        Ok(content) => content,
                                                        Err(_) => "Failed to read file content".to_string(),
                                                    }
                                                } else {
                                                    "No content available".to_string()
                                                };

                                                let response = serde_json::to_string(&serde_json::json!({
                                                    "id": id,
                                                    "content": content
                                                })).unwrap();
                                                ws_sender.lock().await.send(Message::Text(response)).await.unwrap();
                                            }
                                        }
                                    }
                                    _ => {
                                        // 未知动作
                                        println!("Unknown action: {}", action);
                                        let response = serde_json::to_string(&serde_json::json!({
                                            "error": format!("Unknown action: {}", action)
                                        })).unwrap();
                                        ws_sender.lock().await.send(Message::Text(response)).await.unwrap();
                                    }
                                }
                            }
                        } else {
                            // 无法解析JSON
                            println!("Invalid JSON received: {}", text);
                        }
                    }
                    Message::Close(_) => break,
                    _ => {}
                }
            }
            Err(_) => break,
        }
    }

    // 连接断开，从状态中移除客户端
    state.connected_clients.lock().unwrap().remove(&addr);
    println!("WebSocket connection closed: {}", addr);
}

// 启动WebSocket服务器
async fn start_websocket_server(state: Arc<AppState>, port: u16) -> Result<String, String> {
    let ip = local_ip().map_err(|e| format!("Failed to get local IP address: {}", e))?;
    let addr = format!("{}:{}", ip, port);

    // 绑定地址
    let listener = TcpListener::bind(&addr)
        .await
        .map_err(|e| format!("Failed to bind WebSocket server: {}", e))?;
    println!("WebSocket server started: {}", addr);

    // 启动广播服务
    if let Err(e) = start_broadcast_service(ip.to_string(), port).await {
        eprintln!("Failed to start broadcast service: {}", e);
    }

    // 处理连接
    tokio::spawn(async move {
        while let Ok((stream, addr)) = listener.accept().await {
            let state_clone = Arc::clone(&state);
            tokio::spawn(async move {
                handle_connection(state_clone, stream, addr).await;
            });
        }
    });

    Ok(addr)
}

// Tauri命令：获取WebSocket服务器地址
#[tauri::command]
fn get_server_address(state: State<'_, String>) -> String {
    state.inner().clone()
}

// Tauri命令：发现网络上的服务
#[tauri::command]
async fn discover_services() -> Result<Vec<BroadcastMessage>, String> {
    let (tx, mut rx) = mpsc::channel::<BroadcastMessage>(32);
    let mut discovered_services = Vec::new();
    
    // 获取本机IP和端口
    let local_ip = local_ip().map_err(|e| format!("Failed to get local IP: {}", e))?;
    let local_port = 8080; // 使用默认端口
    
    // 添加本机服务到发现列表
    let local_service = BroadcastMessage {
        message_type: SERVICE_DISCOVERY_MSG.to_string(),
        server_address: local_ip.to_string(),
        server_port: local_port,
        server_name: format!("Local Service ({})", local_ip),
        timestamp: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64,
    };
    discovered_services.push(local_service);
    
    // 创建一个临时 UDP 套接字发送广播消息
    let socket = match UdpSocket::bind("0.0.0.0:0").await {
        Ok(s) => s,
        Err(e) => return Err(format!("Failed to bind UDP socket: {}", e)),
    };
    
    // 设置广播选项
    if let Err(e) = socket.set_broadcast(true) {
        return Err(format!("Failed to set broadcast option: {}", e));
    }
    
    // 广播发现请求
    let broadcast_addr = format!("255.255.255.255:{}", BROADCAST_PORT);
    let discovery_request = BroadcastMessage {
        message_type: "DISCOVERY_REQUEST".to_string(),
        server_address: local_ip.to_string(),
        server_port: local_port,
        server_name: format!("Local Service ({})", local_ip),
        timestamp: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64,
    };
    
    let discovery_json = match serde_json::to_string(&discovery_request) {
        Ok(json) => json,
        Err(e) => return Err(format!("Failed to serialize discovery request: {}", e)),
    };
    
    // 发送广播消息
    if let Err(e) = socket.send_to(discovery_json.as_bytes(), &broadcast_addr).await {
        return Err(format!("Failed to send discovery request: {}", e));
    }
    
    // 监听回复 (最多等待 3 秒)
    let timeout = tokio::time::timeout(Duration::from_secs(3), async {
        let mut buf = [0u8; 1024];
        
        loop {
            match socket.recv_from(&mut buf).await {
                Ok((size, addr)) => {
                    if let Ok(json_str) = std::str::from_utf8(&buf[..size]) {
                        if let Ok(message) = serde_json::from_str::<BroadcastMessage>(json_str) {
                            if message.message_type == SERVICE_DISCOVERY_MSG {
                                // 不添加来自自己的响应
                                if message.server_address != local_ip.to_string() || 
                                   message.server_port != local_port {
                                    if tx.send(message).await.is_err() {
                                        break;
                                    }
                                }
                            }
                        }
                    }
                },
                Err(_) => break,
            }
        }
    }).await;
    
    // 收集发现的服务
    if timeout.is_ok() {
        while let Ok(service) = rx.try_recv() {
            // 检查是否已经发现相同的服务
            if !discovered_services.iter().any(|s: &BroadcastMessage| {
                s.server_address == service.server_address && s.server_port == service.server_port
            }) {
                discovered_services.push(service);
            }
        }
    }
    
    Ok(discovered_services)
}

// Tauri命令：获取问候消息
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let rt = tokio::runtime::Runtime::new().unwrap();

    // 初始化应用状态
    let state = Arc::new(AppState {
        connected_clients: Mutex::new(HashMap::new()),
        shared_items: TokioMutex::new(Vec::new()),
    });

    // 启动WebSocket服务器
    let server_addr = rt.block_on(async {
        match start_websocket_server(state.clone(), 8080).await {
            Ok(addr) => addr,
            Err(e) => {
                eprintln!("Failed to start WebSocket server: {}", e);
                "unknown".to_string()
            }
        }
    });

    // 启动服务发现监听器
    rt.block_on(async {
        match start_discovery_service().await {
            Ok(_) => println!("Service discovery listener started"),
            Err(e) => eprintln!("Failed to start service discovery: {}", e),
        }
    });

    tauri::Builder::default()
        // 修复 single-instance 插件初始化，添加回调函数
        .plugin(tauri_plugin_single_instance::init(|_app_handle: &AppHandle, argv: Vec<String>, cwd: String| {
            println!("Another instance tried to launch with args: {:?}, cwd: {}", argv, cwd);
            // 移除不支持的方法调用
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
