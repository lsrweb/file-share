use std::collections::HashSet;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::UdpSocket;
use tokio::sync::Mutex;
use tokio::time::{sleep, Duration, Instant};

#[derive(Clone)]
pub struct UdpBroadcast {
    socket: Arc<Mutex<UdpSocket>>,
    known_addresses: Arc<Mutex<HashSet<SocketAddr>>>,
    broadcast_address: String,
    port: u16,
    last_broadcast_time: Arc<Mutex<Option<Instant>>>,
    broadcast_count: Arc<Mutex<u32>>,
    response_count: Arc<Mutex<u32>>,
}

impl UdpBroadcast {
    pub async fn new(broadcast_address: String, port: u16) -> Self {
        let listen_addr = format!("0.0.0.0:{}", port);
        println!("正在绑定 UDP Socket 到地址：{}", listen_addr);
        let socket = UdpSocket::bind(listen_addr).await.unwrap();

        // Enable broadcast mode on the socket
        socket.set_broadcast(true).unwrap();
        println!("UDP Socket 广播模式已启用");

        let socket = Arc::new(Mutex::new(socket));
        println!("UDP Socket 绑定成功");
        let known_addresses = Arc::new(Mutex::new(HashSet::new()));
        UdpBroadcast {
            socket,
            known_addresses,
            broadcast_address,
            port,
            last_broadcast_time: Arc::new(Mutex::new(None)),
            broadcast_count: Arc::new(Mutex::new(0)),
            response_count: Arc::new(Mutex::new(0)),
        }
    }

    pub async fn start_broadcasting(&self) {
        let broadcast_addr: SocketAddr = format!("{}:{}", self.broadcast_address, self.port)
            .parse()
            .unwrap();
        println!("开始广播到地址：{}", broadcast_addr);
        loop {
            let msg = b"Device Discovery Message";
            println!("发送广播消息...");
            match self.socket.lock().await.send_to(msg, &broadcast_addr).await {
                Ok(bytes_sent) => {
                    println!("广播消息已发送，发送字节数: {}", bytes_sent);
                    // Update broadcast stats
                    let mut count = self.broadcast_count.lock().await;
                    *count += 1;
                    let mut last_time = self.last_broadcast_time.lock().await;
                    *last_time = Some(Instant::now());
                }
                Err(e) => {
                    eprintln!("广播消息发送失败: {}", e);
                }
            }
            println!("等待 5 秒后重试");
            sleep(Duration::from_secs(5)).await;
        }
    }

    pub async fn listen_for_responses(&self) {
        let mut buf = [0; 1024];
        println!("开始监听设备响应...");
        loop {
            match self.socket.lock().await.recv_from(&mut buf).await {
                Ok((n, addr)) => {
                    let received = String::from_utf8_lossy(&buf[..n]);
                    println!("收到来自 {} 的消息: {}", addr, received);

                    // Increment response counter
                    let mut count = self.response_count.lock().await;
                    *count += 1;

                    let is_new = self.known_addresses.lock().await.insert(addr);
                    if is_new {
                        println!("新设备发现: {}", addr);
                    } else {
                        println!("已知设备响应: {}", addr);
                    }
                }
                Err(e) => eprintln!("接收数据报错误: {}", e),
            }
        }
    }

    pub async fn get_known_addresses(&self) -> HashSet<SocketAddr> {
        println!("获取已知设备列表");
        self.known_addresses.lock().await.clone()
    }

    // Verify if broadcast is working properly
    pub async fn verify_broadcast_status(&self) -> BroadcastStatus {
        let devices = self.known_addresses.lock().await.len() as u32;
        let broadcast_count = *self.broadcast_count.lock().await;
        let response_count = *self.response_count.lock().await;
        let last_broadcast = *self.last_broadcast_time.lock().await;

        BroadcastStatus {
            is_broadcasting: last_broadcast.is_some(),
            last_broadcast_time: last_broadcast,
            broadcast_count,
            response_count,
            discovered_devices: devices,
        }
    }

    // Send a single verification broadcast and wait for responses
    pub async fn send_verification_broadcast(&self, timeout_secs: u64) -> bool {
        let broadcast_addr: SocketAddr = format!("{}:{}", self.broadcast_address, self.port)
            .parse()
            .unwrap();

        println!("发送验证广播...");
        let msg = b"Verification Broadcast";

        let before_count = self.known_addresses.lock().await.len();

        // Send the verification message
        match self.socket.lock().await.send_to(msg, &broadcast_addr).await {
            Ok(_) => {
                println!("验证广播发送成功，等待 {} 秒检查响应", timeout_secs);
                sleep(Duration::from_secs(timeout_secs)).await;

                let after_count = self.known_addresses.lock().await.len();
                let success = after_count > before_count;

                if success {
                    println!("验证成功: 发现了 {} 个新设备", after_count - before_count);
                } else {
                    println!("验证超时: 没有发现新设备");
                }

                success
            }
            Err(e) => {
                eprintln!("验证广播发送失败: {}", e);
                false
            }
        }
    }
}

// Struct to hold broadcast status information
#[derive(Debug, Clone)]
pub struct BroadcastStatus {
    pub is_broadcasting: bool,
    pub last_broadcast_time: Option<Instant>,
    pub broadcast_count: u32,
    pub response_count: u32,
    pub discovered_devices: u32,
}
