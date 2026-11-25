use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use super::categoria::{Categoria, TipoCategoria};
use super::cuenta::TipoCuenta;
use super::moneda::CodigoMoneda;
use super::monto::Monto;
use super::movimiento::TipoMovimiento;
use super::tasa_cambio::ParMonedas;
use super::dispositivo::DispositivoId;
use super::errores::ErrorDominio;

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

/// Datos específicos asociados a la creación de una nueva categoría en el libro contable (RF-LIB-07).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DatosCategoriaCreada {
    pub id_categoria: String,
    pub nombre: String,
    pub tipo: TipoCategoria,
}

/// Datos específicos asociados al registro de una nueva tasa de cambio.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DatosTasaRegistrada {
    pub par_monedas: ParMonedas,
    pub valor: Decimal,
    pub fecha: String,
    pub fuente: String,
}

/// Datos específicos asociados a la eliminación lógica de una tasa de cambio registrada.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DatosTasaEliminada {
    pub par_monedas: ParMonedas,
    pub fecha: String,
    pub fuente: String,
    #[serde(default)]
    pub id_tasa: Option<String>,
}

/// Datos específicos asociados al cambio de la moneda base de consolidación de la aplicación.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DatosMonedaBaseCambiada {
    pub nueva_moneda: CodigoMoneda,
}

/// Datos específicos asociados a la aplicación inmutable de una parte de gasto de grupo (INV-13, INV-15, RF-COM-10).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DatosGastoDeGrupoAplicado {
    pub id_evento_servidor: String,
    pub id_movimiento: String,
    pub id_cuenta: String,
    pub grupo_id: String,
    pub nombre_grupo: String,
    pub gasto_id: String,
    pub concepto: String,
    pub monto: Monto,
    pub fecha: String,
}

/// Datos específicos asociados a la creación de una moneda personalizada.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DatosMonedaCreada {
    pub codigo: CodigoMoneda,
    pub nombre: String,
    pub simbolo: String,
    pub decimales: u32,
}

/// Evento inmutable del libro contable (Event Sourcing - INV-13, INV-04).
///
/// Cada evento representa un hecho consumado que nunca se modifica ni se borra.
/// Contiene:
/// - `id`: Identificador único e inmutable del evento.
/// - `tipo`: Nombre o clasificador del tipo de evento (ej. "cuenta_creada", "movimiento_registrado", "movimiento_corregido", "categoria_creada").
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
    CategoriaCreada {
        id: String,
        tipo: String,
        momento_registro: String,
        dispositivo_origen: String,
        datos: DatosCategoriaCreada,
    },
    TasaRegistrada {
        id: String,
        tipo: String,
        momento_registro: String,
        dispositivo_origen: String,
        datos: DatosTasaRegistrada,
    },
    TasaEliminada {
        id: String,
        tipo: String,
        momento_registro: String,
        dispositivo_origen: String,
        datos: DatosTasaEliminada,
    },
    MonedaBaseCambiada {
        id: String,
        tipo: String,
        momento_registro: String,
        dispositivo_origen: String,
        datos: DatosMonedaBaseCambiada,
    },
    MonedaCreada {
        id: String,
        tipo: String,
        momento_registro: String,
        dispositivo_origen: String,
        datos: DatosMonedaCreada,
    },
    GastoDeGrupoAplicado {
        id: String,
        tipo: String,
        momento_registro: String,
        dispositivo_origen: String,
        datos: DatosGastoDeGrupoAplicado,
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

    /// Constructor para un evento de creación de categoría inmutable (RF-LIB-07).
    pub fn nuevo_categoria_creada(
        id: impl Into<String>,
        momento_registro: impl Into<String>,
        dispositivo_origen: impl Into<String>,
        datos: DatosCategoriaCreada,
    ) -> Self {
        Self::CategoriaCreada {
            id: id.into(),
            tipo: "categoria_creada".to_string(),
            momento_registro: momento_registro.into(),
            dispositivo_origen: dispositivo_origen.into(),
            datos,
        }
    }

    /// Constructor para un evento de registro de tasa nueva
    pub fn nuevo_tasa_registrada(
        id: impl Into<String>,
        momento_registro: impl Into<String>,
        dispositivo_origen: impl Into<String>,
        datos: DatosTasaRegistrada,
    ) -> Self {
        Self::TasaRegistrada {
            id: id.into(),
            tipo: "tasa_registrada".to_string(),
            momento_registro: momento_registro.into(),
            dispositivo_origen: dispositivo_origen.into(),
            datos,
        }
    }

    /// Constructor para un evento de eliminación de tasa nueva
    pub fn nuevo_tasa_eliminada(
        id: impl Into<String>,
        momento_registro: impl Into<String>,
        dispositivo_origen: impl Into<String>,
        datos: DatosTasaEliminada,
    ) -> Self {
        Self::TasaEliminada {
            id: id.into(),
            tipo: "tasa_eliminada".to_string(),
            momento_registro: momento_registro.into(),
            dispositivo_origen: dispositivo_origen.into(),
            datos,
        }
    }

    /// Constructor para un evento de cambio de moneda base
    pub fn nuevo_moneda_base_cambiada(
        id: impl Into<String>,
        momento_registro: impl Into<String>,
        dispositivo_origen: impl Into<String>,
        datos: DatosMonedaBaseCambiada,
    ) -> Self {
        Self::MonedaBaseCambiada {
            id: id.into(),
            tipo: "moneda_base_cambiada".to_string(),
            momento_registro: momento_registro.into(),
            dispositivo_origen: dispositivo_origen.into(),
            datos,
        }
    }

    /// Constructor para un evento de creación de moneda
    pub fn nuevo_moneda_creada(
        id: impl Into<String>,
        momento_registro: impl Into<String>,
        dispositivo_origen: impl Into<String>,
        datos: DatosMonedaCreada,
    ) -> Self {
        Self::MonedaCreada {
            id: id.into(),
            tipo: "moneda_creada".to_string(),
            momento_registro: momento_registro.into(),
            dispositivo_origen: dispositivo_origen.into(),
            datos,
        }
    }

    /// Constructor para un evento inmutable de parte de gasto de grupo aplicada (RF-COM-10, INV-13, INV-15).
    pub fn nuevo_gasto_de_grupo_aplicado(
        momento_registro: impl Into<String>,
        dispositivo_origen: impl Into<String>,
        datos: DatosGastoDeGrupoAplicado,
    ) -> Result<Self, ErrorDominio> {
        let id_servidor = datos.id_evento_servidor.trim();
        if id_servidor.is_empty() {
            return Err(ErrorDominio::evento_invalido(
                "El id_evento_servidor no puede estar vacío.",
            ));
        }

        Ok(Self::GastoDeGrupoAplicado {
            id: format!("gasto_grupo:{}", id_servidor),
            tipo: "gasto_de_grupo_aplicado".to_string(),
            momento_registro: momento_registro.into(),
            dispositivo_origen: dispositivo_origen.into(),
            datos,
        })
    }

    /// Retorna el ID único del evento.
    pub fn id(&self) -> &str {
        match self {
            EventoLibro::CuentaCreada { id, .. }
            | EventoLibro::MovimientoRegistrado { id, .. }
            | EventoLibro::MovimientoCorregido { id, .. }
            | EventoLibro::CategoriaCreada { id, .. }
            | EventoLibro::TasaRegistrada { id, .. }
            | EventoLibro::TasaEliminada { id, .. }
            | EventoLibro::MonedaBaseCambiada { id, .. }
            | EventoLibro::MonedaCreada { id, .. }
            | EventoLibro::GastoDeGrupoAplicado { id, .. } => id,
        }
    }

    /// Retorna el tipo de evento en texto descriptivo.
    pub fn tipo(&self) -> &str {
        match self {
            EventoLibro::CuentaCreada { tipo, .. }
            | EventoLibro::MovimientoRegistrado { tipo, .. }
            | EventoLibro::MovimientoCorregido { tipo, .. }
            | EventoLibro::CategoriaCreada { tipo, .. }
            | EventoLibro::TasaRegistrada { tipo, .. }
            | EventoLibro::TasaEliminada { tipo, .. }
            | EventoLibro::MonedaBaseCambiada { tipo, .. }
            | EventoLibro::MonedaCreada { tipo, .. }
            | EventoLibro::GastoDeGrupoAplicado { tipo, .. } => tipo,
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
            }
            | EventoLibro::CategoriaCreada {
                momento_registro, ..
            }
            | EventoLibro::TasaRegistrada {
                momento_registro, ..
            }
            | EventoLibro::TasaEliminada {
                momento_registro, ..
            }
            | EventoLibro::MonedaBaseCambiada {
                momento_registro, ..
            }
            | EventoLibro::MonedaCreada {
                momento_registro, ..
            }
            | EventoLibro::GastoDeGrupoAplicado {
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
            }
            | EventoLibro::CategoriaCreada {
                dispositivo_origen, ..
            }
            | EventoLibro::TasaRegistrada {
                dispositivo_origen, ..
            }
            | EventoLibro::TasaEliminada {
                dispositivo_origen, ..
            }
            | EventoLibro::MonedaBaseCambiada {
                dispositivo_origen, ..
            }
            | EventoLibro::MonedaCreada {
                dispositivo_origen, ..
            }
            | EventoLibro::GastoDeGrupoAplicado {
                dispositivo_origen, ..
            } => dispositivo_origen,
        }
    }

    /// Retorna el identificador de la cuenta afectada por el evento, si aplica.
    pub fn id_cuenta(&self) -> Option<&str> {
        match self {
            EventoLibro::CuentaCreada { datos, .. } => Some(&datos.id_cuenta),
            EventoLibro::MovimientoRegistrado { datos, .. } => Some(&datos.id_cuenta),
            EventoLibro::MovimientoCorregido { datos, .. } => Some(&datos.id_cuenta),
            EventoLibro::GastoDeGrupoAplicado { datos, .. } => Some(&datos.id_cuenta),
            EventoLibro::CategoriaCreada { .. }
            | EventoLibro::TasaRegistrada { .. }
            | EventoLibro::TasaEliminada { .. }
            | EventoLibro::MonedaBaseCambiada { .. }
            | EventoLibro::MonedaCreada { .. } => None,
        }
    }


    /// Valida los metadatos obligatorios de integridad del evento.
    pub fn validar(&self) -> Result<(), ErrorDominio> {
        if self.id().trim().is_empty() {
            return Err(ErrorDominio::evento_invalido(
                "El id del evento no puede estar vacío.",
            ));
        }

        if self.momento_registro().trim().is_empty() {
            return Err(ErrorDominio::evento_invalido(
                "El momento de registro del evento no puede estar vacío.",
            ));
        }
        // Reutiliza exactamente las reglas de validación de DispositivoId
        DispositivoId::nuevo(self.dispositivo_origen())?;

        Ok(())
    }

    /// Compara dos eventos según el orden determinista del libro:
    /// `(momento_registro, dispositivo_origen, id)`.
    pub fn comparar_determinista(&self, otro: &Self) -> std::cmp::Ordering {
        (self.momento_registro(), self.dispositivo_origen(), self.id()).cmp(&(
            otro.momento_registro(),
            otro.dispositivo_origen(),
            otro.id(),
        ))
    }

    /// Ordena un slice mutable de eventos en sitio siguiendo el orden determinista:
    /// `(momento_registro, dispositivo_origen, id)`.
    pub fn ordenar_en_sitio(eventos: &mut [EventoLibro]) {
        eventos.sort_by(Self::comparar_determinista);
    }

    /// Devuelve las referencias a los eventos ordenadas de forma determinista:
    /// `(momento_registro, dispositivo_origen, id)`.
    pub fn ordenar_determinista<'a>(eventos: &'a [EventoLibro]) -> Vec<&'a EventoLibro> {
        let mut ordenados: Vec<&'a EventoLibro> = eventos.iter().collect();
        ordenados.sort_by(|a, b| a.comparar_determinista(b));
        ordenados
    }

    /// Fusiona dos conjuntos de eventos de forma determinista, idempotente (INV-13)
    /// y conmutativa (INV-14).
    ///
    /// - Deduplica por `id` conservando la primera aparición.
    /// - Ordena el resultado por `(momento_registro, dispositivo_origen, id)`.
    pub fn fusionar(locales: &[EventoLibro], remotos: &[EventoLibro]) -> Vec<EventoLibro> {
        let mut ids_vistos = std::collections::HashSet::with_capacity(locales.len() + remotos.len());
        let mut fusionados = Vec::with_capacity(locales.len() + remotos.len());

        for evento in locales.iter().chain(remotos.iter()) {
            if ids_vistos.insert(evento.id().to_string()) {
                fusionados.push(evento.clone());
            }
        }

        Self::ordenar_en_sitio(&mut fusionados);
        fusionados
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal::Decimal;

    #[test]
    fn test_creacion_evento_cuenta_creada() {
        let saldo =
            Monto::nuevo_saldo_inicial(Decimal::new(30000, 2), CodigoMoneda::USD, 2).unwrap();
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
        assert_eq!(evento.id_cuenta(), Some("cta-123"));

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
        assert_eq!(evento.id_cuenta(), Some("cta-123"));

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
        assert_eq!(evento.id_cuenta(), Some("cta-123"));

        let EventoLibro::MovimientoCorregido { datos: d, .. } = &evento else {
            panic!("Tipo de evento incorrecto");
        };
        assert_eq!(d, &datos);
    }

    #[test]
    fn test_creacion_evento_categoria_creada() {
        let datos = DatosCategoriaCreada {
            id_categoria: "cursos_online".to_string(),
            nombre: "Cursos Online".to_string(),
            tipo: TipoCategoria::Gasto,
        };

        let evento = EventoLibro::nuevo_categoria_creada(
            "evt-cat-01",
            "2026-10-24T12:00:00Z",
            "disp-local",
            datos.clone(),
        );

        assert_eq!(evento.id(), "evt-cat-01");
        assert_eq!(evento.tipo(), "categoria_creada");
        assert_eq!(evento.id_cuenta(), None);

        let EventoLibro::CategoriaCreada { datos: d, .. } = &evento else {
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

    #[test]
    fn test_creacion_evento_gasto_de_grupo_aplicado() {
        let monto = Monto::new(
            Decimal::new(3334, 2),
            CodigoMoneda::USD,
        );

        let datos = DatosGastoDeGrupoAplicado {
            id_evento_servidor: "evento-servidor-1".into(),
            id_movimiento: "mov-grupo-1".into(),
            id_cuenta: "cuenta-usd-1".into(),
            grupo_id: "grupo-1".into(),
            nombre_grupo: "Viaje a Mérida".into(),
            gasto_id: "gasto-1".into(),
            concepto: "Hotel".into(),
            monto,
            fecha: "2026-11-22T12:00:00Z".into(),
        };

        let evento = EventoLibro::nuevo_gasto_de_grupo_aplicado(
            "2026-11-22T13:00:00Z",
            "dispositivo-1",
            datos.clone(),
        ).unwrap();

        assert_eq!(evento.id(), "gasto_grupo:evento-servidor-1");
        assert_eq!(evento.tipo(), "gasto_de_grupo_aplicado");
        assert_eq!(evento.id_cuenta(), Some("cuenta-usd-1"));

        let EventoLibro::GastoDeGrupoAplicado {
            datos: datos_evento,
            ..
        } = &evento else {
            panic!("Tipo de evento incorrecto");
        };

        assert_eq!(datos_evento, &datos);
        assert_eq!(
            datos_evento.id_evento_servidor,
            "evento-servidor-1"
        );
        assert_eq!(datos_evento.monto.cantidad, Decimal::new(3334, 2));
    }

    #[test]
    fn ordenar_determinista_respeta_tupla_momento_dispositivo_id() {
        let cat_a = EventoLibro::nuevo_categoria_creada(
            "evt-b",
            "2026-10-14T10:00:00Z",
            "disp-2",
            DatosCategoriaCreada {
                id_categoria: "c1".into(),
                nombre: "Cat 1".into(),
                tipo: crate::dominio::categoria::TipoCategoria::Gasto,
            },
        );

        let cat_b = EventoLibro::nuevo_categoria_creada(
            "evt-a",
            "2026-10-14T10:00:00Z",
            "disp-2",
            DatosCategoriaCreada {
                id_categoria: "c2".into(),
                nombre: "Cat 2".into(),
                tipo: crate::dominio::categoria::TipoCategoria::Gasto,
            },
        );

        let cat_c = EventoLibro::nuevo_categoria_creada(
            "evt-z",
            "2026-10-14T10:00:00Z",
            "disp-1",
            DatosCategoriaCreada {
                id_categoria: "c3".into(),
                nombre: "Cat 3".into(),
                tipo: crate::dominio::categoria::TipoCategoria::Gasto,
            },
        );

        let cat_d = EventoLibro::nuevo_categoria_creada(
            "evt-0",
            "2026-10-14T09:59:59Z",
            "disp-9",
            DatosCategoriaCreada {
                id_categoria: "c4".into(),
                nombre: "Cat 4".into(),
                tipo: crate::dominio::categoria::TipoCategoria::Gasto,
            },
        );

        // Orden esperado:
        // 1. evt-0 (momento más antiguo: 09:59:59Z)
        // 2. evt-z (10:00:00Z, disp-1 < disp-2)
        // 3. evt-a (10:00:00Z, disp-2, id evt-a < evt-b)
        // 4. evt-b (10:00:00Z, disp-2, id evt-b)
        let desordenados = vec![cat_a, cat_b, cat_c, cat_d];
        let ordenados = EventoLibro::ordenar_determinista(&desordenados);
        let ids: Vec<&str> = ordenados.iter().map(|e| e.id()).collect();

        assert_eq!(ids, vec!["evt-0", "evt-z", "evt-a", "evt-b"]);
    }

    #[test]
    fn fusionar_es_idempotente_conmutativo_y_asociativo() {
        let ev1 = EventoLibro::nuevo_categoria_creada(
            "evt-1",
            "2026-10-14T10:00:00Z",
            "disp-a",
            DatosCategoriaCreada {
                id_categoria: "c1".into(),
                nombre: "Comida".into(),
                tipo: crate::dominio::categoria::TipoCategoria::Gasto,
            },
        );
        let ev2 = EventoLibro::nuevo_categoria_creada(
            "evt-2",
            "2026-10-14T10:00:00Z",
            "disp-b",
            DatosCategoriaCreada {
                id_categoria: "c2".into(),
                nombre: "Transporte".into(),
                tipo: crate::dominio::categoria::TipoCategoria::Gasto,
            },
        );
        let ev3 = EventoLibro::nuevo_categoria_creada(
            "evt-3",
            "2026-10-14T10:05:00Z",
            "disp-a",
            DatosCategoriaCreada {
                id_categoria: "c3".into(),
                nombre: "Salud".into(),
                tipo: crate::dominio::categoria::TipoCategoria::Gasto,
            },
        );

        let conj_a = vec![ev1.clone(), ev2.clone()];
        let conj_b = vec![ev2.clone(), ev3.clone()];
        let conj_c = vec![ev3, ev1];

        // INV-13: Idempotencia
        let esperado_a: Vec<EventoLibro> = EventoLibro::ordenar_determinista(&conj_a)
            .into_iter()
            .cloned()
            .collect();
        assert_eq!(EventoLibro::fusionar(&conj_a, &conj_a), esperado_a);
        let ab = EventoLibro::fusionar(&conj_a, &conj_b);
        assert_eq!(EventoLibro::fusionar(&ab, &conj_b), ab);

        // INV-14: Conmutatividad (A ∪ B == B ∪ A)
        let ba = EventoLibro::fusionar(&conj_b, &conj_a);
        assert_eq!(ab, ba);

        // Asociatividad ((A ∪ B) ∪ C == A ∪ (B ∪ C))
        let ab_c = EventoLibro::fusionar(&ab, &conj_c);
        let bc = EventoLibro::fusionar(&conj_b, &conj_c);
        let a_bc = EventoLibro::fusionar(&conj_a, &bc);
        assert_eq!(ab_c, a_bc);
    }

}
