use serde::{Deserialize, Serialize};

use super::cuenta::TipoCuenta;
use super::moneda::CodigoMoneda;
use super::monto::Monto;

/// Datos específicos asociados a la creación de una cuenta en el libro.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DatosCuentaCreada {
    pub id_cuenta: String,
    pub nombre: String,
    pub tipo: TipoCuenta,
    pub moneda: CodigoMoneda,
    pub saldo_inicial: Monto,
}

/// Evento inmutable del libro contable (Event Sourcing - INV-13).
///
/// Cada evento representa un hecho consumado que nunca se modifica ni se borra.
/// Contiene:
/// - `id`: Identificador único e inmutable del evento.
/// - `tipo`: Nombre o clasificador del tipo de evento (ej. "cuenta_creada").
/// - `momento_registro`: Marca temporal ISO 8601 en UTC del registro del evento.
/// - `dispositivo_origen`: Identificador del dispositivo donde se originó el evento.
/// - `datos`: Datos específicos del evento (`DatosCuentaCreada`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "clase_evento")]
pub enum EventoLibro {
    CuentaCreada {
        id: String,
        tipo: String,
        momento_registro: String,
        dispositivo_origen: String,
        datos: DatosCuentaCreada,
    },
}

impl EventoLibro {
    /// Constructor para un evento de creación de cuenta inmutable.
    pub fn nuevo_cuenta_creada(
        id: impl Into<String>,
        momento_registro: impl Into<String>,
        dispositivo_origen: impl Into<String>,
        datos: DatosCuentaCreada,
    ) -> Self {
        Self::CuentaCreada {
            id: id.into(),
            tipo: "cuenta_creada".to_string(),
            momento_registro: momento_registro.into(),
            dispositivo_origen: dispositivo_origen.into(),
            datos,
        }
    }

    /// Retorna el ID único del evento.
    pub fn id(&self) -> &str {
        match self {
            EventoLibro::CuentaCreada { id, .. } => id,
        }
    }

    /// Retorna el tipo del evento.
    pub fn tipo(&self) -> &str {
        match self {
            EventoLibro::CuentaCreada { tipo, .. } => tipo,
        }
    }

    /// Retorna el momento de registro (ISO 8601).
    pub fn momento_registro(&self) -> &str {
        match self {
            EventoLibro::CuentaCreada {
                momento_registro, ..
            } => momento_registro,
        }
    }

    /// Retorna el dispositivo origen donde se originó el evento.
    pub fn dispositivo_origen(&self) -> &str {
        match self {
            EventoLibro::CuentaCreada {
                dispositivo_origen, ..
            } => dispositivo_origen,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal::Decimal;

    #[test]
    fn test_creacion_evento_cuenta_creada() {
        let saldo = Monto::nuevo_saldo_inicial(Decimal::new(30000, 2), CodigoMoneda::USD).unwrap();
        let datos = DatosCuentaCreada {
            id_cuenta: "cta-123".to_string(),
            nombre: "Cuenta Ahorro".to_string(),
            tipo: TipoCuenta::Banco,
            moneda: CodigoMoneda::USD,
            saldo_inicial: saldo,
        };

        let evento = EventoLibro::nuevo_cuenta_creada(
            "evt-001",
            "2026-10-14T20:00:00Z",
            "disp-local-pc",
            datos.clone(),
        );

        assert_eq!(evento.id(), "evt-001");
        assert_eq!(evento.tipo(), "cuenta_creada");
        assert_eq!(evento.momento_registro(), "2026-10-14T20:00:00Z");
        assert_eq!(evento.dispositivo_origen(), "disp-local-pc");

        let EventoLibro::CuentaCreada { datos: d, .. } = &evento;
        assert_eq!(d, &datos);
    }

    #[test]
    fn test_serializacion_deserializacion_evento() {
        let saldo = Monto::cero(CodigoMoneda::VES);
        let datos = DatosCuentaCreada {
            id_cuenta: "cta-ves".to_string(),
            nombre: "Efectivo Bolívares".to_string(),
            tipo: TipoCuenta::Efectivo,
            moneda: CodigoMoneda::VES,
            saldo_inicial: saldo,
        };

        let evento =
            EventoLibro::nuevo_cuenta_creada("evt-999", "2026-10-14T21:00:00Z", "disp-01", datos);

        let json = serde_json::to_string(&evento).expect("Debe serializar a JSON");
        assert!(json.contains("\"id\":\"evt-999\""));
        assert!(json.contains("\"tipo\":\"cuenta_creada\""));

        let deserializado: EventoLibro =
            serde_json::from_str(&json).expect("Debe deserializar desde JSON");
        assert_eq!(evento, deserializado);
    }
}
