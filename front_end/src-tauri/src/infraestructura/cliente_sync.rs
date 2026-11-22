//! Cliente HTTP de Sincronización Unidireccional y Despacho Offline (Entrega 7).
//! Conecta con GET /cambios?desde=<cursor> y POST /grupos/{id}/gastos con Header Idempotency-Key.

use reqwest::header::{HeaderMap, HeaderValue};
use serde::{Deserialize, Serialize};
use std::env;

use super::cliente_auth::{borrar_token_keyring, obtener_token_keyring};
use super::sqlite_repo::{EventoServidorRecibido, OperacionGastoOffline};

fn obtener_url_base_backend() -> String {
    env::var("BACKEND_URL").unwrap_or_else(|_| "http://127.0.0.1:8000".to_string())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RespuestaCambiosBackend {
    pub eventos: Vec<EventoServidorRecibido>,
    pub nuevo_cursor: i64,
    pub quedan_mas: bool,
}

pub async fn peticion_obtener_cambios(desde: i64) -> Result<RespuestaCambiosBackend, String> {
    let token = obtener_token_keyring()?
        .ok_or_else(|| "Tu sesión venció, inicia sesión de nuevo.".to_string())?;

    let client = reqwest::Client::new();
    let url = format!("{}/cambios?desde={}", obtener_url_base_backend(), desde);

    let resp = client
        .get(&url)
        .bearer_auth(&token)
        .send()
        .await
        .map_err(|e| format!("Error conectando con el servidor: {e}"))?;

    let status = resp.status();
    if status.is_success() {
        resp.json::<RespuestaCambiosBackend>()
            .await
            .map_err(|e| format!("Error deserializando cambios del servidor: {e}"))
    } else {
        if status.as_u16() == 401 {
            let _ = borrar_token_keyring();
            return Err("Tu sesión venció, inicia sesión de nuevo.".to_string());
        }
        let error_body = resp.text().await.unwrap_or_default();
        Err(format!("Error del servidor (código {status}): {error_body}"))
    }
}

pub async fn peticion_enviar_gasto_offline(
    operacion: &OperacionGastoOffline,
) -> Result<serde_json::Value, String> {
    let token = obtener_token_keyring()?
        .ok_or_else(|| "Tu sesión venció, inicia sesión de nuevo.".to_string())?;

    let client = reqwest::Client::new();
    let url = format!("{}/grupos/{}/gastos", obtener_url_base_backend(), operacion.grupo_id);

    let mut headers = HeaderMap::new();
    if let Ok(val) = HeaderValue::from_str(&operacion.idempotency_key) {
        headers.insert("Idempotency-Key", val);
    }

    let monto_dec: rust_decimal::Decimal = operacion
        .monto
        .parse()
        .map_err(|e| format!("Monto inválido en gasto offline: {e}"))?;

    let mut body = serde_json::json!({
        "concepto": operacion.concepto,
        "monto": monto_dec,
        "pagador_id": operacion.pagador_id,
        "regla_division": operacion.regla_division,
        "participantes": operacion.participantes,
    });

    if let Some(valores) = &operacion.valores {
        body["valores"] = serde_json::to_value(valores).unwrap_or(serde_json::Value::Null);
    }

    let resp = client
        .post(&url)
        .headers(headers)
        .bearer_auth(&token)
        .json(&body)
        .send()
        .await
        .map_err(|e| format!("Sin conexión al servidor: {e}"))?;

    let status = resp.status();
    if status.is_success() {
        resp.json::<serde_json::Value>()
            .await
            .map_err(|e| format!("Error deserializando respuesta: {e}"))
    } else {
        if status.as_u16() == 401 {
            let _ = borrar_token_keyring();
            return Err("Tu sesión venció, inicia sesión de nuevo.".to_string());
        }
        let error_body = resp.text().await.unwrap_or_default();
        Err(format!("Error del servidor ({status}): {error_body}"))
    }
}
