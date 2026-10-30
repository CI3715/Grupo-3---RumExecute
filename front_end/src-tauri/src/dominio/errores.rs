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

    // moneda.rs
    pub fn moneda_no_encontrada(codigo: &str) -> Self {
        Self::new(
            "moneda_no_encontrada",
            format!(
                "La moneda '{}' no pertenece a las monedas registradas.",
                codigo
            ),
        )
    }

    // moneda.rs
    pub fn codigo_moneda_invalido() -> Self {
        Self::new(
            "codigo",
            "El código de moneda debe tener entre 3 y 5 caracteres alfabéticos en mayúscula.",
        )
    }

    // moneda.rs
    pub fn nombre_moneda_invalido() -> Self {
        Self::new(
            "nombre",
            "El nombre de la moneda debe tener entre 1 y 50 caracteres.",
        )
    }

    // moneda.rs
    pub fn simbolo_moneda_invalido() -> Self {
        Self::new(
            "simbolo",
            "El símbolo de la moneda debe tener entre 1 y 5 caracteres.",
        )
    }

    // moneda.rs
    pub fn decimales_moneda_invalido() -> Self {
        Self::new(
            "decimales",
            "El número de decimales de la moneda no puede ser mayor a 8.",
        )
    }

    // cuenta.rs
    pub fn nombre_invalido(mensaje: impl Into<String>) -> Self {
        Self::new("nombre", mensaje)
    }

    // monto.rs
    pub fn saldo_invalido(mensaje: impl Into<String>) -> Self {
        Self::new("saldo_inicial", mensaje)
    }

    // categoria.rs
    pub fn tipo_invalido(mensaje: impl Into<String>) -> Self {
        Self::new("tipo", mensaje)
    }

    // categoria.rs
    pub fn categoria_invalida(mensaje: impl Into<String>) -> Self {
        Self::new("categoria", mensaje)
    }

    // monto.rs
    pub fn monto_invalido(mensaje: impl Into<String>) -> Self {
        Self::new("monto", mensaje)
    }

    // movimiento.rs
    pub fn fecha_invalida(mensaje: impl Into<String>) -> Self {
        Self::new("fecha", mensaje)
    }

    // tasa_cambio.rs
    pub fn monedas_iguales(origen: &str) -> Self {
        Self::new(
            "par_monedas",
            format!(
                "El par de monedas no puede tener la misma moneda de origen y destino: '{}'.",
                origen
            ),
        )
    }

    // tasa_cambio.rs
    pub fn tasa_invalida(mensaje: impl Into<String>) -> Self {
        Self::new("tasa_cambio", mensaje)
    }

    // conversion.rs
    pub fn tasa_no_existente(origen: &str, destino: &str, fecha: &str) -> Self {
        Self::new(
            "tasa_no_existente",
            format!(
                "Falta una tasa de {} a {} con fecha igual o anterior al {}",
                origen, destino, fecha
            ),
        )
    }
}
