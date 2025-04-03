// ＃网络服务广播和发现模块
//  
// 
// ＃＃ 全双工广播和发现
// 
//  - **服务广播**：定期广播服务器的地址和端口
//    对于本地子网，以便其他实例可以发现它。
//  - **服务发现**：从其他实例中收听广播的功能
//    并编译网络上可用服务器的列表。
// 
// ##实施详细信息
// 
// 广播系统：
//  -默认情况下在端口上使用UDP广播
//  -在所有非环回，非APIPA网络接口上进行广播
//  -每5秒发送一次公告
//  -格式广播消息为`“ server_addr：port”`````````````
// 
// ##当前限制
// 
// 发现服务目前是占位符，将被充分实施
// 在将来的更新中。广播功能依赖于JavaScript侧UDP
// 通过命令实现。
// 
 

use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::time::Duration;
use tokio::time::sleep;

const BROADCAST_PORT: u16 = 5421;
const BROADCAST_INTERVAL: Duration = Duration::from_secs(5);

/// Get all valid IPv4 network interfaces (exclude loopback and APIPA addresses)
fn get_network_interfaces() -> Vec<Ipv4Addr> {
    let mut interfaces = Vec::new();
    if let Ok(ifaces) = local_ip_address::list_afinet_netifas() {
        for (_name, addr) in ifaces {
            if let IpAddr::V4(ipv4) = addr {
                let octets = ipv4.octets();
                if !ipv4.is_loopback() && !(octets[0] == 169 && octets[1] == 254) {
                    interfaces.push(ipv4);
                }
            }
        }
    }
    if interfaces.is_empty() {
        interfaces.push(Ipv4Addr::new(0, 0, 0, 0));
    }
    interfaces
}

/// Calculate subnet broadcast address
fn get_subnet_broadcast(ip: &Ipv4Addr) -> Ipv4Addr {
    let octets = ip.octets();
    Ipv4Addr::new(octets[0], octets[1], octets[2], 255)
}

/// Start service broadcasting
pub async fn start_broadcast_service(server_addr: String, port: u16) -> Result<(), String> {
    let interfaces = get_network_interfaces();
    let id = "broadcast-service";


    loop {
        for iface in &interfaces {
            let broadcast_addr = SocketAddr::new(IpAddr::V4(get_subnet_broadcast(iface)), BROADCAST_PORT);
            let message = format!("{}:{}", server_addr, port);

            // Send command to JS side to broadcast
            if let Err(e) = crate::handlers::udp_send(id.to_string(), broadcast_addr.to_string(), message).await {
                eprintln!("Failed to send broadcast on interface {}: {}", iface, e);
            } else {
                println!("Broadcast sent on interface {} to {}", iface, broadcast_addr);
            }
        }
        sleep(BROADCAST_INTERVAL).await;
    }
}

pub async fn discover_network_services() -> Result<Vec<crate::models::ServerInfo>, String> {
    let _id = "discovery-service";
    Ok(Vec::new())
}

pub async fn start_discovery_service() -> Result<(), String> {
    Ok(())
}
