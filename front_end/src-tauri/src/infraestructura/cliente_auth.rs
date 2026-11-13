"""
Cliente HTTP y Almacén Seguro en Keyring (Entrega 5 - Rust/Tauri).
"""
use keyring::Entry;
use serde::{Deserialize, Serialize};
use std::env;

const SERVICIO_KEYRING: &str = "cuentas_claras_app";
const USUARIO_KEYRING: &str = "session_token";

fn obtener_url_base_backend() -> String {
    env::var("BACKEND_URL").unwrap_or_else(|_| "http://127.0.0.1:8000".to_string())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatosUsuario {
    pub id: String,
    pub nombre_usuario: String,
    pub nombre_visible: String,
    pub correo: String,
    pub fecha_alta: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RespuestaSesion {
    pub access_token: String,
    pub token_type: String,
    pub expires_at: String,
    pub usuario: DatosUsuario,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntradaRegistro {
    pub nombre_usuario: String,
    pub nombre_visible: String,
    pub correo: String,
    pub contrasena: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntradaLogin {
    pub correo: String,
    pub contrasena: String,
}

// --- Manejo del Keyring (Almacén de Secretos del SO) ---

pub fn guardar_token_keyring(token: &str) -> Result<(), String> {
    let entry = Entry::new(SERVICIO_KEYRING, USUARIO_KEYRING)
        .map_err(|e| format!("Error inicializando keyring: {e}"))?;
    entry
        .set_password(token)
        .map_err(|e| format!("Error guardando token en el sistema seguro: {e}"))?;
    Ok(())
}

pub fn obtener_token_keyring() -> Result<Option<String>, String> {
    let entry = Entry::new(SERVICIO_KEYRING, USUARIO_KEYRING)
        .map_err(|e| format!("Error inicializando keyring: {e}"))?;
    match entry.get_password() {
        Ok(token) => Ok(Some(token)),
        Err(keyring::Error::NoEntry) => Ok(None),
        Err(e) => Err(format!("Error leyendo token de keyring: {e}")),
    }
}

pub fn borrar_token_keyring() -> Result<(), String> {
    let entry = Entry::new(SERVICIO_KEYRING, USUARIO_KEYRING)
        .map_err(|e| format!("Error inicializando keyring: {e}"))?;
    match entry.delete_credential() {
        Ok(_) | Err(keyring::Error::NoEntry) => Ok(()),
        Err(e) => Err(format!("Error borrando token de keyring: {e}")),
    }
}

// --- Peticiones HTTP a Backend (Reqwest) ---

pub async fn peticion_registro(entrada: EntradaRegistro) -> Result<DatosUsuario, String> {
    let client = reqwest::Client::new();
    let url = format!("{}/usuarios", obtener_url_base_backend());

    let resp = client
        .post(&url)
        .json(&entrada)
        .send()
        .await
        .map_err(|_| "Sin conexión con el servidor. Verifique que esté activo.".to_string())?;

    let status = resp.status();
    if status.is_success() {
        resp.json::<DatosUsuario>()
            .await
            .map_err(|e| format!("Error deserializando respuesta: {e}"))
    } else {
        let error_body = resp.text().await.unwrap_or_default();
        Err(formatear_error_backend(status.as_u16(), &error_body))
    }
}

pub async fn peticion_login(entrada: EntradaLogin) -> Result<RespuestaSesion, String> {
    let client = reqwest::Client::new();
    let url = format!("{}/sesiones", obtener_url_base_backend());

    let resp = client
        .post(&url)
        .json(&entrada)
        .send()
        .await
        .map_err(|_| "Sin conexión con el servidor. Verifique que esté activo.".to_string())?;

    let status = resp.status();
    if status.is_success() {
        let sesion = resp
            .json::<RespuestaSesion>()
            .await
            .map_err(|e| format!("Error deserializando respuesta de sesión: {e}"))?;
        
        // Guardar token en el almacén seguro del SO
        let _ = guardar_token_keyring(&sesion.access_token);
        
        Ok(sesion)
    } else {
        let error_body = resp.text().await.unwrap_or_default();
        Err(formatear_error_backend(status.as_u16(), &error_body))
    }
}

pub async fn peticion_perfil_propio(token: &str) -> Result<DatosUsuario, String> {
    let client = reqwest::Client::new();
    let url = format!("{}/usuarios/yo", obtener_url_base_backend());

    let resp = client
        .get(&url)
        .bearer_auth(token)
        .send()
        .await
        .map_err(|_| "Sin conexión con el servidor. Verifique que esté activo.".to_string())?;

    let status = resp.status();
    if status.is_success() {
        resp.json::<DatosUsuario>()
            .await
            .map_err(|e| format!("Error deserializando perfil: {e}"))
    } else {
        if status.as_u16() == 401 {
            // Si el token expiró o es inválido, limpiamos el almacén local
            let _ = borrar_token_keyring();
            return Err("Tu sesión venció, inicia sesión de nuevo.".to_string());
        }
        let error_body = resp.text().await.unwrap_or_default();
        Err(formatear_error_backend(status.as_u16(), &error_body))
    }
}

fn formatear_error_backend(codigo: u16, cuerpo: &str) -> String {
    if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(cuerpo) {
        if let Some(detail) = parsed.get("detail") {
            if let Some(mensaje) = detail.get("mensaje").and_then(|m| m.as_str()) {
                return mensaje.to_string();
            }
            if let Some(texto) = detail.as_str() {
                return texto.to_string();
            }
        }
    }
    format!("Error del servidor (código {codigo})")
}

    