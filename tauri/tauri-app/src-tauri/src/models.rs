use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct BroadcastMessage {
    pub message_type: String,
    pub server_address: String,
    pub server_port: u16,
    pub server_name: String,
    pub timestamp: u64,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct SharedItem {
    pub id: String,
    pub name: String,
    #[serde(rename = "type")]
    pub item_type: String, // "text" or "file"
    pub content: String,
    pub path: Option<String>,
    pub username: String,
    pub uploadTime: u64,
    pub size: Option<u64>,
    pub file_type: Option<String>,
}