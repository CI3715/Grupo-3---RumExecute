use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

use super::errores::ErrorDominio;
use super::moneda::CodigoMoneda;
use super::monto::Monto;

/// Representa un cambio generado por el servidor para un usuario
/// a partir de su participación en un gasto compartido.
///
/// No representa todavía un movimiento aplicado al libro personal.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EventoCambioUsuario {
    pub id: String,
    pub posicion: u64,
    pub usuario_id: String,
    pub grupo_id: String,
    pub nombre_grupo: String,
    pub gasto_id: String,
    pub concepto: String,
    pub fecha: DateTime<Utc>,
    pub moneda: CodigoMoneda,
    #[serde(with = "rust_decimal::serde::str")]
    pub monto_parte: Decimal,
    pub incluye_redondeo: bool,
}

impl EventoCambioUsuario {
    /// Construye y valida un evento recibido del servidor.
    #[allow(clippy::too_many_arguments)]
    pub fn nuevo(
        id: impl Into<String>,
        posicion: u64,
        usuario_id: impl Into<String>,
        grupo_id: impl Into<String>,
        nombre_grupo: impl Into<String>,
        gasto_id: impl Into<String>,
        concepto: impl Into<String>,
        fecha: DateTime<Utc>,
        moneda: CodigoMoneda,
        monto_parte: Decimal,
        incluye_redondeo: bool,
    ) -> Result<Self, ErrorDominio> {
        let evento = Self {
            id: id.into(),
            posicion,
            usuario_id: usuario_id.into(),
            grupo_id: grupo_id.into(),
            nombre_grupo: nombre_grupo.into(),
            gasto_id: gasto_id.into(),
            concepto: concepto.into(),
            fecha,
            moneda,
            monto_parte,
            incluye_redondeo,
        };

        evento.validar()?;

        Ok(evento)
    }

    /// Comprueba los invariantes del evento.
    ///
    /// Es público porque Serde puede deserializar una estructura
    /// sin pasar por su constructor.
    pub fn validar(&self) -> Result<(), ErrorDominio> {
        for (campo, valor) in [
            ("id", self.id.as_str()),
            ("usuario_id", self.usuario_id.as_str()),
            ("grupo_id", self.grupo_id.as_str()),
            ("nombre_grupo", self.nombre_grupo.as_str()),
            ("gasto_id", self.gasto_id.as_str()),
            ("concepto", self.concepto.as_str()),
        ] {
            if valor.trim().is_empty() {
                return Err(ErrorDominio::new(
                    campo,
                    format!("El campo '{campo}' no puede estar vacío."),
                ));
            }
        }

        if self.posicion == 0 {
            return Err(ErrorDominio::new(
                "posicion",
                "La posición del evento debe ser mayor que cero.",
            ));
        }

        if self.monto_parte < Decimal::ZERO {
            return Err(ErrorDominio::new(
                "monto_parte",
                "La parte del gasto no puede ser negativa.",
            ));
        }

        // Los gastos compartidos del backend se dividen en céntimos.
        if self.monto_parte.normalize().scale() > 2 {
            return Err(ErrorDominio::new(
                "monto_parte",
                "La parte del gasto admite como máximo dos decimales.",
            ));
        }

        Ok(())
    }

    /// Convierte la cuota recibida a un monto del dominio local.
    pub fn monto(&self) -> Monto {
        Monto::new(self.monto_parte, self.moneda)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn evento_valido() -> EventoCambioUsuario {
        let fecha = DateTime::parse_from_rfc3339("2026-11-22T12:00:00Z")
            .unwrap()
            .with_timezone(&Utc);

        EventoCambioUsuario::nuevo(
            "evento-servidor-1",
            1,
            "usuario-1",
            "grupo-1",
            "Viaje",
            "gasto-1",
            "Hotel",
            fecha,
            CodigoMoneda::USD,
            Decimal::new(3334, 2),
            true,
        )
        .unwrap()
    }

    #[test]
    fn crea_evento_valido() {
        let evento = evento_valido();

        assert_eq!(evento.id, "evento-servidor-1");
        assert_eq!(evento.posicion, 1);
        assert_eq!(evento.usuario_id, "usuario-1");
        assert_eq!(evento.grupo_id, "grupo-1");
        assert_eq!(evento.gasto_id, "gasto-1");
        assert_eq!(evento.monto_parte, Decimal::new(3334, 2));
        assert!(evento.incluye_redondeo);
    }

    #[test]
    fn rechaza_posicion_cero() {
        let mut evento = evento_valido();
        evento.posicion = 0;

        let error = evento.validar().unwrap_err();

        assert_eq!(error.campo, "posicion");
    }

    #[test]
    fn rechaza_identificadores_vacios() {
        let mut evento = evento_valido();
        evento.usuario_id = "   ".to_string();

        let error = evento.validar().unwrap_err();

        assert_eq!(error.campo, "usuario_id");
    }

    #[test]
    fn rechaza_monto_negativo() {
        let mut evento = evento_valido();
        evento.monto_parte = Decimal::new(-100, 2);

        let error = evento.validar().unwrap_err();

        assert_eq!(error.campo, "monto_parte");
    }

    #[test]
    fn rechaza_mas_de_dos_decimales() {
        let mut evento = evento_valido();
        evento.monto_parte = Decimal::new(12345, 3);

        let error = evento.validar().unwrap_err();

        assert_eq!(error.campo, "monto_parte");
    }

    #[test]
    fn acepta_monto_cero() {
        let mut evento = evento_valido();
        evento.monto_parte = Decimal::ZERO;

        assert!(evento.validar().is_ok());
    }

    #[test]
    fn convierte_a_monto_local() {
        let evento = evento_valido();
        let monto = evento.monto();

        assert_eq!(monto.cantidad, Decimal::new(3334, 2));
        assert_eq!(monto.moneda, CodigoMoneda::USD);
    }

    #[test]
    fn serializa_y_deserializa_evento() {
        let evento = evento_valido();

        let json = serde_json::to_string(&evento).unwrap();

        assert!(json.contains("\"moneda\":\"USD\""));
        assert!(json.contains("\"monto_parte\":\"33.34\""));

        let recuperado: EventoCambioUsuario =
            serde_json::from_str(&json).unwrap();

        recuperado.validar().unwrap();

        assert_eq!(recuperado, evento);
    }

    #[test]
    fn rechaza_moneda_con_codigo_invalido() {
        let evento = evento_valido();
        let mut json = serde_json::to_value(&evento).unwrap();

        json["moneda"] = serde_json::json!("US1");

        let resultado =
            serde_json::from_value::<EventoCambioUsuario>(json);

        assert!(resultado.is_err());
    }

    #[test]
    fn deserializacion_no_sustituye_validacion() {
        let evento = evento_valido();
        let mut json = serde_json::to_value(&evento).unwrap();

        json["posicion"] = serde_json::json!(0);

        // Serde reconstruye la estructura, pero no ejecuta validar().
        let recuperado: EventoCambioUsuario =
            serde_json::from_value(json).unwrap();

        assert_eq!(
            recuperado.validar().unwrap_err().campo,
            "posicion"
        );
    }

    #[test]
    fn rechaza_fecha_invalida() {
        let evento = evento_valido();
        let mut json = serde_json::to_value(&evento).unwrap();

        json["fecha"] = serde_json::json!("fecha-invalida");

        let resultado =
            serde_json::from_value::<EventoCambioUsuario>(json);

        assert!(resultado.is_err());
    }
}
