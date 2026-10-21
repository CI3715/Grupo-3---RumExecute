use serde::{Deserialize, Serialize};

use super::categoria::Categoria;
use super::cuenta::TipoCuenta;
use super::moneda::CodigoMoneda;
use super::monto::Monto;
use super::movimiento::TipoMovimiento;

/// Datos específicos asociados a la creación de una cuenta en el libro.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DatosCuentaCreada {
    pub id_cuenta: String,
    pub nombre: String,
    pub tipo: TipoCuenta,
    pub moneda: CodigoMoneda,
    pub saldo_inicial: Monto,
}

/// Datos específicos asociados al registro de un movimiento contable (Gasto o Ingreso).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DatosMovimientoRegistrado {
    pub id_movimiento: String,
    pub id_cuenta: String,
    pub tipo: TipoMovimiento,
    pub monto: Monto,
    pub categoria: Option<Categoria>,
    pub fecha: String,
    pub descripcion: String,
}

/// Datos específicos asociados a la corrección inmutable de un movimiento mediante Ajuste (INV-04).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DatosMovimientoCorregido {
    pub id_ajuste: String,
    pub id_cuenta: String,
    pub id_movimiento_original: String,
    pub monto_ajuste: Monto,
    pub monto_original: Monto,
    pub monto_nuevo: Monto,
    pub fecha: String,
    pub descripcion: String,
}

/// Evento inmutable del libro contable (Event Sourcing - INV-13, INV-04).
///
/// Cada evento representa un hecho consumado que nunca se modifica ni se borra.
/// Contiene:
/// - `id`: Identificador único e inmutable del evento.
/// - `tipo`: Nombre o clasificador del tipo de evento (ej. "cuenta_creada", "movimiento_registrado", "movimiento_corregido").
/// - `momento_registro`: Marca temporal ISO 8601 en UTC del registro del evento.
/// - `dispositivo_origen`: Identificador del dispositivo donde se originó el evento.
/// - `datos`: Carga útil específica de cada clase de evento.
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
    MovimientoRegistrado {
        id: String,
        tipo: String,
        momento_registro: String,
        dispositivo_origen: String,
        datos: DatosMovimientoRegistrado,
    },
    MovimientoCorregido {
        id: String,
        tipo: String,
        momento_registro: String,
        dispositivo_origen: String,
        datos: DatosMovimientoCorregido,
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

    /// Constructor para un evento de registro de movimiento (Gasto o Ingreso) inmutable.
    pub fn nuevo_movimiento_registrado(
        id: impl Into<String>,
        momento_registro: impl Into<String>,
        dispositivo_origen: impl Into<String>,
        datos: DatosMovimientoRegistrado,
    ) -> Self {
        Self::MovimientoRegistrado {
            id: id.into(),
            tipo: "movimiento_registrado".to_string(),
            momento_registro: momento_registro.into(),
            dispositivo_origen: dispositivo_origen.into(),
            datos,
        }
    }

    /// Constructor para un evento de corrección de movimiento mediante Ajuste (INV-04).
    pub fn nuevo_movimiento_corregido(
        id: impl Into<String>,
        momento_registro: impl Into<String>,
        dispositivo_origen: impl Into<String>,
        datos: DatosMovimientoCorregido,
    ) -> Self {
        Self::MovimientoCorregido {
            id: id.into(),
            tipo: "movimiento_corregido".to_string(),
            momento_registro: momento_registro.into(),
            dispositivo_origen: dispositivo_origen.into(),
            datos,
        }
    }

    /// Retorna el ID único del evento.
    pub fn id(&self) -> &str {
        match self {
            EventoLibro::CuentaCreada { id, .. }
            | EventoLibro::MovimientoRegistrado { id, .. }
            | EventoLibro::MovimientoCorregido { id, .. } => id,
        }
    }

    /// Retorna el tipo de evento en texto descriptivo.
    pub fn tipo(&self) -> &str {
        match self {
            EventoLibro::CuentaCreada { tipo, .. }
            | EventoLibro::MovimientoRegistrado { tipo, .. }
            | EventoLibro::MovimientoCorregido { tipo, .. } => tipo,
        }
    }

    /// Retorna el momento de registro (ISO 8601).
    pub fn momento_registro(&self) -> &str {
        match self {
            EventoLibro::CuentaCreada {
                momento_registro, ..
            }
            | EventoLibro::MovimientoRegistrado {
                momento_registro, ..
            }
            | EventoLibro::MovimientoCorregido {
                momento_registro, ..
            } => momento_registro,
        }
    }

    /// Retorna el dispositivo origen donde se originó el evento.
    pub fn dispositivo_origen(&self) -> &str {
        match self {
            EventoLibro::CuentaCreada {
                dispositivo_origen, ..
            }
            | EventoLibro::MovimientoRegistrado {
                dispositivo_origen, ..
            }
            | EventoLibro::MovimientoCorregido {
                dispositivo_origen, ..
            } => dispositivo_origen,
        }
    }

    /// Retorna el identificador de la cuenta afectada por el evento.
    pub fn id_cuenta(&self) -> &str {
        match self {
            EventoLibro::CuentaCreada { datos, .. } => &datos.id_cuenta,
            EventoLibro::MovimientoRegistrado { datos, .. } => &datos.id_cuenta,
            EventoLibro::MovimientoCorregido { datos, .. } => &datos.id_cuenta,
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
        assert_eq!(evento.id_cuenta(), "cta-123");

        let EventoLibro::CuentaCreada { datos: d, .. } = &evento else {
            panic!("Tipo de evento incorrecto");
        };
        assert_eq!(d, &datos);
    }

    #[test]
    fn test_creacion_evento_movimiento_registrado() {
        let monto = Monto::new(Decimal::new(4550, 2), CodigoMoneda::USD);
        let datos = DatosMovimientoRegistrado {
            id_movimiento: "mov-01".to_string(),
            id_cuenta: "cta-123".to_string(),
            tipo: TipoMovimiento::Gasto,
            monto,
            categoria: None,
            fecha: "2026-10-20".to_string(),
            descripcion: "Almuerzo".to_string(),
        };

        let evento = EventoLibro::nuevo_movimiento_registrado(
            "evt-002",
            "2026-10-20T12:30:00Z",
            "disp-01",
            datos.clone(),
        );

        assert_eq!(evento.id(), "evt-002");
        assert_eq!(evento.tipo(), "movimiento_registrado");
        assert_eq!(evento.id_cuenta(), "cta-123");

        let EventoLibro::MovimientoRegistrado { datos: d, .. } = &evento else {
            panic!("Tipo de evento incorrecto");
        };
        assert_eq!(d, &datos);
    }

    #[test]
    fn test_creacion_evento_movimiento_corregido() {
        let monto_ajuste = Monto::new(Decimal::new(45000, 2), CodigoMoneda::USD); // +450.00
        let monto_original = Monto::new(Decimal::new(50000, 2), CodigoMoneda::USD); // 500.00
        let monto_nuevo = Monto::new(Decimal::new(5000, 2), CodigoMoneda::USD); // 50.00

        let datos = DatosMovimientoCorregido {
            id_ajuste: "aj-01".to_string(),
            id_cuenta: "cta-123".to_string(),
            id_movimiento_original: "mov-500".to_string(),
            monto_ajuste,
            monto_original,
            monto_nuevo,
            fecha: "2026-10-20".to_string(),
            descripcion: "Corrección a 50".to_string(),
        };

        let evento = EventoLibro::nuevo_movimiento_corregido(
            "evt-003",
            "2026-10-20T13:00:00Z",
            "disp-01",
            datos.clone(),
        );

        assert_eq!(evento.id(), "evt-003");
        assert_eq!(evento.tipo(), "movimiento_corregido");
        assert_eq!(evento.id_cuenta(), "cta-123");

        let EventoLibro::MovimientoCorregido { datos: d, .. } = &evento else {
            panic!("Tipo de evento incorrecto");
        };
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
