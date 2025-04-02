use std::time::Duration;
use tokio::net::UdpSocket;
use tokio::sync::mpsc;
use tokio::time::sleep;

use crate::models::BroadcastMessage;

// 广播常量
pub const BROADCAST_PORT: u16 = 5421; // Changed from 45678 to fixed port 5421
pub const SERVICE_DISCOVERY_MSG: &str = "FILE_SHARE_SERVICE";
pub const BROADCAST_INTERVAL: Duration = Duration::from_secs(5);
pub const DISCOVERY_TIMEOUT: Duration = Duration::from_secs(5); // 增加超时时间
const BUFFER_SIZE: usize = 4096; // 增加缓冲区大小
const CHANNEL_CAPACITY: usize = 100; // 增加通道容量

// 启动UDP广播服务
pub async fn start_broadcast_service(server_addr: String, server_port: u16) -> Result<(), String> {
    let socket = UdpSocket::bind("0.0.0.0:0")
        .await
        .map_err(|e| format!("Failed to bind UDP socket: {}", e))?;
    
    socket.set_broadcast(true)
        .map_err(|e| format!("Failed to set broadcast option: {}", e))?;
    
    let broadcast_addr = format!("255.255.255.255:{}", BROADCAST_PORT);
    
    let broadcast_msg = BroadcastMessage {
        message_type: SERVICE_DISCOVERY_MSG.to_string(),
        server_address: server_addr.clone(),
        server_port,
        server_name: format!("Service Discovery Server ({})", server_addr),
        timestamp: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64,
    };
    
    let broadcast_json = serde_json::to_string(&broadcast_msg)
        .map_err(|e| format!("Failed to serialize broadcast message: {}", e))?;
    
    println!("Starting broadcast service for server: {}", server_addr);
    
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
pub async fn start_discovery_service() -> Result<mpsc::Receiver<BroadcastMessage>, String> {
    let socket = UdpSocket::bind(format!("0.0.0.0:{}", BROADCAST_PORT))
        .await
        .map_err(|e| format!("Failed to bind discovery socket: {}", e))?;
    
    socket.set_broadcast(true)
        .map_err(|e| format!("Failed to set broadcast option: {}", e))?;
    
    let (tx, rx) = mpsc::channel::<BroadcastMessage>(CHANNEL_CAPACITY);
    
    tokio::spawn(async move {
        let mut buf = vec![0u8; BUFFER_SIZE];
        
        loop {
            match socket.recv_from(&mut buf).await {
                Ok((size, addr)) => {
                    if let Ok(json_str) = std::str::from_utf8(&buf[..size]) {
                        if let Ok(message) = serde_json::from_str::<BroadcastMessage>(json_str) {
                            if message.message_type == SERVICE_DISCOVERY_MSG {
                                println!("Discovered service: {} at {}:{} from {}", 
                                         message.server_name, message.server_address, message.server_port, addr);
                                
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

// 发现网络服务的辅助函数
pub async fn discover_network_services(local_ip: String, local_port: u16) -> Result<Vec<BroadcastMessage>, String> {
    let socket = UdpSocket::bind("0.0.0.0:0")
        .await
        .map_err(|e| format!("Failed to bind UDP socket: {}", e))?;
    
    socket.set_broadcast(true)
        .map_err(|e| format!("Failed to set broadcast option: {}", e))?;
        
    let broadcast_addr = format!("255.255.255.255:{}", BROADCAST_PORT);
    let (tx, mut rx) = mpsc::channel::<BroadcastMessage>(CHANNEL_CAPACITY);
    let mut discovered_services = Vec::new();

    // 添加本机服务
    let local_service = BroadcastMessage {
        message_type: SERVICE_DISCOVERY_MSG.to_string(),
        server_address: local_ip.clone(),
        server_port: local_port,
        server_name: format!("Local Service ({})", local_ip),
        timestamp: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64,
    };
    discovered_services.push(local_service);

    // 发送广播消息
    let discovery_request = BroadcastMessage {
        message_type: "DISCOVERY_REQUEST".to_string(),
        server_address: local_ip.clone(),
        server_port: local_port,
        server_name: format!("Local Service ({})", local_ip),
        timestamp: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64,
    };
    
    let discovery_json = serde_json::to_string(&discovery_request)
        .map_err(|e| format!("Failed to serialize discovery request: {}", e))?;

    if let Err(e) = socket.send_to(discovery_json.as_bytes(), &broadcast_addr).await {
        return Err(format!("Failed to send discovery request: {}", e));
    }

    // 监听回复 (使用新的超时时间)
    let timeout = tokio::time::timeout(DISCOVERY_TIMEOUT, async {
        let mut buf = vec![0u8; BUFFER_SIZE];
        loop {
            match socket.recv_from(&mut buf).await {
                Ok((size, addr)) => {
                    if let Ok(json_str) = std::str::from_utf8(&buf[..size]) {
                        if let Ok(message) = serde_json::from_str::<BroadcastMessage>(json_str) {
                            if message.message_type == SERVICE_DISCOVERY_MSG {
                                println!("Discovery response from {}: {} at {}:{}", 
                                    addr, message.server_name, message.server_address, message.server_port);
                                
                                // 不添加来自自己的响应
                                if message.server_address != local_ip || 
                                   message.server_port != local_port {
                                    if tx.send(message).await.is_err() {
                                        break;
                                    }
                                }
                            }
                        }
                    }
                },
                Err(e) => {
                    eprintln!("Error receiving discovery response: {}", e);
                    break;
                },
            }
        }
    }).await;

    if timeout.is_ok() {
        while let Ok(service) = rx.try_recv() {
            if !discovered_services.iter().any(|s: &BroadcastMessage| {
                s.server_address == service.server_address && s.server_port == service.server_port
            }) {
                discovered_services.push(service);
            }
        }
    }

    Ok(discovered_services)
}