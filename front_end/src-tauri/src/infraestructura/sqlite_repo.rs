use std::collections::{HashMap, HashSet};
use std::path::Path;
use std::str::FromStr;

use rusqlite::{params, Connection, OptionalExtension, TransactionBehavior};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use crate::aplicacion::puertos::sincronizacion::{
    DireccionLoteSincronizacion, RepositorioLibroSincronizable,
};
use crate::aplicacion::puertos::{ErrorRepositorio, RepositorioLibro};
use crate::dominio::evento::{DatosGastoDeGrupoAplicado, EventoLibro, ResultadoFusion};
use crate::dominio::moneda::CodigoMoneda;
use crate::dominio::monto::Monto;

// ============================================================================
// Modelos DTO de Persistencia Local SQLite (Entrega 7)
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CuentaPorGrupo {
    pub grupo_id: String,
    pub cuenta_id_personal: String,
    pub moneda: String,
    pub actualizado_en: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PartePendienteLocal {
    pub id: String,
    pub id_evento_servidor: String,
    pub posicion: i64,
    pub grupo_id: String,
    pub nombre_grupo: String,
    pub gasto_id: String,
    pub concepto: String,
    pub monto: String,
    pub moneda: String,
    pub fecha: String,
    pub incluye_redondeo: bool,
    pub recibido_en: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperacionGastoOffline {
    pub id: String,
    pub idempotency_key: String,
    pub grupo_id: String,
    pub concepto: String,
    pub monto: String,
    pub pagador_id: String,
    pub regla_division: String,
    pub participantes: Vec<String>,
    pub valores: Option<HashMap<String, String>>,
    pub creado_en: String,
    pub estado: String,
    pub error_mensaje: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventoServidorRecibido {
    pub id: String,
    pub posicion: i64,
    pub usuario_id: String,
    pub grupo_id: String,
    pub nombre_grupo: String,
    pub gasto_id: String,
    pub concepto: String,
    pub fecha: String,
    pub moneda: String,
    pub monto_parte: Decimal,
    pub incluye_redondeo: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResultadoSincronizacionLocal {
    pub eventos_aplicados: usize,
    pub partes_pendientes: usize,
    pub nuevo_cursor: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TipoDestinoSincronizacion {
    Carpeta,
    Git,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DestinoSincronizacion {
    pub tipo: TipoDestinoSincronizacion,
    pub ubicacion: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EstadoSincronizacionPersonal {
    pub dispositivo_id: String,
    pub destino: Option<DestinoSincronizacion>,
    pub fecha_ultima_sincronizacion: Option<String>,
}

// ============================================================================
// Repositorio SQLite
// ============================================================================

pub struct RepositorioLibroSqlite {
    pub(crate) conexion: Connection,
}

impl RepositorioLibroSqlite {
    pub fn abrir(ruta: impl AsRef<Path>) -> Result<Self, ErrorRepositorio> {
        let conexion = Connection::open(ruta)
            .map_err(|error| ErrorRepositorio(format!("No se pudo abrir SQLite: {error}")))?;

        conexion
            .execute_batch(
                "
                PRAGMA foreign_keys = ON;

                -- 1. Libro contable append-only protegido por triggers (Entregas 1-4)
                CREATE TABLE IF NOT EXISTS eventos_libro (
                    secuencia INTEGER PRIMARY KEY AUTOINCREMENT,
                    id TEXT NOT NULL UNIQUE,
                    tipo TEXT NOT NULL,
                    momento_registro TEXT NOT NULL,
                    dispositivo_origen TEXT NOT NULL,
                    evento_json TEXT NOT NULL
                );
                CREATE TRIGGER IF NOT EXISTS eventos_libro_no_actualizar
                BEFORE UPDATE ON eventos_libro
                BEGIN
                    SELECT RAISE(ABORT, 'Los eventos del libro son inmutables');
                END;
                CREATE TRIGGER IF NOT EXISTS eventos_libro_no_eliminar
                BEFORE DELETE ON eventos_libro
                BEGIN
                    SELECT RAISE(ABORT, 'Los eventos del libro son inmutables');
                END;

                -- 2. Índice único de eventos del servidor ya aplicados (INV-13, INV-15)
                CREATE TABLE IF NOT EXISTS local_eventos_servidor_aplicados (
                    id_evento_servidor TEXT PRIMARY KEY,
                    evento_libro_id TEXT NOT NULL,
                    aplicado_en TEXT NOT NULL
                );

                -- 3. Elección local de cuenta personal por grupo (INV-16, INV-18)
                CREATE TABLE IF NOT EXISTS local_cuentas_por_grupo (
                    grupo_id TEXT PRIMARY KEY,
                    cuenta_id_personal TEXT NOT NULL,
                    moneda TEXT NOT NULL,
                    actualizado_en TEXT NOT NULL
                );

                -- 4. Partes pendientes de aplicar cuando un grupo no tiene cuenta elegida (INV-18)
                CREATE TABLE IF NOT EXISTS local_partes_pendientes (
                    id TEXT PRIMARY KEY,
                    id_evento_servidor TEXT NOT NULL UNIQUE,
                    posicion INTEGER NOT NULL,
                    grupo_id TEXT NOT NULL,
                    nombre_grupo TEXT NOT NULL,
                    gasto_id TEXT NOT NULL,
                    concepto TEXT NOT NULL,
                    monto TEXT NOT NULL,
                    moneda TEXT NOT NULL,
                    fecha TEXT NOT NULL,
                    incluye_redondeo INTEGER NOT NULL DEFAULT 0,
                    recibido_en TEXT NOT NULL
                );

                -- 5. Control de cursor de sincronización unidireccional (INV-14)
                CREATE TABLE IF NOT EXISTS local_sincronizacion_cursor (
                    id INTEGER PRIMARY KEY,
                    ultimo_cursor INTEGER NOT NULL DEFAULT 0,
                    fecha_ultima_sincronizacion TEXT
                );
                INSERT OR IGNORE INTO local_sincronizacion_cursor (id, ultimo_cursor, fecha_ultima_sincronizacion)
                VALUES (1, 0, NULL);

                -- 6. Cola local de gastos sin conexión (RNF-17, ESC-09)
                CREATE TABLE IF NOT EXISTS local_cola_gastos_offline (
                    id TEXT PRIMARY KEY,
                    idempotency_key TEXT NOT NULL UNIQUE,
                    grupo_id TEXT NOT NULL,
                    concepto TEXT NOT NULL,
                    monto TEXT NOT NULL,
                    pagador_id TEXT NOT NULL,
                    regla_division TEXT NOT NULL,
                    participantes_json TEXT NOT NULL,
                    valores_json TEXT,
                    creado_en TEXT NOT NULL,
                    estado TEXT NOT NULL DEFAULT 'PENDIENTE',
                    error_mensaje TEXT
                );

                CREATE TABLE IF NOT EXISTS local_e8_configuracion (
                    id INTEGER PRIMARY KEY CHECK (id = 1),
                    dispositivo_id TEXT NOT NULL,
                    tipo_destino TEXT,
                    ubicacion_destino TEXT,
                    fecha_ultima_sincronizacion TEXT
                );
                INSERT OR IGNORE INTO local_e8_configuracion (id, dispositivo_id)
                VALUES (1, lower(hex(randomblob(16))));

                CREATE TABLE IF NOT EXISTS local_e8_eventos_remotos (
                    id_evento TEXT PRIMARY KEY,
                    conocido_en TEXT NOT NULL
                );

                CREATE TABLE IF NOT EXISTS local_e8_lotes_sincronizacion (
                    id_lote TEXT NOT NULL,
                    dispositivo_origen TEXT NOT NULL,
                    direccion TEXT NOT NULL CHECK (direccion IN ('recibido', 'enviado')),
                    registrado_en TEXT NOT NULL,
                    PRIMARY KEY (id_lote, direccion)
                );
                ",
            )
            .map_err(|error| {
                ErrorRepositorio(format!("No se pudo inicializar el libro SQLite: {error}"))
            })?;

        Ok(Self { conexion })
    }

    pub fn estado_sincronizacion_personal(
        &self,
    ) -> Result<EstadoSincronizacionPersonal, ErrorRepositorio> {
        let (dispositivo_id, tipo, ubicacion, fecha): (
            String,
            Option<String>,
            Option<String>,
            Option<String>,
        ) = self
            .conexion
            .query_row(
                "SELECT dispositivo_id, tipo_destino, ubicacion_destino, fecha_ultima_sincronizacion
                 FROM local_e8_configuracion WHERE id = 1",
                [],
                |fila| Ok((fila.get(0)?, fila.get(1)?, fila.get(2)?, fila.get(3)?)),
            )
            .map_err(|error| {
                ErrorRepositorio(format!("No se pudo leer la configuración E8: {error}"))
            })?;
        let destino = match (tipo, ubicacion) {
            (Some(tipo), Some(ubicacion)) => Some(DestinoSincronizacion {
                tipo: match tipo.as_str() {
                    "carpeta" => TipoDestinoSincronizacion::Carpeta,
                    "git" => TipoDestinoSincronizacion::Git,
                    _ => {
                        return Err(ErrorRepositorio(
                            "El tipo de destino persistido no es válido.".to_string(),
                        ))
                    }
                },
                ubicacion,
            }),
            (None, None) => None,
            _ => {
                return Err(ErrorRepositorio(
                    "La configuración persistida del destino está incompleta.".to_string(),
                ))
            }
        };
        Ok(EstadoSincronizacionPersonal {
            dispositivo_id,
            destino,
            fecha_ultima_sincronizacion: fecha,
        })
    }

    pub fn configurar_destino_sincronizacion(
        &mut self,
        destino: Option<&DestinoSincronizacion>,
    ) -> Result<(), ErrorRepositorio> {
        let (tipo, ubicacion) = match destino {
            Some(destino) if !destino.ubicacion.trim().is_empty() => (
                Some(match &destino.tipo {
                    TipoDestinoSincronizacion::Carpeta => "carpeta",
                    TipoDestinoSincronizacion::Git => "git",
                }),
                Some(destino.ubicacion.as_str()),
            ),
            Some(_) => {
                return Err(ErrorRepositorio(
                    "La ubicación del destino no puede estar vacía.".to_string(),
                ))
            }
            None => (None, None),
        };
        let tx = self
            .conexion
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(|error| {
                ErrorRepositorio(format!("No se pudo iniciar el cambio de destino: {error}"))
            })?;
        let previo: (Option<String>, Option<String>) = tx
            .query_row(
                "SELECT tipo_destino, ubicacion_destino
                 FROM local_e8_configuracion WHERE id = 1",
                [],
                |fila| Ok((fila.get(0)?, fila.get(1)?)),
            )
            .map_err(|error| {
                ErrorRepositorio(format!("No se pudo leer el destino anterior: {error}"))
            })?;
        if previo != (tipo.map(str::to_string), ubicacion.map(str::to_string)) {
            tx.execute("DELETE FROM local_e8_eventos_remotos", [])
                .map_err(|error| {
                    ErrorRepositorio(format!("No se pudo reiniciar el estado del destino: {error}"))
                })?;
        }
        tx.execute(
                "UPDATE local_e8_configuracion
                 SET tipo_destino = ?1, ubicacion_destino = ?2 WHERE id = 1",
                params![tipo, ubicacion],
            )
            .map_err(|error| {
                ErrorRepositorio(format!("No se pudo guardar la configuración E8: {error}"))
            })?;
        if previo != (tipo.map(str::to_string), ubicacion.map(str::to_string)) {
            tx.execute(
                "UPDATE local_e8_configuracion
                 SET fecha_ultima_sincronizacion = NULL WHERE id = 1",
                [],
            )
            .map_err(|error| {
                ErrorRepositorio(format!("No se pudo reiniciar la fecha de sincronización: {error}"))
            })?;
        }
        tx.commit()
            .map_err(|error| ErrorRepositorio(format!("No se pudo confirmar el destino: {error}")))
    }

    // --- Métodos de Asignación de Cuenta por Grupo (INV-18) ---

    pub fn asignar_cuenta_grupo(
        &mut self,
        grupo_id: &str,
        cuenta_id: &str,
        moneda: &str,
        momento_actual: &str,
        dispositivo: &str,
    ) -> Result<usize, ErrorRepositorio> {
        let tx = self
            .conexion
            .transaction()
            .map_err(|e| ErrorRepositorio(format!("Error iniciando transacción SQLite: {e}")))?;

        // Guardar la asignación de cuenta por grupo
        tx.execute(
            "INSERT INTO local_cuentas_por_grupo (grupo_id, cuenta_id_personal, moneda, actualizado_en)
             VALUES (?1, ?2, ?3, ?4)
             ON CONFLICT(grupo_id) DO UPDATE SET
                cuenta_id_personal = excluded.cuenta_id_personal,
                moneda = excluded.moneda,
                actualizado_en = excluded.actualizado_en",
            params![grupo_id, cuenta_id, moneda, momento_actual],
        )
        .map_err(|e| ErrorRepositorio(format!("Error guardando cuenta de grupo: {e}")))?;

        // Buscar partes pendientes para este grupo y aplicarlas automáticamente (INV-18 / Subtarea 2.3)
        let mut consulta = tx
            .prepare(
                "SELECT id, id_evento_servidor, grupo_id, nombre_grupo, gasto_id, concepto, monto, moneda, fecha
                 FROM local_partes_pendientes WHERE grupo_id = ?1",
            )
            .map_err(|e| ErrorRepositorio(format!("Error consultando partes pendientes: {e}")))?;

        let filas = consulta
            .query_map(params![grupo_id], |fila| {
                Ok((
                    fila.get::<_, String>(0)?,
                    fila.get::<_, String>(1)?,
                    fila.get::<_, String>(2)?,
                    fila.get::<_, String>(3)?,
                    fila.get::<_, String>(4)?,
                    fila.get::<_, String>(5)?,
                    fila.get::<_, String>(6)?,
                    fila.get::<_, String>(7)?,
                    fila.get::<_, String>(8)?,
                ))
            })
            .map_err(|e| ErrorRepositorio(format!("Error leyendo partes pendientes: {e}")))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| ErrorRepositorio(format!("Error extrayendo partes pendientes: {e}")))?;

        drop(consulta);

        let mut aplicadas = 0;
        for (_id_parte, id_ev_servidor, g_id, nom_g, gasto_id, concepto, monto_str, mon, fecha) in filas {
            if mon.trim().eq_ignore_ascii_case(moneda.trim()) {
                let monto_dec = Decimal::from_str(&monto_str)
                    .map_err(|e| ErrorRepositorio(format!("Error convirtiendo monto decimal: {e}")))?;
                let cod_moneda = CodigoMoneda::parse(&mon)
                    .map_err(|e| ErrorRepositorio(format!("Moneda inválida: {e:?}")))?;
                let monto_obj = Monto::new(monto_dec, cod_moneda);

                let id_evento_local = uuid::Uuid::new_v4().to_string();
                let id_movimiento = EventoLibro::id_movimiento_gasto_grupo(&id_ev_servidor);

                let datos = DatosGastoDeGrupoAplicado {
                    id_evento_servidor: id_ev_servidor.clone(),
                    id_movimiento,
                    id_cuenta: cuenta_id.to_string(),
                    grupo_id: g_id,
                    nombre_grupo: nom_g,
                    gasto_id,
                    concepto,
                    monto: monto_obj,
                    fecha,
                };

                let ev_libro = EventoLibro::nuevo_gasto_de_grupo_aplicado(
                    momento_actual,
                    dispositivo,
                    datos,
                )
                .map_err(|e| ErrorRepositorio(e.to_string()))?;

                let json = serde_json::to_string(&ev_libro)
                    .map_err(|e| ErrorRepositorio(format!("Error serializando evento: {e}")))?;

                tx.execute(
                    "INSERT INTO eventos_libro (id, tipo, momento_registro, dispositivo_origen, evento_json)
                     VALUES (?1, ?2, ?3, ?4, ?5)",
                    params![
                        ev_libro.id(),
                        ev_libro.tipo(),
                        ev_libro.momento_registro(),
                        ev_libro.dispositivo_origen(),
                        json
                    ],
                )
                .map_err(|e| ErrorRepositorio(format!("Error insertando en eventos_libro: {e}")))?;

                tx.execute(
                    "INSERT INTO local_eventos_servidor_aplicados (id_evento_servidor, evento_libro_id, aplicado_en)
                     VALUES (?1, ?2, ?3)",
                    params![id_ev_servidor, id_evento_local, momento_actual],
                )
                .map_err(|e| ErrorRepositorio(format!("Error registrando evento servidor: {e}")))?;

                tx.execute(
                    "DELETE FROM local_partes_pendientes WHERE id_evento_servidor = ?1",
                    params![id_ev_servidor],
                )
                .map_err(|e| ErrorRepositorio(format!("Error eliminando parte pendiente: {e}")))?;

                aplicadas += 1;
            }
        }

        tx.commit()
            .map_err(|e| ErrorRepositorio(format!("Error en commit de asignación de cuenta: {e}")))?;

        Ok(aplicadas)
    }

    pub fn obtener_cuenta_grupo(&self, grupo_id: &str) -> Result<Option<CuentaPorGrupo>, ErrorRepositorio> {
        let fila = self
            .conexion
            .query_row(
                "SELECT grupo_id, cuenta_id_personal, moneda, actualizado_en FROM local_cuentas_por_grupo WHERE grupo_id = ?1",
                params![grupo_id],
                |r| {
                    Ok(CuentaPorGrupo {
                        grupo_id: r.get(0)?,
                        cuenta_id_personal: r.get(1)?,
                        moneda: r.get(2)?,
                        actualizado_en: r.get(3)?,
                    })
                },
            )
            .optional()
            .map_err(|e| ErrorRepositorio(format!("Error consultando cuenta por grupo: {e}")))?;

        Ok(fila)
    }

    pub fn listar_cuentas_por_grupo(&self) -> Result<Vec<CuentaPorGrupo>, ErrorRepositorio> {
        let mut prep = self
            .conexion
            .prepare("SELECT grupo_id, cuenta_id_personal, moneda, actualizado_en FROM local_cuentas_por_grupo")
            .map_err(|e| ErrorRepositorio(format!("Error preparando consulta: {e}")))?;

        let iter = prep
            .query_map([], |r| {
                Ok(CuentaPorGrupo {
                    grupo_id: r.get(0)?,
                    cuenta_id_personal: r.get(1)?,
                    moneda: r.get(2)?,
                    actualizado_en: r.get(3)?,
                })
            })
            .map_err(|e| ErrorRepositorio(format!("Error leyendo cuentas por grupo: {e}")))?;

        let mut res = Vec::new();
        for item in iter {
            res.push(item.map_err(|e| ErrorRepositorio(format!("Error mapeando fila: {e}")))?);
        }
        Ok(res)
    }

    // --- Métodos de Partes Pendientes (INV-18) ---

    pub fn listar_partes_pendientes(&self) -> Result<Vec<PartePendienteLocal>, ErrorRepositorio> {
        let mut prep = self
            .conexion
            .prepare(
                "SELECT id, id_evento_servidor, posicion, grupo_id, nombre_grupo, gasto_id, concepto, monto, moneda, fecha, incluye_redondeo, recibido_en
                 FROM local_partes_pendientes ORDER BY posicion ASC",
            )
            .map_err(|e| ErrorRepositorio(format!("Error preparando consulta de partes pendientes: {e}")))?;

        let iter = prep
            .query_map([], |r| {
                let redondeo_int: i32 = r.get(10)?;
                Ok(PartePendienteLocal {
                    id: r.get(0)?,
                    id_evento_servidor: r.get(1)?,
                    posicion: r.get(2)?,
                    grupo_id: r.get(3)?,
                    nombre_grupo: r.get(4)?,
                    gasto_id: r.get(5)?,
                    concepto: r.get(6)?,
                    monto: r.get(7)?,
                    moneda: r.get(8)?,
                    fecha: r.get(9)?,
                    incluye_redondeo: redondeo_int != 0,
                    recibido_en: r.get(11)?,
                })
            })
            .map_err(|e| ErrorRepositorio(format!("Error consultando partes pendientes: {e}")))?;

        let mut res = Vec::new();
        for item in iter {
            res.push(item.map_err(|e| ErrorRepositorio(format!("Error mapeando parte pendiente: {e}")))?);
        }
        Ok(res)
    }

    // --- Métodos de Cursor de Sincronización (INV-14) ---

    pub fn obtener_ultimo_cursor(&self) -> Result<i64, ErrorRepositorio> {
        let cursor: i64 = self
            .conexion
            .query_row(
                "SELECT ultimo_cursor FROM local_sincronizacion_cursor WHERE id = 1",
                [],
                |r| r.get(0),
            )
            .unwrap_or(0);
        Ok(cursor)
    }

    pub fn reiniciar_cursor(&mut self) -> Result<(), ErrorRepositorio> {
        self.conexion
            .execute(
                "UPDATE local_sincronizacion_cursor SET ultimo_cursor = 0 WHERE id = 1",
                [],
            )
            .map_err(|e| ErrorRepositorio(format!("Error reiniciando cursor SQLite: {e}")))?;
        Ok(())
    }

    // --- Transacción SQLite Atómica de Sincronización de Lote (Subtarea 3.3 / INV-13, 14, 15) ---

    pub fn aplicar_lote_cambios(
        &mut self,
        eventos_servidor: &[EventoServidorRecibido],
        nuevo_cursor: i64,
        momento_actual: &str,
        dispositivo: &str,
    ) -> Result<ResultadoSincronizacionLocal, ErrorRepositorio> {
        let tx = self
            .conexion
            .transaction()
            .map_err(|e| ErrorRepositorio(format!("Error iniciando transacción SQLite: {e}")))?;

        let mut eventos_aplicados = 0;
        let mut partes_pendientes = 0;

        for ev in eventos_servidor {
            // INV-13 / INV-15: Si el evento del servidor ya fue aplicado, se ignora
            let ya_aplicado: bool = tx
                .query_row(
                    "SELECT 1 FROM local_eventos_servidor_aplicados WHERE id_evento_servidor = ?1",
                    params![ev.id],
                    |_| Ok(true),
                )
                .optional()
                .map_err(|e| ErrorRepositorio(format!("Error verificando evento aplicado: {e}")))?
                .unwrap_or(false);

            if ya_aplicado {
                continue;
            }

            // Consultar si hay una cuenta asignada a este grupo
            let cuenta_asignada: Option<(String, String)> = tx
                .query_row(
                    "SELECT cuenta_id_personal, moneda FROM local_cuentas_por_grupo WHERE grupo_id = ?1",
                    params![ev.grupo_id],
                    |fila| Ok((fila.get(0)?, fila.get(1)?)),
                )
                .optional()
                .map_err(|e| ErrorRepositorio(format!("Error consultando cuenta de grupo: {e}")))?;

            if let Some((id_cuenta, moneda_cuenta)) = cuenta_asignada {
                if moneda_cuenta.trim().eq_ignore_ascii_case(ev.moneda.trim()) {
                    // Hay cuenta y la moneda coincide exactamente (INV-18)
                    let cod_moneda = CodigoMoneda::parse(&ev.moneda)
                        .map_err(|e| ErrorRepositorio(format!("Moneda inválida: {e:?}")))?;
                    let monto_obj = Monto::new(ev.monto_parte, cod_moneda);
                    let id_movimiento = EventoLibro::id_movimiento_gasto_grupo(&ev.id);

                    let datos = DatosGastoDeGrupoAplicado {
                        id_evento_servidor: ev.id.clone(),
                        id_movimiento,
                        id_cuenta,
                        grupo_id: ev.grupo_id.clone(),
                        nombre_grupo: ev.nombre_grupo.clone(),
                        gasto_id: ev.gasto_id.clone(),
                        concepto: ev.concepto.clone(),
                        monto: monto_obj,
                        fecha: ev.fecha.clone(),
                    };

                    let ev_libro = EventoLibro::nuevo_gasto_de_grupo_aplicado(
                        momento_actual,
                        dispositivo,
                        datos,
                    )
                    .map_err(|e| ErrorRepositorio(e.to_string()))?;

                    let json = serde_json::to_string(&ev_libro).map_err(|e| {
                        ErrorRepositorio(format!("Error serializando evento: {e}"))
                    })?;

                    tx.execute(
                        "INSERT INTO eventos_libro (id, tipo, momento_registro, dispositivo_origen, evento_json)
                         VALUES (?1, ?2, ?3, ?4, ?5)",
                        params![
                            ev_libro.id(),
                            ev_libro.tipo(),
                            ev_libro.momento_registro(),
                            ev_libro.dispositivo_origen(),
                            json
                        ],
                    )
                    .map_err(|e| ErrorRepositorio(format!("Error insertando en eventos_libro: {e}")))?;

                    tx.execute(
                        "INSERT INTO local_eventos_servidor_aplicados (id_evento_servidor, evento_libro_id, aplicado_en)
                         VALUES (?1, ?2, ?3)",
                        params![ev.id, ev_libro.id(), momento_actual],
                    )
                    .map_err(|e| ErrorRepositorio(format!("Error registrando evento servidor: {e}")))?;
                    
                    // Si estaba en partes pendientes, eliminarlo
                    tx.execute(
                        "DELETE FROM local_partes_pendientes WHERE id_evento_servidor = ?1",
                        params![ev.id],
                    )
                    .map_err(|e| ErrorRepositorio(format!("Error limpiando parte pendiente: {e}")))?;

                    eventos_aplicados += 1;
                    continue;
                }
            }

            // Si no hay cuenta asignada o no coincide la moneda, va a pendientes de aplicar (INV-18)
            let id_parte = uuid::Uuid::new_v4().to_string();
            let incluye_redondeo_int = if ev.incluye_redondeo { 1 } else { 0 };
            tx.execute(
                "INSERT OR REPLACE INTO local_partes_pendientes
                    (id, id_evento_servidor, posicion, grupo_id, nombre_grupo, gasto_id, concepto, monto, moneda, fecha, incluye_redondeo, recibido_en)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
                params![
                    id_parte,
                    ev.id,
                    ev.posicion,
                    ev.grupo_id,
                    ev.nombre_grupo,
                    ev.gasto_id,
                    ev.concepto,
                    ev.monto_parte.to_string(),
                    ev.moneda,
                    ev.fecha,
                    incluye_redondeo_int,
                    momento_actual
                ],
            )
            .map_err(|e| ErrorRepositorio(format!("Error guardando parte pendiente: {e}")))?;

            partes_pendientes += 1;
        }

        // Actualizar el cursor dentro de la MISMA transacción atómica (INV-14)
        tx.execute(
            "UPDATE local_sincronizacion_cursor SET ultimo_cursor = ?1, fecha_ultima_sincronizacion = ?2 WHERE id = 1",
            params![nuevo_cursor, momento_actual],
        )
        .map_err(|e| ErrorRepositorio(format!("Error actualizando cursor SQLite: {e}")))?;

        tx.commit()
            .map_err(|e| ErrorRepositorio(format!("Error haciendo commit del lote de sincronización: {e}")))?;

        Ok(ResultadoSincronizacionLocal {
            eventos_aplicados,
            partes_pendientes,
            nuevo_cursor,
        })
    }

    // --- Métodos de la Cola Offline (RNF-17, ESC-09) ---

    pub fn encolar_gasto_offline(&mut self, op: &OperacionGastoOffline) -> Result<(), ErrorRepositorio> {
        let participantes_json = serde_json::to_string(&op.participantes)
            .map_err(|e| ErrorRepositorio(format!("Error serializando participantes: {e}")))?;
        let valores_json = op
            .valores
            .as_ref()
            .map(|v| serde_json::to_string(v))
            .transpose()
            .map_err(|e| ErrorRepositorio(format!("Error serializando valores: {e}")))?;

        self.conexion
            .execute(
                "INSERT INTO local_cola_gastos_offline
                    (id, idempotency_key, grupo_id, concepto, monto, pagador_id, regla_division, participantes_json, valores_json, creado_en, estado, error_mensaje)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
                params![
                    op.id,
                    op.idempotency_key,
                    op.grupo_id,
                    op.concepto,
                    op.monto,
                    op.pagador_id,
                    op.regla_division,
                    participantes_json,
                    valores_json,
                    op.creado_en,
                    op.estado,
                    op.error_mensaje,
                ],
            )
            .map_err(|e| ErrorRepositorio(format!("Error encolando gasto offline: {e}")))?;

        Ok(())
    }

    pub fn listar_cola_offline(&self) -> Result<Vec<OperacionGastoOffline>, ErrorRepositorio> {
        let mut prep = self
            .conexion
            .prepare(
                "SELECT id, idempotency_key, grupo_id, concepto, monto, pagador_id, regla_division, participantes_json, valores_json, creado_en, estado, error_mensaje
                 FROM local_cola_gastos_offline ORDER BY creado_en ASC",
            )
            .map_err(|e| ErrorRepositorio(format!("Error preparando consulta de cola offline: {e}")))?;

        let iter = prep
            .query_map([], |r| {
                let part_json: String = r.get(7)?;
                let val_json: Option<String> = r.get(8)?;

                let participantes: Vec<String> = serde_json::from_str(&part_json).unwrap_or_default();
                let valores: Option<HashMap<String, String>> = val_json
                    .and_then(|j| serde_json::from_str(&j).ok());

                Ok(OperacionGastoOffline {
                    id: r.get(0)?,
                    idempotency_key: r.get(1)?,
                    grupo_id: r.get(2)?,
                    concepto: r.get(3)?,
                    monto: r.get(4)?,
                    pagador_id: r.get(5)?,
                    regla_division: r.get(6)?,
                    participantes,
                    valores,
                    creado_en: r.get(9)?,
                    estado: r.get(10)?,
                    error_mensaje: r.get(11)?,
                })
            })
            .map_err(|e| ErrorRepositorio(format!("Error consultando cola offline: {e}")))?;

        let mut res = Vec::new();
        for item in iter {
            res.push(item.map_err(|e| ErrorRepositorio(format!("Error leyendo elemento de cola: {e}")))?);
        }
        Ok(res)
    }

    pub fn actualizar_estado_cola_offline(
        &mut self,
        id: &str,
        estado: &str,
        error: Option<&str>,
    ) -> Result<(), ErrorRepositorio> {
        self.conexion
            .execute(
                "UPDATE local_cola_gastos_offline SET estado = ?1, error_mensaje = ?2 WHERE id = ?3",
                params![estado, error, id],
            )
            .map_err(|e| ErrorRepositorio(format!("Error actualizando estado en cola offline: {e}")))?;
        Ok(())
    }

    pub fn eliminar_cola_offline(&mut self, id: &str) -> Result<(), ErrorRepositorio> {
        self.conexion
            .execute(
                "DELETE FROM local_cola_gastos_offline WHERE id = ?1",
                params![id],
            )
            .map_err(|e| ErrorRepositorio(format!("Error eliminando de cola offline: {e}")))?;
        Ok(())
    }
}

impl RepositorioLibro for RepositorioLibroSqlite {
    fn guardar(&mut self, evento: EventoLibro) -> Result<(), ErrorRepositorio> {
        let evento_json = serde_json::to_string(&evento).map_err(|error| {
            ErrorRepositorio(format!("No se pudo serializar el evento: {error}"))
        })?;

        self.conexion
            .execute(
                "INSERT INTO eventos_libro
                    (id, tipo, momento_registro, dispositivo_origen, evento_json)
                 VALUES (?1, ?2, ?3, ?4, ?5)
                 ON CONFLICT(id) DO NOTHING",
                params![
                    evento.id(),
                    evento.tipo(),
                    evento.momento_registro(),
                    evento.dispositivo_origen(),
                    evento_json,
                ],
            )
            .map_err(|error| ErrorRepositorio(format!("No se pudo guardar el evento: {error}")))?;

        Ok(())
    }

    fn listar_eventos(&self) -> Result<Vec<EventoLibro>, ErrorRepositorio> {
        let mut consulta = self
            .conexion
            .prepare("SELECT evento_json FROM eventos_libro ORDER BY secuencia")
            .map_err(|error| {
                ErrorRepositorio(format!("No se pudieron consultar los eventos: {error}"))
            })?;

        let filas = consulta
            .query_map([], |fila| fila.get::<_, String>(0))
            .map_err(|error| {
                ErrorRepositorio(format!("No se pudieron consultar los eventos: {error}"))
            })?;

        filas
            .map(|fila| {
                let json = fila.map_err(|error| {
                    ErrorRepositorio(format!("No se pudo leer un evento SQLite: {error}"))
                })?;
                serde_json::from_str(&json).map_err(|error| {
                    ErrorRepositorio(format!("No se pudo deserializar un evento SQLite: {error}"))
                })
            })
            .collect()
    }

    fn dispositivo_id(&self) -> Result<String, ErrorRepositorio> {
        self.conexion
            .query_row(
                "SELECT dispositivo_id FROM local_e8_configuracion WHERE id = 1",
                [],
                |fila| fila.get(0),
            )
            .map_err(|error| {
                ErrorRepositorio(format!("No se pudo leer el identificador local: {error}"))
            })
    }
}

impl RepositorioLibroSincronizable for RepositorioLibroSqlite {
    fn fusionar_eventos_atomico(
        &mut self,
        entrantes: &[EventoLibro],
    ) -> Result<ResultadoFusion, ErrorRepositorio> {
        let locales = self.listar_eventos()?;
        let ids_locales = locales
            .iter()
            .map(|evento| evento.id().to_string())
            .collect::<HashSet<_>>();
        let resultado = EventoLibro::fusionar_con_reporte(&locales, entrantes);

        let tx = self
            .conexion
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(|error| {
                ErrorRepositorio(format!("No se pudo iniciar la fusión SQLite: {error}"))
            })?;
        for evento in &resultado.eventos_resultantes {
            if ids_locales.contains(evento.id()) {
                continue;
            }
            let json = serde_json::to_string(evento).map_err(|error| {
                ErrorRepositorio(format!("No se pudo serializar un evento: {error}"))
            })?;
            tx.execute(
                "INSERT INTO eventos_libro
                    (id, tipo, momento_registro, dispositivo_origen, evento_json)
                 VALUES (?1, ?2, ?3, ?4, ?5)",
                params![
                    evento.id(),
                    evento.tipo(),
                    evento.momento_registro(),
                    evento.dispositivo_origen(),
                    json,
                ],
            )
            .map_err(|error| {
                ErrorRepositorio(format!("No se pudo fusionar el libro en SQLite: {error}"))
            })?;
        }
        tx.commit()
            .map_err(|error| ErrorRepositorio(format!("No se pudo confirmar la fusión: {error}")))?;
        Ok(resultado)
    }

    fn ids_eventos_remotos_conocidos(&self) -> Result<HashSet<String>, ErrorRepositorio> {
        let mut consulta = self
            .conexion
            .prepare("SELECT id_evento FROM local_e8_eventos_remotos")
            .map_err(|error| {
                ErrorRepositorio(format!("No se pudieron consultar eventos remotos: {error}"))
            })?;
        let filas = consulta
            .query_map([], |fila| fila.get::<_, String>(0))
            .map_err(|error| {
                ErrorRepositorio(format!("No se pudieron consultar eventos remotos: {error}"))
            })?;
        filas
            .map(|fila| {
                fila.map_err(|error| {
                    ErrorRepositorio(format!("No se pudo leer un evento remoto: {error}"))
                })
            })
            .collect()
    }

    fn registrar_ids_eventos_remotos(
        &mut self,
        ids: &[String],
    ) -> Result<(), ErrorRepositorio> {
        let tx = self
            .conexion
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(|error| {
                ErrorRepositorio(format!("No se pudo registrar estado remoto: {error}"))
            })?;
        let fecha = chrono::Utc::now().to_rfc3339();
        for id in ids {
            tx.execute(
                "INSERT OR IGNORE INTO local_e8_eventos_remotos (id_evento, conocido_en)
                 VALUES (?1, ?2)",
                params![id, fecha],
            )
            .map_err(|error| {
                ErrorRepositorio(format!("No se pudo registrar un evento remoto: {error}"))
            })?;
        }
        tx.commit().map_err(|error| {
            ErrorRepositorio(format!("No se pudo confirmar el estado remoto: {error}"))
        })
    }

    fn registrar_lote_sincronizacion(
        &mut self,
        id_lote: &str,
        dispositivo_origen: &str,
        direccion: DireccionLoteSincronizacion,
        fecha: &str,
    ) -> Result<(), ErrorRepositorio> {
        let direccion = match direccion {
            DireccionLoteSincronizacion::Recibido => "recibido",
            DireccionLoteSincronizacion::Enviado => "enviado",
        };
        self.conexion
            .execute(
                "INSERT INTO local_e8_lotes_sincronizacion
                    (id_lote, dispositivo_origen, direccion, registrado_en)
                 VALUES (?1, ?2, ?3, ?4)
                 ON CONFLICT(id_lote, direccion) DO UPDATE SET
                    dispositivo_origen = excluded.dispositivo_origen,
                    registrado_en = excluded.registrado_en",
                params![id_lote, dispositivo_origen, direccion, fecha],
            )
            .map_err(|error| {
                ErrorRepositorio(format!("No se pudo guardar el lote sincronizado: {error}"))
            })?;
        Ok(())
    }

    fn registrar_sincronizacion_exitosa(
        &mut self,
        ids_eventos_remotos: &[String],
        fecha: &str,
    ) -> Result<(), ErrorRepositorio> {
        let tx = self
            .conexion
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(|error| {
                ErrorRepositorio(format!("No se pudo registrar la sincronización: {error}"))
            })?;
        for id in ids_eventos_remotos {
            tx.execute(
                "INSERT OR IGNORE INTO local_e8_eventos_remotos (id_evento, conocido_en)
                 VALUES (?1, ?2)",
                params![id, fecha],
            )
            .map_err(|error| {
                ErrorRepositorio(format!("No se pudo registrar un evento remoto: {error}"))
            })?;
        }
        tx.execute(
            "UPDATE local_e8_configuracion
             SET fecha_ultima_sincronizacion = ?1 WHERE id = 1",
            params![fecha],
        )
        .map_err(|error| {
            ErrorRepositorio(format!("No se pudo guardar la fecha de sincronización: {error}"))
        })?;
        tx.commit().map_err(|error| {
            ErrorRepositorio(format!("No se pudo confirmar la sincronización: {error}"))
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dominio::cuenta::TipoCuenta;
    use crate::dominio::evento::{
        DatosCuentaCreada, DatosMovimientoCorregido, DatosMovimientoRegistrado,
    };
    use crate::dominio::moneda::CodigoMoneda;
    use crate::dominio::monto::Monto;
    use crate::dominio::movimiento::TipoMovimiento;
    use rust_decimal::Decimal;

    #[test]
    fn las_cuentas_movimientos_y_ajustes_persistidos_no_se_pueden_actualizar_ni_eliminar() {
        let mut repo = RepositorioLibroSqlite::abrir(":memory:").unwrap();
        let moneda = CodigoMoneda::USD;
        let decimales = 2;
        let eventos = [
            EventoLibro::nuevo_cuenta_creada(
                "evento-cuenta",
                "2026-10-15T00:00:00.000Z",
                "dispositivo-prueba",
                DatosCuentaCreada {
                    id_cuenta: "cuenta-1".to_string(),
                    nombre: "Cuenta de prueba".to_string(),
                    tipo: TipoCuenta::Banco,
                    moneda,
                    saldo_inicial: Monto::cero(moneda),
                },
            ),
            EventoLibro::nuevo_movimiento_registrado(
                "evento-movimiento",
                "2026-10-15T01:00:00.000Z",
                "dispositivo-prueba",
                DatosMovimientoRegistrado {
                    id_movimiento: "movimiento-1".to_string(),
                    id_cuenta: "cuenta-1".to_string(),
                    tipo: TipoMovimiento::Gasto,
                    monto: Monto::parse_movimiento("500,00", moneda, decimales).unwrap(),
                    categoria: None,
                    fecha: "2026-10-15".to_string(),
                    descripcion: "Gasto de prueba".to_string(),
                },
            ),
            EventoLibro::nuevo_movimiento_corregido(
                "evento-ajuste",
                "2026-10-15T02:00:00.000Z",
                "dispositivo-prueba",
                DatosMovimientoCorregido {
                    id_ajuste: "ajuste-1".to_string(),
                    id_cuenta: "cuenta-1".to_string(),
                    id_movimiento_original: "movimiento-1".to_string(),
                    monto_ajuste: Monto::new(Decimal::new(45_000, 2), moneda),
                    monto_original: Monto::parse_movimiento("500,00", moneda, decimales).unwrap(),
                    monto_nuevo: Monto::parse_movimiento("50,00", moneda, decimales).unwrap(),
                    fecha: "2026-10-15".to_string(),
                    descripcion: "Corrección de prueba".to_string(),
                },
            ),
            EventoLibro::nuevo_categoria_creada(
                "evento-categoria",
                "2026-10-15T03:00:00.000Z",
                "dispositivo-prueba",
                crate::dominio::evento::DatosCategoriaCreada {
                    id_categoria: "software".to_string(),
                    nombre: "Software".to_string(),
                    tipo: crate::dominio::categoria::TipoCategoria::Gasto,
                },
            ),
        ];

        for evento in &eventos {
            repo.guardar(evento.clone()).unwrap();
        }

        for evento in &eventos {
            assert!(
                repo.conexion
                    .execute(
                        "UPDATE eventos_libro SET evento_json = '{}' WHERE id = ?1",
                        [evento.id()],
                    )
                    .is_err(),
                "El evento {} no debe poder actualizarse",
                evento.id()
            );
            assert!(
                repo.conexion
                    .execute("DELETE FROM eventos_libro WHERE id = ?1", [evento.id()])
                    .is_err(),
                "El evento {} no debe poder eliminarse",
                evento.id()
            );
        }

        assert_eq!(repo.listar_eventos().unwrap(), eventos);
    }

    #[test]
    fn sincronizacion_atomica_idempotencia_y_partes_pendientes() {
        let mut repo = RepositorioLibroSqlite::abrir(":memory:").unwrap();

        // 1. Crear un evento servidor
        let evento_srv = EventoServidorRecibido {
            id: "ev-srv-001".to_string(),
            posicion: 1,
            usuario_id: "user-beto".to_string(),
            grupo_id: "grupo-viaje".to_string(),
            nombre_grupo: "Viaje a Mérida".to_string(),
            gasto_id: "gasto-hotel".to_string(),
            concepto: "Hotel".to_string(),
            fecha: "2026-10-07T20:00:00Z".to_string(),
            moneda: "USD".to_string(),
            monto_parte: Decimal::new(3333, 2), // 33.33
            incluye_redondeo: false,
        };

        // 2. Aplicar lote sin cuenta asignada al grupo -> Va a partes pendientes
        let res1 = repo
            .aplicar_lote_cambios(&[evento_srv.clone()], 1, "2026-10-07T20:01:00Z", "disp-1")
            .unwrap();

        assert_eq!(res1.eventos_aplicados, 0);
        assert_eq!(res1.partes_pendientes, 1);
        assert_eq!(res1.nuevo_cursor, 1);
        assert_eq!(repo.obtener_ultimo_cursor().unwrap(), 1);

        let pendientes = repo.listar_partes_pendientes().unwrap();
        assert_eq!(pendientes.len(), 1);
        assert_eq!(pendientes[0].concepto, "Hotel");

        // 3. Re-aplicar el mismo lote (o reiniciar cursor y re-aplicar) -> no duplica pendientes
        let res_reintento = repo
            .aplicar_lote_cambios(&[evento_srv.clone()], 1, "2026-10-07T20:02:00Z", "disp-1")
            .unwrap();
        assert_eq!(res_reintento.partes_pendientes, 1);
        let pendientes_reintento = repo.listar_partes_pendientes().unwrap();
        assert_eq!(pendientes_reintento.len(), 1); // No duplicado

        // 4. Usuario elige cuenta personal para el grupo -> se aplica automáticamente la parte pendiente
        let aplicadas = repo
            .asignar_cuenta_grupo("grupo-viaje", "cuenta-efectivo-usd", "USD", "2026-10-07T20:05:00Z", "disp-1")
            .unwrap();
        assert_eq!(aplicadas, 1);

        // Ya no hay partes pendientes
        assert_eq!(repo.listar_partes_pendientes().unwrap().len(), 0);

        // Se creó el evento GastoDeGrupoAplicado en el libro local
        let eventos_libro = repo.listar_eventos().unwrap();
        assert_eq!(eventos_libro.len(), 1);
        if let EventoLibro::GastoDeGrupoAplicado { datos, .. } = &eventos_libro[0] {
            assert_eq!(datos.id_evento_servidor, "ev-srv-001");
            assert_eq!(datos.id_cuenta, "cuenta-efectivo-usd");
            assert_eq!(datos.monto.cantidad, Decimal::new(3333, 2));
        } else {
            panic!("Se esperaba GastoDeGrupoAplicado");
        }

        // 5. Demostración Paso 3: Reiniciar cursor a 0 y re-sincronizar el mismo evento
        repo.reiniciar_cursor().unwrap();
        assert_eq!(repo.obtener_ultimo_cursor().unwrap(), 0);

        let res_demo = repo
            .aplicar_lote_cambios(&[evento_srv], 1, "2026-10-07T20:10:00Z", "disp-1")
            .unwrap();

        // Como ya fue aplicado (INV-13 / INV-15), no hace nada: 0 aplicados, 0 pendientes
        assert_eq!(res_demo.eventos_aplicados, 0);
        assert_eq!(res_demo.partes_pendientes, 0);
        assert_eq!(repo.listar_eventos().unwrap().len(), 1); // No se duplicó el evento
        assert_eq!(repo.obtener_ultimo_cursor().unwrap(), 1); // El cursor avanzó normalmente
    }

    #[test]
    fn identidad_de_dispositivo_se_mantiene_al_reabrir_sqlite() {
        let ruta = std::env::temp_dir().join(format!(
            "cuentas_claras_e8_identidad_{}.db",
            uuid::Uuid::new_v4()
        ));
        let id_inicial = {
            let repo = RepositorioLibroSqlite::abrir(&ruta).unwrap();
            repo.dispositivo_id().unwrap()
        };
        let repo = RepositorioLibroSqlite::abrir(&ruta).unwrap();
        assert_eq!(repo.dispositivo_id().unwrap(), id_inicial);
        assert!(!id_inicial.is_empty());
        drop(repo);
        std::fs::remove_file(ruta).unwrap();
    }

    #[test]
    fn fusion_sqlite_es_idempotente_y_rechaza_colisiones_sin_modificar_eventos() {
        let mut repo = RepositorioLibroSqlite::abrir(":memory:").unwrap();
        let evento = EventoLibro::nuevo_cuenta_creada(
            "evento-fusion",
            "2026-10-15T00:00:00.000Z",
            "dispositivo-prueba",
            DatosCuentaCreada {
                id_cuenta: "cuenta-fusion".to_string(),
                nombre: "Cuenta".to_string(),
                tipo: TipoCuenta::Banco,
                moneda: CodigoMoneda::USD,
                saldo_inicial: Monto::cero(CodigoMoneda::USD),
            },
        );
        assert_eq!(
            repo.fusionar_eventos_atomico(&[evento.clone()])
                .unwrap()
                .eventos_nuevos,
            1
        );
        assert_eq!(
            repo.fusionar_eventos_atomico(&[evento.clone()])
                .unwrap()
                .eventos_existentes,
            1
        );

        let conflicto = EventoLibro::nuevo_cuenta_creada(
            "evento-fusion",
            "2026-10-15T00:00:00.000Z",
            "dispositivo-prueba",
            DatosCuentaCreada {
                id_cuenta: "cuenta-distinta".to_string(),
                nombre: "Otra cuenta".to_string(),
                tipo: TipoCuenta::Banco,
                moneda: CodigoMoneda::USD,
                saldo_inicial: Monto::cero(CodigoMoneda::USD),
            },
        );
        let resultado = repo.fusionar_eventos_atomico(&[conflicto]).unwrap();
        assert_eq!(resultado.eventos_rechazados.len(), 1);
        assert_eq!(repo.listar_eventos().unwrap(), vec![evento]);
    }

    #[test]
    fn persiste_destino_ultimo_sync_y_registro_de_lotes() {
        let mut repo = RepositorioLibroSqlite::abrir(":memory:").unwrap();
        repo.configurar_destino_sincronizacion(Some(&DestinoSincronizacion {
            tipo: TipoDestinoSincronizacion::Carpeta,
            ubicacion: "C:\\copias\\cuentas".to_string(),
        }))
        .unwrap();
        repo.registrar_lote_sincronizacion(
            "lote-1",
            "dispositivo-remoto",
            DireccionLoteSincronizacion::Recibido,
            "2026-11-26T18:00:00Z",
        )
        .unwrap();
        repo.registrar_sincronizacion_exitosa(&["evento-1".to_string()], "2026-11-26T18:01:00Z")
            .unwrap();

        let estado = repo.estado_sincronizacion_personal().unwrap();
        assert_eq!(
            estado.destino,
            Some(DestinoSincronizacion {
                tipo: TipoDestinoSincronizacion::Carpeta,
                ubicacion: "C:\\copias\\cuentas".to_string(),
            })
        );
        assert_eq!(
            estado.fecha_ultima_sincronizacion.as_deref(),
            Some("2026-11-26T18:01:00Z")
        );
        assert_eq!(
            repo.ids_eventos_remotos_conocidos().unwrap(),
            HashSet::from(["evento-1".to_string()])
        );
        let lotes: usize = repo
            .conexion
            .query_row(
                "SELECT COUNT(*) FROM local_e8_lotes_sincronizacion WHERE id_lote = 'lote-1'",
                [],
                |fila| fila.get(0),
            )
            .unwrap();
        assert_eq!(lotes, 1);

            repo.configurar_destino_sincronizacion(Some(&DestinoSincronizacion {
                tipo: TipoDestinoSincronizacion::Git,
                ubicacion: "https://git.example.org/cuentas.git".to_string(),
            }))
            .unwrap();
            let estado_nuevo = repo.estado_sincronizacion_personal().unwrap();
            assert!(estado_nuevo.fecha_ultima_sincronizacion.is_none());
            assert!(repo.ids_eventos_remotos_conocidos().unwrap().is_empty());
    }
}
