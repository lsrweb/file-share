// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
use futures_util::{SinkExt, StreamExt};
use local_ip_address::local_ip;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use tauri::{AppHandle, State};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{mpsc, Mutex as TokioMutex}; // 添加 Tokio 的 Mutex
use tokio_tungstenite::tungstenite::Message;
use walkdir::WalkDir;

// 共享状态结构体 - 修改为使用 TokioMutex
struct AppState {
    connected_clients: Mutex<HashMap<SocketAddr, mpsc::Sender<Message>>>,
    shared_dir: TokioMutex<PathBuf>, // 改为 TokioMutex 以便在异步代码中安全使用
}

// 文件信息结构体
#[derive(Serialize, Deserialize, Clone)]
struct FileInfo {
    name: String,
    path: String,
    is_dir: bool,
}

// 文件列表响应
#[derive(Serialize, Deserialize)]
struct FileListResponse {
    files: Vec<FileInfo>,
}

// 文件内容响应
#[derive(Serialize, Deserialize)]
struct FileContentResponse {
    content: String,
    path: String,
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
    
    // 用 TokioMutex 包装 ws_sender，这样它可以在多个任务间共享
    let ws_sender = Arc::new(TokioMutex::new(ws_sender));

    // 为该客户端创建一个消息通道
    let (tx, mut rx) = mpsc::channel::<Message>(100);

    // 将发送端存储在共享状态中
    state.connected_clients.lock().unwrap().insert(addr, tx);

    // 获取文件列表并发送
    let file_list = get_file_list(&state.shared_dir.lock().await).await;
    let response = serde_json::to_string(&FileListResponse { files: file_list }).unwrap();
    ws_sender.lock().await.send(Message::Text(response)).await.unwrap();

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
                        // 解析客户端请求
                        if let Ok(request) = serde_json::from_str::<serde_json::Value>(&text) {
                            if let Some(action) = request.get("action").and_then(|a| a.as_str()) {
                                match action {
                                    "getFileList" => {
                                        let file_list = get_file_list(&state.shared_dir.lock().await).await;
                                        let response = serde_json::to_string(&FileListResponse {
                                            files: file_list,
                                        }).unwrap();
                                        ws_sender.lock().await.send(Message::Text(response)).await.unwrap();
                                    }
                                    "getFileContent" => {
                                        if let Some(path_str) = request.get("path").and_then(|p| p.as_str()) {
                                            // 获取共享目录路径，但不要持有锁跨越 await 点
                                            let full_path = {
                                                let shared_dir = state.shared_dir.lock().await;
                                                shared_dir.join(path_str)
                                            };
                                            
                                            // 获取规范路径
                                            match std::fs::canonicalize(&full_path) {
                                                Ok(canonical_path) => {
                                                    // 重新获取锁来检查路径
                                                    let is_valid = {
                                                        let shared_dir = state.shared_dir.lock().await;
                                                        canonical_path.starts_with(&*shared_dir) && canonical_path.is_file()
                                                    };
                                                    
                                                    if is_valid {
                                                        match std::fs::read_to_string(&canonical_path) {
                                                            Ok(content) => {
                                                                let response = serde_json::to_string(
                                                                    &FileContentResponse {
                                                                        content,
                                                                        path: path_str.to_string(),
                                                                    },
                                                                ).unwrap();
                                                                ws_sender.lock().await.send(Message::Text(response)).await.unwrap();
                                                            }
                                                            Err(e) => {
                                                                let error_msg = format!("{{\"error\": \"无法读取文件: {}\"}}", e);
                                                                ws_sender.lock().await.send(Message::Text(error_msg)).await.unwrap();
                                                            }
                                                        }
                                                    } else {
                                                        let error_msg = "{\"error\": \"请求的文件不在共享目录中或不是文件\"}".to_string();
                                                        ws_sender.lock().await.send(Message::Text(error_msg)).await.unwrap();
                                                    }
                                                }
                                                Err(_) => {
                                                    let error_msg = "{\"error\": \"无效的文件路径\"}".to_string();
                                                    ws_sender.lock().await.send(Message::Text(error_msg)).await.unwrap();
                                                }
                                            }
                                        }
                                    }
                                    _ => {
                                        let error_msg = "{\"error\": \"未知的操作\"}".to_string();
                                        ws_sender.lock().await.send(Message::Text(error_msg)).await.unwrap();
                                    }
                                }
                            }
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

// 获取目录中的文件列表 - 改为异步函数
async fn get_file_list(dir: &Path) -> Vec<FileInfo> {
    let mut files = Vec::new();

    for entry in WalkDir::new(dir)
        .max_depth(1)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        if entry.path() == dir {
            continue;
        }

        let relative_path = entry.path().strip_prefix(dir).unwrap_or(entry.path());
        let path_string = relative_path.to_string_lossy().into_owned();

        files.push(FileInfo {
            name: entry.file_name().to_string_lossy().into_owned(),
            path: path_string,
            is_dir: entry.file_type().is_dir(),
        });
    }

    files
}

// 启动WebSocket服务器
async fn start_websocket_server(state: Arc<AppState>, port: u16) -> Result<String, String> {
    let ip = local_ip().map_err(|e| format!("获取本地IP地址失败: {}", e))?;
    let addr = format!("{}:{}", ip, port);

    // 绑定地址
    let listener = TcpListener::bind(&addr)
        .await
        .map_err(|e| format!("绑定WebSocket服务器失败: {}", e))?;
    println!("WebSocket服务器已启动: {}", addr);

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

// Tauri命令：设置共享目录
#[tauri::command]
async fn set_shared_dir(path: String, state: State<'_, Arc<AppState>>) -> Result<String, String> {
    let path = PathBuf::from(path);

    if !path.exists() {
        return Err("目录不存在".into());
    }

    if !path.is_dir() {
        return Err("所选路径不是目录".into());
    }

    // 更新共享目录
    *state.shared_dir.lock().await = path;

    // 向所有连接的客户端发送更新
    let file_list = get_file_list(&state.shared_dir.lock().await).await;
    let response = serde_json::to_string(&FileListResponse { files: file_list }).unwrap();

    for (_, tx) in state.connected_clients.lock().unwrap().iter() {
        let _ = tx.try_send(Message::Text(response.clone()));
    }

    Ok("已成功设置共享目录".into())
}

// Tauri命令：获取WebSocket服务器地址
#[tauri::command]
fn get_server_address(state: State<'_, String>) -> String {
    state.inner().clone()
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
        shared_dir: TokioMutex::new(PathBuf::from(".")), // 使用 TokioMutex
    });

    // 启动WebSocket服务器
    let server_addr = rt.block_on(async {
        match start_websocket_server(state.clone(), 8080).await {
            Ok(addr) => addr,
            Err(e) => {
                eprintln!("启动WebSocket服务器失败: {}", e);
                "未知".to_string()
            }
        }
    });

    tauri::Builder::default()
        // 修复 single-instance 插件初始化，添加回调函数
        .plugin(tauri_plugin_single_instance::init(|app_handle: &AppHandle, argv: Vec<String>, cwd: String| {
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
            set_shared_dir,
            get_server_address
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
