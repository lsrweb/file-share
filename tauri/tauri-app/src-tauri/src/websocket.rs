use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::mpsc;
use tokio_tungstenite::tungstenite::Message;
use futures_util::{StreamExt, SinkExt};
use local_ip_address::local_ip;

use crate::state::AppState;
use crate::handlers;
use crate::broadcast;

// 启动WebSocket服务器
pub async fn start_websocket_server(state: Arc<AppState>, port: u16) -> Result<String, String> {
    let ip = local_ip().map_err(|e| format!("Failed to get local IP address: {}", e))?;
    let addr = format!("{}:{}", ip, port);

    // 绑定地址
    let listener = TcpListener::bind(&addr)
        .await
        .map_err(|e| format!("Failed to bind WebSocket server: {}", e))?;
    println!("WebSocket server started: {}", addr);

    // 启动广播服务
    if let Err(e) = broadcast::start_broadcast_service(ip.to_string(), port).await {
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

// 处理WebSocket连接
async fn handle_connection(state: Arc<AppState>, socket: TcpStream, addr: SocketAddr) {
    println!("WebSocket connection established: {}", addr);

    // 升级连接为WebSocket
    let ws_stream = match tokio_tungstenite::accept_async(socket).await {
        Ok(stream) => stream,
        Err(e) => {
            eprintln!("Error during WebSocket handshake: {}", e);
            return;
        }
    };

    // 拆分读写流
    let (ws_sender, mut ws_receiver) = ws_stream.split();
    let ws_sender = Arc::new(tokio::sync::Mutex::new(ws_sender));

    // 为该客户端创建一个消息通道
    let (tx, mut rx) = mpsc::channel::<Message>(100);
    state.connected_clients.lock().unwrap().insert(addr, tx);

    // 发送连接成功消息
    let connect_msg = serde_json::json!({
        "event": "connected",
        "message": "Connection established"
    }).to_string();
    
    if let Err(e) = ws_sender.lock().await.send(Message::Text(connect_msg)).await {
        eprintln!("Error sending connection message: {}", e);
        return;
    }

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
                if let Message::Text(text) = msg {
                    if let Ok(request) = serde_json::from_str::<serde_json::Value>(&text) {
                        if let Some(action) = request.get("action").and_then(|a| a.as_str()) {
                            match action {
                                "getSharedItems" => {
                                    if let Err(e) = handlers::handle_get_shared_items(
                                        state.clone(),
                                        ws_sender.clone(),
                                    ).await {
                                        eprintln!("Error handling getSharedItems: {}", e);
                                    }
                                }
                                "shareFile" => {
                                    if let Some(path) = request.get("path").and_then(|p| p.as_str()) {
                                        if let Err(e) = handlers::handle_share_file(
                                            state.clone(),
                                            ws_sender.clone(),
                                            path,
                                            addr,
                                        ).await {
                                            eprintln!("Error handling shareFile: {}", e);
                                            let error_response = serde_json::json!({
                                                "error": e
                                            }).to_string();
                                            if let Err(e) = ws_sender.lock().await.send(Message::Text(error_response)).await {
                                                eprintln!("Error sending error response: {}", e);
                                            }
                                        }
                                    }
                                }
                                "shareText" => {
                                    if let Some(content) = request.get("content").and_then(|c| c.as_str()) {
                                        if let Err(e) = handlers::handle_share_text(
                                            state.clone(),
                                            ws_sender.clone(),
                                            content,
                                            addr,
                                        ).await {
                                            eprintln!("Error handling shareText: {}", e);
                                            let error_response = serde_json::json!({
                                                "error": e
                                            }).to_string();
                                            if let Err(e) = ws_sender.lock().await.send(Message::Text(error_response)).await {
                                                eprintln!("Error sending error response: {}", e);
                                            }
                                        }
                                    }
                                }
                                "deleteSharedItem" => {
                                    if let Some(id) = request.get("id").and_then(|id| id.as_str()) {
                                        if let Err(e) = handlers::handle_delete_shared_item(
                                            state.clone(),
                                            ws_sender.clone(),
                                            id,
                                        ).await {
                                            eprintln!("Error handling deleteSharedItem: {}", e);
                                        }
                                    }
                                }
                                "getItemContent" => {
                                    if let Some(id) = request.get("id").and_then(|id| id.as_str()) {
                                        if let Err(e) = handlers::handle_get_item_content(
                                            state.clone(),
                                            ws_sender.clone(),
                                            id,
                                        ).await {
                                            eprintln!("Error handling getItemContent: {}", e);
                                        }
                                    }
                                }
                                _ => {
                                    let error_response = serde_json::json!({
                                        "error": format!("Unknown action: {}", action)
                                    }).to_string();
                                    if let Err(e) = ws_sender.lock().await.send(Message::Text(error_response)).await {
                                        eprintln!("Error sending error response: {}", e);
                                    }
                                }
                            }
                        }
                    }
                }
            }
            Err(e) => {
                eprintln!("Error receiving message: {}", e);
                break;
            }
        }
    }

    // 连接断开，从状态中移除客户端
    state.connected_clients.lock().unwrap().remove(&addr);
    println!("WebSocket connection closed: {}", addr);
}