use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Representa un error de validación en la capa de dominio.
/// Cumple con el contrato exigido por la especificación:
/// { campo: "nombre" | "moneda" | "tipo" | "saldo_inicial", mensaje: "..." }
/// sin exponer trazas técnicas (RNF-18).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Error)]
#[error("{mensaje} (campo: {campo})")]
pub struct ErrorDominio {
    pub campo: String,
    pub mensaje: String,
}

impl ErrorDominio {
    pub fn new(campo: impl Into<String>, mensaje: impl Into<String>) -> Self {
        Self {
            campo: campo.into(),
            mensaje: mensaje.into(),
        }
    }

    pub fn moneda_invalida(codigo: &str) -> Self {
        Self::new(
            "moneda",
            format!(
                "La moneda '{}' no pertenece a las monedas registradas.",
                codigo
            ),
        )
    }

    pub fn codigo_moneda_invalido() -> Self {
        Self::new(
            "codigo",
            "El código de moneda debe tener entre 3 y 5 caracteres alfabéticos en mayúscula.",
        )
    }

    pub fn nombre_invalido(mensaje: impl Into<String>) -> Self {
        Self::new("nombre", mensaje)
    }

    pub fn saldo_invalido(mensaje: impl Into<String>) -> Self {
        Self::new("saldo_inicial", mensaje)
    }

    pub fn tipo_invalido(mensaje: impl Into<String>) -> Self {
        Self::new("tipo", mensaje)
    }

    pub fn categoria_invalida(mensaje: impl Into<String>) -> Self {
        Self::new("categoria", mensaje)
    }

    pub fn monto_invalido(mensaje: impl Into<String>) -> Self {
        Self::new("monto", mensaje)
    }

    pub fn fecha_invalida(mensaje: impl Into<String>) -> Self {
        Self::new("fecha", mensaje)
    }
}
