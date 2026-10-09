use serde::{Deserialize, Serialize};
use std::time::Instant;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PingBackendResponse {
    pub estado: String,
    pub mensaje: String,
    pub version: String,
    pub timestamp: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PingResult {
    pub estado: String,
    pub mensaje: String,
    pub version: String,
    pub timestamp: String,
    pub latencia_ms: u64,
}

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

/// Función que ejecuta la petición HTTP GET a /ping en el servidor backend,
/// mide la latencia de respuesta y deserializa el resultado.
#[tauri::command]
async fn ping_servidor() -> Result<PingResult, String> {
    let inicio = Instant::now();
    let url = "http://127.0.0.1:8000/ping";

    let response = reqwest::get(url)
        .await
        .map_err(|e| format!("No se pudo conectar con el servidor backend en {}. Asegúrese de que el servidor esté activo. Detalle: {}", url, e))?;

    let status = response.status();
    if !status.is_success() {
        return Err(format!("El servidor backend respondió con error HTTP {}", status));
    }

    let backend_data = response
        .json::<PingBackendResponse>()
        .await
        .map_err(|e| format!("Error al procesar la respuesta JSON del servidor: {}", e))?;

    let latencia_ms = inicio.elapsed().as_millis() as u64;

    Ok(PingResult {
        estado: backend_data.estado,
        mensaje: backend_data.mensaje,
        version: backend_data.version,
        timestamp: backend_data.timestamp,
        latencia_ms,
    })
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![greet, ping_servidor])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
