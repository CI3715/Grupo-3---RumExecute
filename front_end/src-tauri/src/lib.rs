// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

// Funcion para ejecutar la petición HTTP GET al servidor
#[tauri::command]
async fn ping_servidor() -> Result<String,String> {
    match reqwest::get("http://127.0.0.1:8000").await {
        Ok(_) => Ok("Servidor conectado correctamente".to_string()),
        Err(e) => Err(format!("Error al conectar con el servidor: {}", e)),
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![greet, ping_servidor])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
