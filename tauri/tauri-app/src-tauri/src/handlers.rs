use std::net::SocketAddr;
use std::path::Path;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio_tungstenite::tungstenite::Message;
use uuid::Uuid;
use futures_util::SinkExt;

use crate::models::SharedItem;
use crate::state::AppState;

// 获取文件类型
pub fn get_file_type(path: &str) -> Option<String> {
    let extension = Path::new(path)
        .extension()
        .and_then(|ext| ext.to_str())?
        .to_lowercase();
    
    match extension.as_str() {
        // 图片类型
        "jpg" | "jpeg" | "png" | "gif" | "bmp" | "webp" | "svg" => Some("image".to_string()),
        
        // 视频类型
        "mp4" | "avi" | "mov" | "wmv" | "flv" | "webm" | "mkv" => Some("video".to_string()),
        
        // 音频类型
        "mp3" | "wav" | "ogg" | "m4a" | "aac" | "flac" => Some("audio".to_string()),
        
        // PDF文档
        "pdf" => Some("pdf".to_string()),
        
        // 文本文件
        "txt" | "log" | "csv" | "cfg" | "ini" => Some("text".to_string()),
        
        // 代码文件
        "js" | "ts" | "html" | "css" | "json" | "xml" | "yaml" | "yml" => Some("code".to_string()),
        "py" | "java" | "c" | "cpp" | "h" | "cs" | "go" | "rs" | "php" => Some("code".to_string()),
        
        // Markdown文件
        "md" | "markdown" => Some("markdown".to_string()),
        
        // Office文档
        "doc" | "docx" => Some("document".to_string()),
        "xls" | "xlsx" => Some("spreadsheet".to_string()),
        "ppt" | "pptx" => Some("presentation".to_string()),
        
        // 压缩文件
        "zip" | "rar" | "7z" | "tar" | "gz" => Some("archive".to_string()),
        
        // 其他文件类型
        _ => Some("other".to_string()),
    }
}

// 获取MIME类型
pub fn get_mime_type(path: &str) -> Option<String> {
    let extension = Path::new(path)
        .extension()
        .and_then(|ext| ext.to_str())?
        .to_lowercase();
    
    match extension.as_str() {
        // 图片
        "jpg" | "jpeg" => Some("image/jpeg".to_string()),
        "png" => Some("image/png".to_string()),
        "gif" => Some("image/gif".to_string()),
        "webp" => Some("image/webp".to_string()),
        "svg" => Some("image/svg+xml".to_string()),
        "bmp" => Some("image/bmp".to_string()),
        
        // 视频
        "mp4" => Some("video/mp4".to_string()),
        "webm" => Some("video/webm".to_string()),
        "avi" => Some("video/x-msvideo".to_string()),
        "mov" => Some("video/quicktime".to_string()),
        "mkv" => Some("video/x-matroska".to_string()),
        
        // 音频
        "mp3" => Some("audio/mpeg".to_string()),
        "wav" => Some("audio/wav".to_string()),
        "ogg" => Some("audio/ogg".to_string()),
        "flac" => Some("audio/flac".to_string()),
        "m4a" => Some("audio/mp4".to_string()),
        
        // 文档
        "pdf" => Some("application/pdf".to_string()),
        "doc" => Some("application/msword".to_string()),
        "docx" => Some("application/vnd.openxmlformats-officedocument.wordprocessingml.document".to_string()),
        "xls" => Some("application/vnd.ms-excel".to_string()),
        "xlsx" => Some("application/vnd.openxmlformats-officedocument.spreadsheetml.sheet".to_string()),
        "ppt" => Some("application/vnd.ms-powerpoint".to_string()),
        "pptx" => Some("application/vnd.openxmlformats-officedocument.presentationml.presentation".to_string()),
        
        // 文本
        "txt" => Some("text/plain".to_string()),
        "html" => Some("text/html".to_string()),
        "css" => Some("text/css".to_string()),
        "js" => Some("text/javascript".to_string()),
        "json" => Some("application/json".to_string()),
        "xml" => Some("application/xml".to_string()),
        "md" => Some("text/markdown".to_string()),
        
        // 压缩文件
        "zip" => Some("application/zip".to_string()),
        "rar" => Some("application/x-rar-compressed".to_string()),
        "7z" => Some("application/x-7z-compressed".to_string()),
        "tar" => Some("application/x-tar".to_string()),
        "gz" => Some("application/gzip".to_string()),
        
        // 默认
        _ => Some("application/octet-stream".to_string()),
    }
}

// 处理共享列表请求
pub async fn handle_get_shared_items(
    state: Arc<AppState>,
    ws_sender: Arc<Mutex<futures_util::stream::SplitSink<tokio_tungstenite::WebSocketStream<tokio::net::TcpStream>, Message>>>,
) -> Result<(), String> {
    let items = state.shared_items.lock().await.clone();
    let response = serde_json::to_string(&serde_json::json!({
        "sharedItems": items
    })).map_err(|e| format!("Failed to serialize shared items: {}", e))?;
    
    ws_sender.lock().await.send(Message::Text(response))
        .await
        .map_err(|e| format!("Failed to send shared items: {}", e))?;
    
    Ok(())
}

// 处理文件分享请求
pub async fn handle_share_file(
    state: Arc<AppState>,
    ws_sender: Arc<Mutex<futures_util::stream::SplitSink<tokio_tungstenite::WebSocketStream<tokio::net::TcpStream>, Message>>>,
    path: &str,
    addr: SocketAddr,
) -> Result<(), String> {
    if !Path::new(path).exists() {
        return Err("File not found".to_string());
    }

    let metadata = std::fs::metadata(path)
        .map_err(|_| "Failed to read file metadata".to_string())?;

    let file_name = Path::new(path)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown");

    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64;

    let new_item = SharedItem {
        id: Uuid::new_v4().to_string(),
        name: file_name.to_string(),
        item_type: "file".to_string(),
        content: String::new(),
        path: Some(path.to_string()),
        username: format!("User {}", addr),
        uploadTime: timestamp,
        size: Some(metadata.len()),
        file_type: get_file_type(path),
        mime_type: get_mime_type(path),
        content_type: None,
    };

    state.shared_items.lock().await.push(new_item.clone());

    let notification = serde_json::to_string(&serde_json::json!({
        "itemAdded": new_item
    })).map_err(|e| format!("Failed to serialize notification: {}", e))?;

    // 通知其他客户端
    for (client_addr, client_tx) in state.connected_clients.lock().unwrap().iter() {
        if *client_addr != addr {
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
    })).map_err(|e| format!("Failed to serialize response: {}", e))?;

    ws_sender.lock().await.send(Message::Text(response))
        .await
        .map_err(|e| format!("Failed to send response: {}", e))?;

    Ok(())
}

// 处理文本分享请求
pub async fn handle_share_text(
    state: Arc<AppState>,
    ws_sender: Arc<Mutex<futures_util::stream::SplitSink<tokio_tungstenite::WebSocketStream<tokio::net::TcpStream>, Message>>>,
    content: &str,
    addr: SocketAddr,
) -> Result<(), String> {
    if content.trim().is_empty() {
        return Err("Content is empty".to_string());
    }

    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64;

    // 确定内容类型 (plain 或 markdown)
    let content_type = if content.contains('#') || content.contains('*') || content.contains('`') {
        Some("markdown".to_string())
    } else {
        Some("plain".to_string())
    };

    let new_item = SharedItem {
        id: Uuid::new_v4().to_string(),
        name: format!("Text {}", timestamp),
        item_type: "text".to_string(),
        content: content.to_string(),
        path: None,
        username: format!("User {}", addr),
        uploadTime: timestamp,
        size: None,
        file_type: Some("text".to_string()),
        mime_type: Some("text/plain".to_string()),
        content_type,
    };

    state.shared_items.lock().await.push(new_item.clone());

    let notification = serde_json::to_string(&serde_json::json!({
        "itemAdded": new_item
    })).map_err(|e| format!("Failed to serialize notification: {}", e))?;

    // 通知其他客户端
    for (client_addr, client_tx) in state.connected_clients.lock().unwrap().iter() {
        if *client_addr != addr {
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
    })).map_err(|e| format!("Failed to serialize response: {}", e))?;

    ws_sender.lock().await.send(Message::Text(response))
        .await
        .map_err(|e| format!("Failed to send response: {}", e))?;

    Ok(())
}

// 处理删除共享项请求
pub async fn handle_delete_shared_item(
    state: Arc<AppState>,
    ws_sender: Arc<Mutex<futures_util::stream::SplitSink<tokio_tungstenite::WebSocketStream<tokio::net::TcpStream>, Message>>>,
    id: &str,
) -> Result<(), String> {
    let mut items = state.shared_items.lock().await;
    let initial_len = items.len();
    items.retain(|item| item.id != id);

    if items.len() < initial_len {
        let notification = serde_json::to_string(&serde_json::json!({
            "itemRemoved": id
        })).map_err(|e| format!("Failed to serialize notification: {}", e))?;

        // 通知所有客户端
        for (_, client_tx) in state.connected_clients.lock().unwrap().iter() {
            if client_tx.try_send(Message::Text(notification.clone())).is_err() {
                println!("Failed to notify client");
            }
        }

        // 发送成功响应给发送者
        let response = serde_json::to_string(&serde_json::json!({
            "status": "success",
            "message": "Item deleted successfully"
        })).map_err(|e| format!("Failed to serialize response: {}", e))?;

        ws_sender.lock().await.send(Message::Text(response))
            .await
            .map_err(|e| format!("Failed to send response: {}", e))?;
    }

    Ok(())
}

// 处理获取项目内容请求
pub async fn handle_get_item_content(
    state: Arc<AppState>,
    ws_sender: Arc<Mutex<futures_util::stream::SplitSink<tokio_tungstenite::WebSocketStream<tokio::net::TcpStream>, Message>>>,
    id: &str,
) -> Result<(), String> {
    let items = state.shared_items.lock().await;
    if let Some(item) = items.iter().find(|item| item.id == id) {
        let content = if item.item_type == "text" {
            item.content.clone()
        } else if let Some(path) = &item.path {
            std::fs::read_to_string(path)
                .unwrap_or_else(|_| "Failed to read file content".to_string())
        } else {
            "No content available".to_string()
        };

        let response = serde_json::to_string(&serde_json::json!({
            "id": id,
            "content": content
        })).map_err(|e| format!("Failed to serialize response: {}", e))?;

        ws_sender.lock().await.send(Message::Text(response))
            .await
            .map_err(|e| format!("Failed to send response: {}", e))?;
    }

    Ok(())
}