use std::sync::Mutex;

use serde::Serialize;
use tauri::State;

use crate::aplicacion::cambiar_moneda_base::{
    cambiar_moneda_base as ejecutar_cambiar_moneda_base, ErrorCambiarMonedaBase,
};
use crate::aplicacion::consultar_patrimonio::{
    consultar_patrimonio as ejecutar_consultar_patrimonio, ErrorConsultarPatrimonio,
};
use crate::aplicacion::corregir_movimiento::{
    corregir_movimiento as ejecutar_corregir_movimiento, EntradaCorregirMovimiento,
    ErrorCorregirMovimiento,
};
use crate::aplicacion::crear_categoria::{
    crear_categoria as ejecutar_crear_categoria, EntradaCrearCategoria, ErrorCrearCategoria,
};
use crate::aplicacion::crear_cuenta::{
    crear_cuenta as ejecutar_crear_cuenta, EntradaCrearCuenta, ErrorCrearCuenta,
};
use crate::aplicacion::crear_moneda::{crear_moneda as ejecutar_crear_moneda, ErrorCrearMoneda};
use crate::aplicacion::dto::{
    CambiarMonedaBaseDto, ConsultarPatrimonioEntrada, CrearMonedaDto, EliminarTasaDto,
    ListarHistorialTasasEntrada, PatrimonioDto, RegistrarTasaDto, ResultadoMonedaBaseDto,
    ResultadoMonedaDto, ResultadoOperacionDto, ResultadoTasaDto, TasaCambioDto,
};
use crate::aplicacion::eliminar_tasa::{
    eliminar_tasa as ejecutar_eliminar_tasa, ErrorEliminarTasa,
};
use crate::aplicacion::listar_categorias::{
    listar_categorias as ejecutar_listar_categorias, ErrorListarCategorias,
};
use crate::aplicacion::listar_cuentas::{
    listar_cuentas as ejecutar_listar_cuentas, ErrorListarCuentas, ResumenCuentas,
};
use crate::aplicacion::listar_historial_tasas::{
    listar_historial_tasas as ejecutar_listar_historial_tasas, ErrorListarHistorialTasas,
};
use crate::aplicacion::listar_monedas::{self as ejecutar_listar_monedas, ErrorListarMonedas};
use crate::aplicacion::obtener_detalle_cuenta::{
    obtener_detalle_cuenta as ejecutar_obtener_detalle_cuenta, DetalleCuenta,
    ErrorObtenerDetalleCuenta,
};
use crate::aplicacion::registrar_movimiento::{
    registrar_movimiento as ejecutar_registrar_movimiento, EntradaRegistrarMovimiento,
    ErrorRegistrarMovimiento,
};
use crate::aplicacion::registrar_tasa::{
    registrar_tasa as ejecutar_registrar_tasa, ErrorRegistrarTasa,
};
use crate::dominio::categoria::Categoria;
use crate::dominio::cuenta::Cuenta;
use crate::dominio::errores::ErrorDominio;
use crate::dominio::moneda::Moneda;
use crate::dominio::movimiento::Movimiento;
use crate::infraestructura::sistema::{GeneradorIdUuid, RelojSistema};
use crate::infraestructura::sqlite_repo::RepositorioLibroSqlite;

#[derive(Debug, Serialize)]
#[serde(untagged)]
pub enum ErrorComando {
    Validacion(ErrorDominio),
    Tecnico(String),
}

#[tauri::command]
pub fn listar_monedas(
    repositorio: State<'_, Mutex<RepositorioLibroSqlite>>,
) -> Result<Vec<Moneda>, String> {
    let repositorio = repositorio
        .lock()
        .map_err(|error| format!("No se pudo acceder al libro local: {error}"))?;

    ejecutar_listar_monedas::listar_monedas_registradas(&*repositorio).map_err(
        |error| match error {
            ErrorListarMonedas::Dominio(error) => error.to_string(),
            ErrorListarMonedas::Repositorio(error) => error.0,
        },
    )
}

#[tauri::command]
pub fn crear_moneda(
    entrada: CrearMonedaDto,
    repositorio: State<'_, Mutex<RepositorioLibroSqlite>>,
) -> Result<ResultadoMonedaDto, ErrorComando> {
    let mut repositorio = repositorio.lock().map_err(|error| {
        ErrorComando::Tecnico(format!("No se pudo acceder al libro local: {error}"))
    })?;
    let mut ids = GeneradorIdUuid;

    ejecutar_crear_moneda(&mut *repositorio, &RelojSistema, &mut ids, entrada).map_err(|error| {
        match error {
            ErrorCrearMoneda::Validacion(error) => ErrorComando::Validacion(error),
            ErrorCrearMoneda::Repositorio(error) => ErrorComando::Tecnico(error.0),
        }
    })
}

#[tauri::command]
pub fn cambiar_moneda_base(
    nuevo_codigo: String,
    repositorio: State<'_, Mutex<RepositorioLibroSqlite>>,
) -> Result<ResultadoMonedaBaseDto, ErrorComando> {
    let mut repositorio = repositorio.lock().map_err(|error| {
        ErrorComando::Tecnico(format!("No se pudo acceder al libro local: {error}"))
    })?;
    let mut ids = GeneradorIdUuid;

    let entrada = CambiarMonedaBaseDto { nuevo_codigo };
    ejecutar_cambiar_moneda_base(&mut *repositorio, &RelojSistema, &mut ids, entrada).map_err(
        |error| match error {
            ErrorCambiarMonedaBase::Validacion(error) => ErrorComando::Validacion(error),
            ErrorCambiarMonedaBase::Repositorio(error) => ErrorComando::Tecnico(error.0),
        },
    )
}

#[tauri::command]
pub fn registrar_tasa(
    entrada: RegistrarTasaDto,
    repositorio: State<'_, Mutex<RepositorioLibroSqlite>>,
) -> Result<ResultadoTasaDto, ErrorComando> {
    let mut repositorio = repositorio.lock().map_err(|error| {
        ErrorComando::Tecnico(format!("No se pudo acceder al libro local: {error}"))
    })?;
    let mut ids = GeneradorIdUuid;

    ejecutar_registrar_tasa(&mut *repositorio, &RelojSistema, &mut ids, entrada).map_err(|error| {
        match error {
            ErrorRegistrarTasa::Validacion(error) => ErrorComando::Validacion(error),
            ErrorRegistrarTasa::Repositorio(error) => ErrorComando::Tecnico(error.0),
        }
    })
}

#[tauri::command]
pub fn eliminar_tasa(
    id_tasa: String,
    repositorio: State<'_, Mutex<RepositorioLibroSqlite>>,
) -> Result<ResultadoOperacionDto, ErrorComando> {
    let mut repositorio = repositorio.lock().map_err(|error| {
        ErrorComando::Tecnico(format!("No se pudo acceder al libro local: {error}"))
    })?;
    let mut ids = GeneradorIdUuid;

    let entrada = EliminarTasaDto { id_tasa };
    ejecutar_eliminar_tasa(&mut *repositorio, &RelojSistema, &mut ids, entrada).map_err(|error| {
        match error {
            ErrorEliminarTasa::Validacion(error) => ErrorComando::Validacion(error),
            ErrorEliminarTasa::Repositorio(error) => ErrorComando::Tecnico(error.0),
        }
    })
}

#[tauri::command]
pub fn listar_historial_tasas(
    entrada: ListarHistorialTasasEntrada,
    repositorio: State<'_, Mutex<RepositorioLibroSqlite>>,
) -> Result<Vec<TasaCambioDto>, ErrorComando> {
    let repositorio = repositorio.lock().map_err(|error| {
        ErrorComando::Tecnico(format!("No se pudo acceder al libro local: {error}"))
    })?;

    ejecutar_listar_historial_tasas(&*repositorio, entrada).map_err(|error| match error {
        ErrorListarHistorialTasas::Validacion(error) => ErrorComando::Validacion(error),
        ErrorListarHistorialTasas::Repositorio(error) => ErrorComando::Tecnico(error.0),
    })
}

#[tauri::command]
pub fn consultar_patrimonio(
    fecha_consulta: Option<String>,
    repositorio: State<'_, Mutex<RepositorioLibroSqlite>>,
) -> Result<PatrimonioDto, ErrorComando> {
    let repositorio = repositorio.lock().map_err(|error| {
        ErrorComando::Tecnico(format!("No se pudo acceder al libro local: {error}"))
    })?;

    let entrada = ConsultarPatrimonioEntrada { fecha_consulta };
    ejecutar_consultar_patrimonio(&*repositorio, &RelojSistema, entrada).map_err(
        |error| match error {
            ErrorConsultarPatrimonio::Validacion(error) => ErrorComando::Validacion(error),
            ErrorConsultarPatrimonio::Repositorio(error) => ErrorComando::Tecnico(error.0),
        },
    )
}

#[tauri::command]
pub fn listar_cuentas(
    repositorio: State<'_, Mutex<RepositorioLibroSqlite>>,
) -> Result<ResumenCuentas, String> {
    let repositorio = repositorio
        .lock()
        .map_err(|error| format!("No se pudo acceder al libro local: {error}"))?;

    ejecutar_listar_cuentas(&*repositorio).map_err(|error| match error {
        ErrorListarCuentas::Dominio(error) => error.to_string(),
        ErrorListarCuentas::Repositorio(error) => error.0,
    })
}

#[tauri::command]
pub fn crear_cuenta(
    entrada: EntradaCrearCuenta,
    repositorio: State<'_, Mutex<RepositorioLibroSqlite>>,
) -> Result<Cuenta, ErrorComando> {
    let mut repositorio = repositorio.lock().map_err(|error| {
        ErrorComando::Tecnico(format!("No se pudo acceder al libro local: {error}"))
    })?;
    let mut ids = GeneradorIdUuid;

    ejecutar_crear_cuenta(&mut *repositorio, &RelojSistema, &mut ids, entrada).map_err(|error| {
        match error {
            ErrorCrearCuenta::Validacion(error) => ErrorComando::Validacion(error),
            ErrorCrearCuenta::Repositorio(error) => ErrorComando::Tecnico(error.0),
        }
    })
}

#[tauri::command]
pub fn registrar_movimiento(
    entrada: EntradaRegistrarMovimiento,
    repositorio: State<'_, Mutex<RepositorioLibroSqlite>>,
) -> Result<Movimiento, ErrorComando> {
    let mut repositorio = repositorio.lock().map_err(|error| {
        ErrorComando::Tecnico(format!("No se pudo acceder al libro local: {error}"))
    })?;
    let mut ids = GeneradorIdUuid;

    ejecutar_registrar_movimiento(&mut *repositorio, &RelojSistema, &mut ids, entrada).map_err(
        |error| match error {
            ErrorRegistrarMovimiento::Validacion(error) => ErrorComando::Validacion(error),
            ErrorRegistrarMovimiento::Repositorio(error) => ErrorComando::Tecnico(error.0),
        },
    )
}

#[tauri::command]
pub fn corregir_movimiento(
    entrada: EntradaCorregirMovimiento,
    repositorio: State<'_, Mutex<RepositorioLibroSqlite>>,
) -> Result<Movimiento, ErrorComando> {
    let mut repositorio = repositorio.lock().map_err(|error| {
        ErrorComando::Tecnico(format!("No se pudo acceder al libro local: {error}"))
    })?;
    let mut ids = GeneradorIdUuid;

    ejecutar_corregir_movimiento(&mut *repositorio, &RelojSistema, &mut ids, entrada).map_err(
        |error| match error {
            ErrorCorregirMovimiento::Validacion(error) => ErrorComando::Validacion(error),
            ErrorCorregirMovimiento::Repositorio(error) => ErrorComando::Tecnico(error.0),
        },
    )
}

#[tauri::command]
pub fn obtener_detalle_cuenta(
    id_cuenta: String,
    repositorio: State<'_, Mutex<RepositorioLibroSqlite>>,
) -> Result<DetalleCuenta, ErrorComando> {
    let repositorio = repositorio.lock().map_err(|error| {
        ErrorComando::Tecnico(format!("No se pudo acceder al libro local: {error}"))
    })?;

    ejecutar_obtener_detalle_cuenta(&*repositorio, &id_cuenta).map_err(|error| match error {
        ErrorObtenerDetalleCuenta::Dominio(error) => ErrorComando::Validacion(error),
        ErrorObtenerDetalleCuenta::Repositorio(error) => ErrorComando::Tecnico(error.0),
    })
}

#[tauri::command]
pub fn crear_categoria(
    entrada: EntradaCrearCategoria,
    repositorio: State<'_, Mutex<RepositorioLibroSqlite>>,
) -> Result<Categoria, ErrorComando> {
    let mut repositorio = repositorio.lock().map_err(|error| {
        ErrorComando::Tecnico(format!("No se pudo acceder al libro local: {error}"))
    })?;
    let mut ids = GeneradorIdUuid;

    ejecutar_crear_categoria(&mut *repositorio, &RelojSistema, &mut ids, entrada).map_err(|error| {
        match error {
            ErrorCrearCategoria::Validacion(error) => ErrorComando::Validacion(error),
            ErrorCrearCategoria::Repositorio(error) => ErrorComando::Tecnico(error.0),
        }
    })
}

#[tauri::command]
pub fn listar_categorias(
    repositorio: State<'_, Mutex<RepositorioLibroSqlite>>,
) -> Result<Vec<Categoria>, String> {
    let repositorio = repositorio
        .lock()
        .map_err(|error| format!("No se pudo acceder al libro local: {error}"))?;

    ejecutar_listar_categorias(&*repositorio).map_err(|error| match error {
        ErrorListarCategorias::Repositorio(error) => error.0,
    })
}

// --- Comandos de Autenticación y Sesión ---

#[tauri::command]
pub async fn registrar_usuario(
    entrada: crate::infraestructura::cliente_auth::EntradaRegistro,
) -> Result<crate::infraestructura::cliente_auth::DatosUsuario, String> {
    crate::infraestructura::cliente_auth::peticion_registro(entrada).await
}

#[tauri::command]
pub async fn iniciar_sesion(
    entrada: crate::infraestructura::cliente_auth::EntradaLogin,
) -> Result<crate::infraestructura::cliente_auth::RespuestaSesion, String> {
    crate::infraestructura::cliente_auth::peticion_login(entrada).await
}

#[tauri::command]
pub async fn obtener_perfil_propio() -> Result<crate::infraestructura::cliente_auth::DatosUsuario, String> {
    let token = crate::infraestructura::cliente_auth::obtener_token_keyring()?
        .ok_or_else(|| "No hay una sesión activa guardada.".to_string())?;
    crate::infraestructura::cliente_auth::peticion_perfil_propio(&token).await
}

#[tauri::command]
pub fn cerrar_sesion() -> Result<(), String> {
    crate::infraestructura::cliente_auth::borrar_token_keyring()
}

#[tauri::command]
pub fn obtener_token_sesion() -> Result<Option<String>, String> {
    crate::infraestructura::cliente_auth::obtener_token_keyring()
}

// ============================================================================
// Comandos de Sincronización, Elección de Cuentas y Cola Offline (Entrega 7)
// ============================================================================

use crate::aplicacion::puertos::Reloj;

#[derive(Debug, Clone, Serialize, serde::Deserialize)]
pub struct ResultadoElegirCuentaDto {
    pub grupo_id: String,
    pub cuenta_id: String,
    pub partes_aplicadas: usize,
}

#[derive(Debug, Clone, Serialize, serde::Deserialize)]
pub struct ResultadoSincronizacionDto {
    pub eventos_aplicados: usize,
    pub partes_pendientes: usize,
    pub nuevo_cursor: i64,
    pub quedan_mas: bool,
    pub operaciones_offline_despachadas: usize,
}

#[derive(Debug, Clone, Serialize, serde::Deserialize)]
pub struct ResultadoDespachoColaDto {
    pub total_procesadas: usize,
    pub exitosas: usize,
    pub con_error: usize,
}

#[derive(Debug, Clone, Serialize, serde::Deserialize)]
pub struct EntradaEncolarGastoDto {
    pub grupo_id: String,
    pub concepto: String,
    pub monto: String,
    pub pagador_id: String,
    pub regla_division: String,
    pub participantes: Vec<String>,
    pub valores: Option<std::collections::HashMap<String, String>>,
}

#[tauri::command]
pub fn elegir_cuenta_grupo(
    grupo_id: String,
    cuenta_id: String,
    moneda: String,
    repositorio: State<'_, Mutex<RepositorioLibroSqlite>>,
) -> Result<ResultadoElegirCuentaDto, String> {
    let momento_actual = RelojSistema.ahora();
    let mut repo = repositorio
        .lock()
        .map_err(|e| format!("No se pudo acceder a la BD local: {e}"))?;

    let partes_aplicadas = repo
        .asignar_cuenta_grupo(&grupo_id, &cuenta_id, &moneda, &momento_actual, "app-desktop")
        .map_err(|e| e.0)?;

    Ok(ResultadoElegirCuentaDto {
        grupo_id,
        cuenta_id,
        partes_aplicadas,
    })
}

#[tauri::command]
pub async fn sincronizar_cambios(
    repositorio: State<'_, Mutex<RepositorioLibroSqlite>>,
) -> Result<ResultadoSincronizacionDto, String> {
    // 1. Despachar primero cualquier elemento pendiente en la cola offline (RNF-17 / ESC-09)
    let ops_despachadas = despachar_cola_offline_interno(&repositorio).await.unwrap_or(0);

    // 2. Obtener el cursor local actual
    let cursor_local = {
        let repo = repositorio
            .lock()
            .map_err(|e| format!("No se pudo acceder a la BD local: {e}"))?;
        repo.obtener_ultimo_cursor().map_err(|e| e.0)?
    };

    // 3. Consultar cambios al servidor
    let cambios_servidor =
        crate::infraestructura::cliente_sync::peticion_obtener_cambios(cursor_local).await?;

    // 4. Aplicar el lote atómicamente en SQLite
    let momento_actual = RelojSistema.ahora();
    let resultado = {
        let mut repo = repositorio
            .lock()
            .map_err(|e| format!("No se pudo acceder a la BD local: {e}"))?;
        repo.aplicar_lote_cambios(
            &cambios_servidor.eventos,
            cambios_servidor.nuevo_cursor,
            &momento_actual,
            "app-desktop",
        )
        .map_err(|e| e.0)?
    };

    Ok(ResultadoSincronizacionDto {
        eventos_aplicados: resultado.eventos_aplicados,
        partes_pendientes: resultado.partes_pendientes,
        nuevo_cursor: resultado.nuevo_cursor,
        quedan_mas: cambios_servidor.quedan_mas,
        operaciones_offline_despachadas: ops_despachadas,
    })
}

#[tauri::command]
pub fn reiniciar_cursor(
    repositorio: State<'_, Mutex<RepositorioLibroSqlite>>,
) -> Result<ResultadoOperacionDto, String> {
    let mut repo = repositorio
        .lock()
        .map_err(|e| format!("No se pudo acceder a la BD local: {e}"))?;
    repo.reiniciar_cursor().map_err(|e| e.0)?;
    Ok(ResultadoOperacionDto { exito: true })
}

async fn despachar_cola_offline_interno(
    repositorio: &State<'_, Mutex<RepositorioLibroSqlite>>,
) -> Result<usize, String> {
    let elementos = {
        let repo = repositorio
            .lock()
            .map_err(|e| format!("No se pudo acceder a la BD local: {e}"))?;
        repo.listar_cola_offline().map_err(|e| e.0)?
    };

    let mut exitosas = 0;
    for op in elementos {
        if op.estado == "ENVIADO" {
            continue;
        }

        match crate::infraestructura::cliente_sync::peticion_enviar_gasto_offline(&op).await {
            Ok(_) => {
                let mut repo = repositorio
                    .lock()
                    .map_err(|e| format!("No se pudo acceder a la BD local: {e}"))?;
                let _ = repo.eliminar_cola_offline(&op.id);
                exitosas += 1;
            }
            Err(e) => {
                let mut repo = repositorio
                    .lock()
                    .map_err(|e| format!("No se pudo acceder a la BD local: {e}"))?;
                let _ = repo.actualizar_estado_cola_offline(&op.id, "ERROR", Some(&e));
            }
        }
    }

    Ok(exitosas)
}

#[tauri::command]
pub async fn despachar_cola_offline(
    repositorio: State<'_, Mutex<RepositorioLibroSqlite>>,
) -> Result<ResultadoDespachoColaDto, String> {
    let elementos = {
        let repo = repositorio
            .lock()
            .map_err(|e| format!("No se pudo acceder a la BD local: {e}"))?;
        repo.listar_cola_offline().map_err(|e| e.0)?
    };

    let total = elementos.len();
    let mut exitosas = 0;
    let mut con_error = 0;

    for op in elementos {
        match crate::infraestructura::cliente_sync::peticion_enviar_gasto_offline(&op).await {
            Ok(_) => {
                let mut repo = repositorio
                    .lock()
                    .map_err(|e| format!("No se pudo acceder a la BD local: {e}"))?;
                let _ = repo.eliminar_cola_offline(&op.id);
                exitosas += 1;
            }
            Err(e) => {
                let mut repo = repositorio
                    .lock()
                    .map_err(|e| format!("No se pudo acceder a la BD local: {e}"))?;
                let _ = repo.actualizar_estado_cola_offline(&op.id, "ERROR", Some(&e));
                con_error += 1;
            }
        }
    }

    Ok(ResultadoDespachoColaDto {
        total_procesadas: total,
        exitosas,
        con_error,
    })
}

#[tauri::command]
pub fn encolar_gasto_offline(
    entrada: EntradaEncolarGastoDto,
    repositorio: State<'_, Mutex<RepositorioLibroSqlite>>,
) -> Result<crate::infraestructura::sqlite_repo::OperacionGastoOffline, String> {
    let momento_actual = RelojSistema.ahora();
    let op = crate::infraestructura::sqlite_repo::OperacionGastoOffline {
        id: uuid::Uuid::new_v4().to_string(),
        idempotency_key: uuid::Uuid::new_v4().to_string(),
        grupo_id: entrada.grupo_id,
        concepto: entrada.concepto,
        monto: entrada.monto,
        pagador_id: entrada.pagador_id,
        regla_division: entrada.regla_division,
        participantes: entrada.participantes,
        valores: entrada.valores,
        creado_en: momento_actual,
        estado: "PENDIENTE".to_string(),
        error_mensaje: None,
    };

    let mut repo = repositorio
        .lock()
        .map_err(|e| format!("No se pudo acceder a la BD local: {e}"))?;
    repo.encolar_gasto_offline(&op).map_err(|e| e.0)?;
    Ok(op)
}

#[tauri::command]
pub fn listar_cola_offline(
    repositorio: State<'_, Mutex<RepositorioLibroSqlite>>,
) -> Result<Vec<crate::infraestructura::sqlite_repo::OperacionGastoOffline>, String> {
    let repo = repositorio
        .lock()
        .map_err(|e| format!("No se pudo acceder a la BD local: {e}"))?;
    repo.listar_cola_offline().map_err(|e| e.0)
}

#[tauri::command]
pub fn listar_partes_pendientes(
    repositorio: State<'_, Mutex<RepositorioLibroSqlite>>,
) -> Result<Vec<crate::infraestructura::sqlite_repo::PartePendienteLocal>, String> {
    let repo = repositorio
        .lock()
        .map_err(|e| format!("No se pudo acceder a la BD local: {e}"))?;
    repo.listar_partes_pendientes().map_err(|e| e.0)
}

#[tauri::command]
pub fn obtener_cuenta_grupo(
    grupo_id: String,
    repositorio: State<'_, Mutex<RepositorioLibroSqlite>>,
) -> Result<Option<crate::infraestructura::sqlite_repo::CuentaPorGrupo>, String> {
    let repo = repositorio
        .lock()
        .map_err(|e| format!("No se pudo acceder a la BD local: {e}"))?;
    repo.obtener_cuenta_grupo(&grupo_id).map_err(|e| e.0)
}
