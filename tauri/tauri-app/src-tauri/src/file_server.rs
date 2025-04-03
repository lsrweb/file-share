use axum::{
    body::Body,
    extract::{Path as AxumPath, State},
    http::{header, HeaderMap, Request, Response, StatusCode},
    response::{IntoResponse, Json},
    routing::get,
    Router,
};
use mime_guess::from_path;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use sysinfo::{System, SystemExt};
use tokio::{fs::File, io::AsyncReadExt, net::TcpListener, sync::Mutex};
use tower::ServiceExt;
use tower_http::{services::ServeFile, trace::TraceLayer};

// 文件服务器结构体
pub struct FileServer {
    addr: String,
    port: u16,
    base_path: PathBuf,
    server_handle: Arc<Mutex<Option<tokio::task::JoinHandle<()>>>>,
}

// 文件服务器状态结构体
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileServerStatus {
    pub running: bool,
    pub address: String,
    pub port: u16,
    pub url: String,
}

// Device info structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceInfo {
    pub hostname: String,
    pub ip_address: String,
    pub os_type: String,
    pub os_version: String,
    pub cpu_cores: usize,
    pub memory_total_mb: u64,
    pub server_port: u16,
}

// 自定义错误类型
#[derive(Debug)]
pub enum FileServerError {
    IoError(std::io::Error),
    AddrInUse,
    ServerNotRunning,
    FileNotFound,
    AccessDenied,
}

impl From<std::io::Error> for FileServerError {
    fn from(error: std::io::Error) -> Self {
        FileServerError::IoError(error)
    }
}

impl FileServer {
    // 创建新的文件服务器实例
    pub fn new(addr: String, port: u16, base_path: PathBuf) -> Self {
        FileServer {
            addr,
            port,
            base_path,
            server_handle: Arc::new(Mutex::new(None)),
        }
    }

    // 启动文件服务器
    pub async fn start(&self) -> Result<FileServerStatus, FileServerError> {
        let addr = format!("{}:{}", self.addr, self.port);
        let listener = match TcpListener::bind(&addr).await {
            Ok(listener) => listener,
            Err(e) if e.kind() == std::io::ErrorKind::AddrInUse => {
                return Err(FileServerError::AddrInUse);
            }
            Err(e) => return Err(FileServerError::IoError(e)),
        };

        let actual_addr = listener
            .local_addr()
            .map_err(|e| FileServerError::IoError(e))?;

        // 创建路由
        let app = Router::new()
            .route("/", get(get_device_info))
            .route("/files/{*file_path}", get(serve_file))
            .route("/download/{filename}", get(download_file))
            .route("/status", get(server_status))
            .route("/share/{filename}", get(share_file))
            .layer(TraceLayer::new_for_http())
            .with_state(AppState {
                base_path: self.base_path.clone(),
                server_addr: self.addr.clone(),
                server_port: self.port,
            });

        // 启动服务器
        let handle = tokio::spawn(async move {
            println!("File server started on http://{}", addr);
            axum::serve(listener, app).await.unwrap();
        });

        // 存储服务器句柄
        *self.server_handle.lock().await = Some(handle);

        Ok(FileServerStatus {
            running: true,
            address: self.addr.clone(),
            port: self.port,
            url: format!("http://{}:{}", self.addr, self.port),
        })
    }

    // 停止文件服务器
    pub async fn stop(&self) -> Result<(), FileServerError> {
        let mut server_handle = self.server_handle.lock().await;

        if let Some(handle) = server_handle.take() {
            handle.abort();
            println!("File server stopped");
            Ok(())
        } else {
            Err(FileServerError::ServerNotRunning)
        }
    }

    // 获取当前服务器状态
    pub async fn status(&self) -> FileServerStatus {
        let is_running = self.server_handle.lock().await.is_some();

        FileServerStatus {
            running: is_running,
            address: self.addr.clone(),
            port: self.port,
            url: format!("http://{}:{}", self.addr, self.port),
        }
    }
}

// 应用程序状态
#[derive(Clone)]
struct AppState {
    base_path: PathBuf,
    server_addr: String,
    server_port: u16,
}

// 处理文件请求
async fn serve_file(
    State(state): State<AppState>,
    AxumPath(file_path): AxumPath<String>,
) -> impl IntoResponse {
    let path = state.base_path.join(Path::new(&file_path));

    // 安全检查
    if !path.starts_with(&state.base_path) {
        return (StatusCode::FORBIDDEN, "Access denied").into_response();
    }

    // 检查文件是否存在
    if !path.exists() || !path.is_file() {
        return (StatusCode::NOT_FOUND, "File not found").into_response();
    }

    // 尝试提供文件
    match ServeFile::new(path)
        .oneshot(Request::new(Body::empty()))
        .await
    {
        Ok(response) => response.into_response(),
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Failed to serve file").into_response(),
    }
}

// 修改文件下载逻辑，确保只允许传入相对路径
async fn download_file(
    State(state): State<AppState>,
    AxumPath(file_path): AxumPath<String>,
) -> impl IntoResponse {
    let path = state.base_path.join(Path::new(&file_path));

    // 安全检查，确保路径在 base_path 内
    if !path.starts_with(&state.base_path) {
        return (StatusCode::FORBIDDEN, "Access denied").into_response();
    }

    // 检查文件是否存在
    if !path.exists() || !path.is_file() {
        return (StatusCode::NOT_FOUND, "File not found").into_response();
    }

    // 获取文件元数据
    let metadata = match tokio::fs::metadata(&path).await {
        Ok(metadata) => metadata,
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to read file metadata",
            )
                .into_response()
        }
    };

    // 打开文件
    let file = match File::open(&path).await {
        Ok(file) => file,
        Err(_) => {
            return (StatusCode::INTERNAL_SERVER_ERROR, "Failed to open file").into_response()
        }
    };

    // 创建包含 attachment 配置的头部
    let mut headers = HeaderMap::new();
    headers.insert(
        header::CONTENT_DISPOSITION,
        format!("attachment; filename=\"{}\"", file_path)
            .parse()
            .unwrap(),
    );

    // 根据文件扩展名设置 content type
    if let Some(mime) = from_path(&path).first() {
        headers.insert(header::CONTENT_TYPE, mime.to_string().parse().unwrap());
    }

    // 设置 content length
    headers.insert(
        header::CONTENT_LENGTH,
        metadata.len().to_string().parse().unwrap(),
    );

    // 创建流式响应
    let stream = file_to_stream(file, metadata.len() as usize);

    let mut builder = Response::builder().status(StatusCode::OK);
    for (name, value) in headers.iter() {
        builder = builder.header(name, value);
    }
    builder.body(Body::from_stream(stream)).unwrap()
}

// 新增一个方法，用于分享文件并广播文件信息
async fn share_file(
    State(state): State<AppState>,
    AxumPath(file_path): AxumPath<String>,
) -> impl IntoResponse {
    let path = state.base_path.join(Path::new(&file_path));

    // 安全检查，确保路径在 base_path 内
    if !path.starts_with(&state.base_path) {
        return (StatusCode::FORBIDDEN, "Access denied").into_response();
    }

    // 检查文件是否存在
    if !path.exists() || !path.is_file() {
        return (StatusCode::NOT_FOUND, "File not found").into_response();
    }

    // 获取文件元数据
    let metadata = match tokio::fs::metadata(&path).await {
        Ok(metadata) => metadata,
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to read file metadata",
            )
                .into_response()
        }
    };

    // 构建文件信息
    let file_info = serde_json::json!({
        "file_name": file_path,
        "file_size": metadata.len(),
        "relative_path": file_path,
    });

    // 广播文件信息给其他客户端
    if let Err(e) = broadcast_file_info(file_info).await {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to broadcast file info: {}", e),
        )
            .into_response();
    }

    (StatusCode::OK, "File shared successfully").into_response()
}

// 模拟广播文件信息的方法
async fn broadcast_file_info(file_info: serde_json::Value) -> Result<(), String> {
    // 这里可以实现实际的广播逻辑，例如通过 WebSocket 或 UDP 广播
    println!("Broadcasting file info: {}", file_info);
    Ok(())
}

// 将文件转换为异步流
fn file_to_stream(
    mut file: File,
    file_size: usize,
) -> impl futures::Stream<Item = Result<bytes::Bytes, std::io::Error>> {
    // 使用更大的缓冲区以获得更好的性能
    const CHUNK_SIZE: usize = 64 * 1024; // 64KB 块

    async_stream::stream! {
        let mut remaining = file_size;
        let mut buffer = vec![0; CHUNK_SIZE.min(remaining)];

        while remaining > 0 {
            let read_size = buffer.len().min(remaining);

            // 如果需要，为最后一个块调整缓冲区大小
            if read_size < buffer.len() {
                buffer.resize(read_size, 0);
            }

            match file.read_exact(&mut buffer).await {
                Ok(_) => {
                    remaining -= read_size;
                    yield Ok(bytes::Bytes::from(buffer.clone()));
                }
                Err(e) => {
                    yield Err(e);
                    break;
                }
            }
        }
    }
}

// 获取服务器状态
async fn server_status() -> impl IntoResponse {
    (
        StatusCode::OK,
        [(header::CONTENT_TYPE, "application/json")],
        r#"{"status":"running"}"#,
    )
}

// 获取设备信息
async fn get_device_info(State(state): State<AppState>) -> impl IntoResponse {
    // Initialize system information
    let mut sys = System::new_all();
    sys.refresh_all();

    // Get hostname
    let hostname = hostname::get()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();

    // Get OS information
    let os_type = System::name(&sys).unwrap_or_else(|| "Unknown".to_string());
    let os_version = System::os_version(&sys).unwrap_or_else(|| "Unknown".to_string());

    // Get CPU cores
    let cpu_cores = sys.cpus().len();

    // Get memory (convert to MB)
    let memory_total_mb = sys.total_memory() / 1024;

    let device_info = DeviceInfo {
        hostname,
        ip_address: state.server_addr.clone(),
        os_type,
        os_version,
        cpu_cores,
        memory_total_mb,
        server_port: state.server_port,
    };

    // Include application/json content type
    (
        StatusCode::OK,
        [(header::CONTENT_TYPE, "application/json")],
        Json(device_info),
    )
}
