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
    println!("[WebSocket][INFO] WebSocket server started: {}", addr);

    // 启动广播服务
    if let Err(e) = broadcast::start_broadcast_service(ip.to_string(), port).await {
        eprintln!("[WebSocket][ERROR] Failed to start broadcast service: {}", e);
    } else {
        println!("[WebSocket][INFO] Broadcast service started successfully");
    }

    // 处理连接
    tokio::spawn(async move {
        println!("[WebSocket][INFO] WebSocket server is listening for connections");
        while let Ok((stream, addr)) = listener.accept().await {
            println!("[WebSocket][INFO] New TCP connection: {}", addr);
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
    println!("[WebSocket][INFO] Attempting to establish WebSocket connection: {}", addr);

    // 升级连接为WebSocket
    let ws_stream = match tokio_tungstenite::accept_async(socket).await {
        Ok(stream) => {
            println!("[WebSocket][INFO] WebSocket handshake successful: {}", addr);
            stream
        },
        Err(e) => {
            eprintln!("[WebSocket][ERROR] Error during WebSocket handshake: {}", e);
            return;
        }
    };

    // 拆分读写流
    let (ws_sender, mut ws_receiver) = ws_stream.split();
    let ws_sender = Arc::new(tokio::sync::Mutex::new(ws_sender));

    // 为该客户端创建一个消息通道
    let (tx, mut rx) = mpsc::channel::<Message>(100);
    {
        let mut clients = state.connected_clients.lock().unwrap();
        clients.insert(addr, tx);
        println!("[WebSocket][INFO] Client registered, total clients: {}", clients.len());
    }

    // 发送连接成功消息
    let connect_msg = serde_json::json!({
        "event": "connected",
        "message": "Connection established"
    }).to_string();
    
    println!("[WebSocket][DEBUG] Sending connection established message to client: {}", addr);
    if let Err(e) = ws_sender.lock().await.send(Message::Text(connect_msg)).await {
        eprintln!("[WebSocket][ERROR] Error sending connection message: {}", e);
        return;
    }
    println!("[WebSocket][DEBUG] Connection message sent successfully to: {}", addr);

    // 处理从服务器到客户端的消息
    let ws_sender_clone = ws_sender.clone();
    tokio::spawn(async move {
        println!("[WebSocket][DEBUG] Starting server-to-client message handler for: {}", addr);
        while let Some(msg) = rx.recv().await {
            println!("[WebSocket][DEBUG] Sending message to client {}: {:?}", addr, msg);
            if ws_sender_clone.lock().await.send(msg).await.is_err() {
                eprintln!("[WebSocket][ERROR] Failed to send message to client: {}", addr);
                break;
            }
            println!("[WebSocket][DEBUG] Message sent successfully to client: {}", addr);
        }
        println!("[WebSocket][INFO] Server-to-client message handler terminated for: {}", addr);
    });

    // 处理从客户端到服务器的消息
    println!("[WebSocket][DEBUG] Starting client-to-server message handler for: {}", addr);
    while let Some(result) = ws_receiver.next().await {
        match result {
            Ok(msg) => {
                println!("[WebSocket][DEBUG] Received message from {}: {:?}", addr, msg);
                match msg {
                    // 处理ping消息，立即回复pong保持连接活跃
                    Message::Ping(data) => {
                        println!("[WebSocket][DEBUG] Received Ping from client: {}", addr);
                        if let Err(e) = ws_sender.lock().await.send(Message::Pong(data)).await {
                            eprintln!("[WebSocket][ERROR] Error sending pong response: {}", e);
                        } else {
                            println!("[WebSocket][DEBUG] Pong sent to client: {}", addr);
                        }
                    },
                    // 添加对客户端ping/pong协议的支持
                    Message::Text(text) => {
                        println!("[WebSocket][DEBUG] Received text message from {}: {}", addr, text);
                        if let Ok(request) = serde_json::from_str::<serde_json::Value>(&text) {
                            println!("[WebSocket][DEBUG] Parsed JSON request: {:?}", request);
                            if let Some(action) = request.get("action").and_then(|a| a.as_str()) {
                                println!("[WebSocket][DEBUG] Processing action: {}", action);
                                match action {
                                    // 处理客户端的ping请求
                                    "ping" => {
                                        println!("[WebSocket][DEBUG] Handling ping request from: {}", addr);
                                        let pong_response = serde_json::json!({
                                            "action": "pong",
                                            "timestamp": request.get("timestamp").and_then(|t| t.as_u64()).unwrap_or(0)
                                        }).to_string();
                                        if let Err(e) = ws_sender.lock().await.send(Message::Text(pong_response)).await {
                                            eprintln!("[WebSocket][ERROR] Error sending pong response: {}", e);
                                        } else {
                                            println!("[WebSocket][DEBUG] Pong response sent to client: {}", addr);
                                        }
                                    },
                                    "getSharedItems" => {
                                        println!("[WebSocket][DEBUG] Handling getSharedItems request from: {}", addr);
                                        if let Err(e) = handlers::handle_get_shared_items(
                                            state.clone(),
                                            ws_sender.clone(),
                                        ).await {
                                            eprintln!("[WebSocket][ERROR] Error handling getSharedItems: {}", e);
                                        } else {
                                            println!("[WebSocket][DEBUG] getSharedItems handled successfully for: {}", addr);
                                        }
                                    },
                                    "shareFile" => {
                                        if let Some(path) = request.get("path").and_then(|p| p.as_str()) {
                                            println!("[WebSocket][DEBUG] Handling shareFile request for path: {} from: {}", path, addr);
                                            if let Err(e) = handlers::handle_share_file(
                                                state.clone(),
                                                ws_sender.clone(),
                                                path,
                                                addr,
                                            ).await {
                                                eprintln!("[WebSocket][ERROR] Error handling shareFile: {}", e);
                                                let error_response = serde_json::json!({
                                                    "error": e
                                                }).to_string();
                                                if let Err(e) = ws_sender.lock().await.send(Message::Text(error_response)).await {
                                                    eprintln!("[WebSocket][ERROR] Error sending error response: {}", e);
                                                }
                                            } else {
                                                println!("[WebSocket][DEBUG] shareFile handled successfully for: {}", addr);
                                            }
                                        } else {
                                            eprintln!("[WebSocket][ERROR] Missing path in shareFile request from: {}", addr);
                                        }
                                    },
                                    "shareText" => {
                                        if let Some(content) = request.get("content").and_then(|c| c.as_str()) {
                                            println!("[WebSocket][DEBUG] Handling shareText request with content length: {} from: {}", content.len(), addr);
                                            if let Err(e) = handlers::handle_share_text(
                                                state.clone(),
                                                ws_sender.clone(),
                                                content,
                                                addr,
                                            ).await {
                                                eprintln!("[WebSocket][ERROR] Error handling shareText: {}", e);
                                                let error_response = serde_json::json!({
                                                    "error": e
                                                }).to_string();
                                                if let Err(e) = ws_sender.lock().await.send(Message::Text(error_response)).await {
                                                    eprintln!("[WebSocket][ERROR] Error sending error response: {}", e);
                                                }
                                            } else {
                                                println!("[WebSocket][DEBUG] Successfully shared text from client: {}", addr);
                                            }
                                        } else {
                                            eprintln!("[WebSocket][ERROR] Missing content field in shareText request from: {}", addr);
                                            let error_response = serde_json::json!({
                                                "error": "Missing content field in request"
                                            }).to_string();
                                            if let Err(e) = ws_sender.lock().await.send(Message::Text(error_response)).await {
                                                eprintln!("[WebSocket][ERROR] Error sending error response: {}", e);
                                            }
                                        }
                                    },
                                    "deleteSharedItem" => {
                                        if let Some(id) = request.get("id").and_then(|id| id.as_str()) {
                                            println!("[WebSocket][DEBUG] Handling deleteSharedItem request for id: {} from: {}", id, addr);
                                            if let Err(e) = handlers::handle_delete_shared_item(
                                                state.clone(),
                                                ws_sender.clone(),
                                                id,
                                            ).await {
                                                eprintln!("[WebSocket][ERROR] Error handling deleteSharedItem: {}", e);
                                            } else {
                                                println!("[WebSocket][DEBUG] deleteSharedItem handled successfully for: {}", addr);
                                            }
                                        } else {
                                            eprintln!("[WebSocket][ERROR] Missing id in deleteSharedItem request from: {}", addr);
                                        }
                                    },
                                    "getItemContent" => {
                                        if let Some(id) = request.get("id").and_then(|id| id.as_str()) {
                                            println!("[WebSocket][DEBUG] Handling getItemContent request for id: {} from: {}", id, addr);
                                            if let Err(e) = handlers::handle_get_item_content(
                                                state.clone(),
                                                ws_sender.clone(),
                                                id,
                                            ).await {
                                                eprintln!("[WebSocket][ERROR] Error handling getItemContent: {}", e);
                                            } else {
                                                println!("[WebSocket][DEBUG] getItemContent handled successfully for: {}", addr);
                                            }
                                        } else {
                                            eprintln!("[WebSocket][ERROR] Missing id in getItemContent request from: {}", addr);
                                        }
                                    },
                                    _ => {
                                        eprintln!("[WebSocket][ERROR] Unknown action: {} from client: {}", action, addr);
                                        let error_response = serde_json::json!({
                                            "error": format!("Unknown action: {}", action)
                                        }).to_string();
                                        if let Err(e) = ws_sender.lock().await.send(Message::Text(error_response)).await {
                                            eprintln!("[WebSocket][ERROR] Error sending error response: {}", e);
                                        }
                                    }
                                }
                            } else {
                                eprintln!("[WebSocket][ERROR] No action field in request from client: {}, content: {}", addr, text);
                                let error_response = serde_json::json!({
                                    "error": "Missing action field in request"
                                }).to_string();
                                if let Err(e) = ws_sender.lock().await.send(Message::Text(error_response)).await {
                                    eprintln!("[WebSocket][ERROR] Error sending error response: {}", e);
                                }
                            }
                        } else {
                            eprintln!("[WebSocket][ERROR] Invalid JSON message received from client: {}, content: {}", addr, text);
                            let error_response = serde_json::json!({
                                "error": "Invalid JSON format"
                            }).to_string();
                            if let Err(e) = ws_sender.lock().await.send(Message::Text(error_response)).await {
                                eprintln!("[WebSocket][ERROR] Error sending error response: {}", e);
                            }
                        }
                    },
                    // 添加对其他WebSocket消息类型的处理
                    Message::Binary(data) => {
                        println!("[WebSocket][DEBUG] Received binary message of size: {} bytes from: {}", data.len(), addr);
                    },
                    Message::Pong(_) => {
                        println!("[WebSocket][DEBUG] Received Pong from client: {}", addr);
                        // 客户端响应服务器的Ping，无需特殊处理
                    },
                    Message::Close(_) => {
                        println!("[WebSocket][INFO] Received close message from client: {}", addr);
                        break;
                    },
                    _ => {
                        println!("[WebSocket][DEBUG] Received other message type from client: {}", addr);
                        // 处理其他类型的消息
                    }
                }
            },
            Err(e) => {
                eprintln!("[WebSocket][ERROR] Error receiving message from {}: {}", addr, e);
                break;
            }
        }
    }

    // 连接断开，从状态中移除客户端
    {
        let mut clients = state.connected_clients.lock().unwrap();
        clients.remove(&addr);
        println!("[WebSocket][INFO] Client removed, total clients: {}", clients.len());
    }
    println!("[WebSocket][INFO] WebSocket connection closed: {}", addr);
}