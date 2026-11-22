use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EventoServidorDto {
    pub id: String,
    pub posicion: i64,
    pub usuario_id: String,
    pub grupo_id: String,
    pub nombre_grupo: String,
    pub gasto_id: String,
    pub concepto: String,
    pub fecha: String,
    pub moneda: String,
    pub monto_parte: String,
    pub incluye_redondeo: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CambiosServidorRespuestaDto {
    pub eventos: Vec<EventoServidorDto>,
    pub nuevo_cursor: i64,
    pub quedan_mas: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResultadoSincronizacionDto {
    pub eventos_aplicados: usize,
    pub partes_pendientes: usize,
    pub nuevo_cursor: i64,
    pub quedan_mas: bool,
}

/// Puerto secundario: Cliente HTTP hacia GET /cambios?desde=<cursor>
pub trait ClienteSincronizacion: Send + Sync {
    fn obtener_cambios(&self, desde_cursor: i64) -> Result<CambiosServidorRespuestaDto, String>;
}

/// Puerto secundario: Persistencia local SQLite
pub trait RepositorioSincronizacionLocal: Send + Sync {
    fn obtener_cursor(&self) -> Result<i64, String>;
    fn aplicar_lote(
        &mut self,
        eventos: &[EventoServidorDto],
        nuevo_cursor: i64,
        momento_actual: &str,
        dispositivo: &str,
    ) -> Result<(usize, usize), String>; // (aplicados, pendientes)
}

pub struct SincronizarCambios<'a> {
    cliente_sync: &'a dyn ClienteSincronizacion,
    repositorio: &'a mut dyn RepositorioSincronizacionLocal,
}

impl<'a> SincronizarCambios<'a> {
    pub fn new(
        cliente_sync: &'a dyn ClienteSincronizacion,
        repositorio: &'a mut dyn RepositorioSincronizacionLocal,
    ) -> Self {
        Self { cliente_sync, repositorio }
    }

    pub fn ejecutar(
        &mut self,
        momento_actual: &str,
        dispositivo: &str,
    ) -> Result<ResultadoSincronizacionDto, String> {
        let cursor_local = self.repositorio.obtener_cursor()?;

        let respuesta_servidor = self
            .cliente_sync
            .obtener_cambios(cursor_local)?;

        let (aplicados, pendientes) = self.repositorio.aplicar_lote(
            &respuesta_servidor.eventos,
            respuesta_servidor.nuevo_cursor,
            momento_actual,
            dispositivo,
        )?;

        Ok(ResultadoSincronizacionDto {
            eventos_aplicados: aplicados,
            partes_pendientes: pendientes,
            nuevo_cursor: respuesta_servidor.nuevo_cursor,
            quedan_mas: respuesta_servidor.quedan_mas,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    struct MockCliente {
        respuesta: Mutex<Option<CambiosServidorRespuestaDto>>,
        error: Option<String>,
    }

    impl ClienteSincronizacion for MockCliente {
        fn obtener_cambios(&self, _desde: i64) -> Result<CambiosServidorRespuestaDto, String> {
            if let Some(err) = &self.error {
                return Err(err.clone());
            }
            Ok(self.respuesta.lock().unwrap().take().unwrap_or_else(|| CambiosServidorRespuestaDto {
                eventos: vec![],
                nuevo_cursor: 0,
                quedan_mas: false,
            }))
        }
    }

    struct MockRepo {
        cursor_guardado: i64,
        lotes_recibidos: Vec<Vec<EventoServidorDto>>,
    }

    impl RepositorioSincronizacionLocal for MockRepo {
        fn obtener_cursor(&self) -> Result<i64, String> {
            Ok(self.cursor_guardado)
        }

        fn aplicar_lote(
            &mut self,
            eventos: &[EventoServidorDto],
            nuevo_cursor: i64,
            _momento: &str,
            _dispositivo: &str,
        ) -> Result<(usize, usize), String> {
            self.lotes_recibidos.push(eventos.to_vec());
            self.cursor_guardado = nuevo_cursor;
            Ok((eventos.len(), 0))
        }
    }

    #[test]
    fn test_sincronizacion_exitosa_avanza_cursor() {
        let cliente = MockCliente {
            respuesta: Mutex::new(Some(CambiosServidorRespuestaDto {
                eventos: vec![EventoServidorDto {
                    id: "ev-1".into(),
                    posicion: 5,
                    usuario_id: "u-1".into(),
                    grupo_id: "g-1".into(),
                    nombre_grupo: "Viaje".into(),
                    gasto_id: "gst-1".into(),
                    concepto: "Hotel".into(),
                    fecha: "2026-11-22T10:00:00Z".into(),
                    moneda: "USD".into(),
                    monto_parte: "50.00".into(),
                    incluye_redondeo: false,
                }],
                nuevo_cursor: 5,
                quedan_mas: false,
            })),
            error: None,
        };
        let mut repo = MockRepo {
            cursor_guardado: 0,
            lotes_recibidos: vec![],
        };

        let mut caso_uso = SincronizarCambios::new(&cliente, &mut repo);
        let res = caso_uso.ejecutar("2026-11-22T12:00:00Z", "app-desktop").unwrap();

        assert_eq!(res.eventos_aplicados, 1);
        assert_eq!(res.nuevo_cursor, 5);
        assert_eq!(repo.cursor_guardado, 5);
    }

    #[test]
    fn test_falla_red_no_modifica_cursor() {
        let cliente = MockCliente {
            respuesta: Mutex::new(None),
            error: Some("Error de conexión".into()),
        };
        let mut repo = MockRepo {
            cursor_guardado: 2,
            lotes_recibidos: vec![],
        };

        let mut caso_uso = SincronizarCambios::new(&cliente, &mut repo);
        let error = caso_uso.ejecutar("2026-11-22T12:00:00Z", "app-desktop").unwrap_err();

        assert_eq!(error, "Error de conexión");
        assert_eq!(repo.cursor_guardado, 2);
    }
}