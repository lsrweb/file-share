use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Mutex;
use tokio::sync::Mutex as TokioMutex;
use tokio::sync::mpsc;
use tokio_tungstenite::tungstenite::Message;

use crate::models::SharedItem;

pub struct AppState {
    pub connected_clients: Mutex<HashMap<SocketAddr, mpsc::Sender<Message>>>,
    pub shared_items: TokioMutex<Vec<SharedItem>>,
}