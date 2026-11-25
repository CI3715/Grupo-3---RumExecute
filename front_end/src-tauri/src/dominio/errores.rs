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
                "Falta una tasa de {} a {} con fecha igual o anterior al {}.",
                origen, destino, fecha
            ),
        )
    }

    pub fn dispositivo_id_invalido(mensaje: impl Into<String>) -> Self {
        Self::new("dispositivo_origen", mensaje)
    }

    pub fn evento_invalido(mensaje: impl Into<String>) -> Self {
        Self::new("evento", mensaje)
    }

    pub fn cuenta_no_encontrada(id_cuenta: &str) -> Self {
        Self::new(
            "id_cuenta",
            format!(
                "No existe ninguna cuenta registrada con el id '{}'.",
                id_cuenta
            ),
        )
    }

    /// Error legible cuando la versión del archivo respaldado no coincide con la soportada.
    pub fn version_formato_no_soportada(version_archivo: &str, version_soportada: &str) -> Self {
        Self::new(
            "version_formato",
            format!(
                "La versión de formato '{}' del archivo no es compatible; esta aplicación soporta la versión '{}'.",
                version_archivo, version_soportada
            ),
        )
    }

    /// Error legible cuando una línea o estructura de evento del respaldo no cumple el formato esperado.
    pub fn evento_mal_formado(detalle: impl Into<String>) -> Self {
        Self::new("evento", detalle)
    }

    /// Error legible cuando un evento entrante tiene el mismo identificador que uno existente pero distinto contenido.
    pub fn evento_repetido_contenido_distinto(id_evento: &str) -> Self {
        Self::new(
            "id_evento",
            format!(
                "El evento '{}' ya existe en el libro con un contenido diferente y fue rechazado.",
                id_evento
            ),
        )
    }

    /// Error legible cuando la frase de paso no cumple la longitud mínima exigida o está en blanco.
    pub fn frase_paso_invalida() -> Self {
        Self::new(
            "frase_paso",
            format!(
                "La frase de paso debe tener al menos {} caracteres y no puede estar en blanco.",
                LONGITUD_MINIMA_FRASE_PASO
            ),
        )
    }
}

pub const LONGITUD_MINIMA_FRASE_PASO: usize = 12;

/// Valida que la frase de paso no esté compuesta únicamente por espacios en blanco
/// y tenga al menos `LONGITUD_MINIMA_FRASE_PASO` caracteres (contando caracteres Unicode, no bytes).
pub fn validar_frase_paso(frase: &str) -> Result<(), ErrorDominio> {
    if frase.trim().is_empty() || frase.chars().count() < LONGITUD_MINIMA_FRASE_PASO {
        return Err(ErrorDominio::frase_paso_invalida());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn errores_dominio_son_legibles_y_sin_codigos_internos() {
        let muestras = vec![
            ErrorDominio::moneda_no_encontrada("XYZ"),
            ErrorDominio::codigo_moneda_invalido(),
            ErrorDominio::nombre_moneda_invalido(),
            ErrorDominio::simbolo_moneda_invalido(),
            ErrorDominio::decimales_moneda_invalido(),
            ErrorDominio::monedas_iguales("USD"),
            ErrorDominio::tasa_no_existente("USD", "EUR", "2026-11-25"),
            ErrorDominio::cuenta_no_encontrada("cta-1"),
            ErrorDominio::version_formato_no_soportada("99", "1"),
            ErrorDominio::evento_mal_formado("La línea del evento no tiene el formato esperado."),
            ErrorDominio::evento_repetido_contenido_distinto("evt-dup-1"),
            ErrorDominio::frase_paso_invalida(),
        ];

        for err in muestras {
            assert!(!err.campo.trim().is_empty());
            assert!(err.mensaje.ends_with('.'));
            assert!(!err.mensaje.contains("INV-"));
            assert!(!err.mensaje.contains("RF-"));
            assert!(!err.mensaje.contains("RNF-"));
            assert_eq!(err.to_string(), format!("{} (campo: {})", err.mensaje, err.campo));
        }
    }

    #[test]
    fn valida_longitud_minima_y_contenido_de_frase_paso() {
        assert!(validar_frase_paso("corta123456").is_err()); // 11 caracteres ASCII
        assert!(validar_frase_paso("ñññññññññññ").is_err()); // 11 caracteres Unicode (22 bytes)
        assert!(validar_frase_paso("            ").is_err()); // 12 espacios en blanco
        assert!(validar_frase_paso("ññññññññññññ").is_ok()); // 12 caracteres Unicode (24 bytes)
        assert!(validar_frase_paso("clave-segura").is_ok()); // 12 caracteres ASCII
    }
}
