    use tauri::{AppHandle, State};


    // Tauri commands
    #[tauri::command]
    fn get_server_address(state: State<'_, String>) -> String {
        state.inner().clone()
    }


    #[tauri::command]
    fn greet(name: &str) -> String {
        format!("Hello, {}! You've been greeted from Rust!", name)
    }

    #[tauri::command]
    fn get_local_ip() -> Result<String, String> {
        match local_ip_address::local_ip() {
            Ok(ip) => Ok(ip.to_string()),
            Err(e) => Err(format!("Failed to get local IP address: {}", e))
        }
    }

    #[cfg_attr(mobile, tauri::mobile_entry_point)]
    pub fn run() {
        tauri::Builder::default()
            .plugin(tauri_plugin_udp::init())
            .plugin(tauri_plugin_single_instance::init(|_app_handle: &AppHandle, argv: Vec<String>, cwd: String| {
                println!("Another instance tried to launch with args: {:?}, cwd: {}", argv, cwd);
                println!("Focus the existing instance.");
            }))
            .plugin(tauri_plugin_shell::init())
            .plugin(tauri_plugin_process::init())
            .plugin(tauri_plugin_dialog::init())
            .plugin(tauri_plugin_opener::init())
            .invoke_handler(tauri::generate_handler![
                greet,
                get_server_address,
                get_local_ip
            ])
            .run(tauri::generate_context!())
            .expect("error while running tauri application");
    }
