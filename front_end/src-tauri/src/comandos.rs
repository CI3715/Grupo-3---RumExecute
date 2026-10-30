use std::sync::Mutex;

use serde::Serialize;
use tauri::State;

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
use crate::aplicacion::listar_categorias::{
    listar_categorias as ejecutar_listar_categorias, ErrorListarCategorias,
};
use crate::aplicacion::listar_cuentas::{
    listar_cuentas as ejecutar_listar_cuentas, ErrorListarCuentas, ResumenCuentas,
};
use crate::aplicacion::cambiar_moneda_base::{
    cambiar_moneda_base as ejecutar_cambiar_moneda_base, ErrorCambiarMonedaBase,
};
use crate::aplicacion::crear_moneda::{
    crear_moneda as ejecutar_crear_moneda, ErrorCrearMoneda,
};
use crate::aplicacion::consultar_patrimonio::{
    consultar_patrimonio as ejecutar_consultar_patrimonio, ErrorConsultarPatrimonio,
};
use crate::aplicacion::dto::{
    CambiarMonedaBaseDto, ConsultarPatrimonioEntrada, CrearMonedaDto, EliminarTasaDto,
    ListarHistorialTasasEntrada, PatrimonioDto, RegistrarTasaDto, ResultadoMonedaBaseDto,
    ResultadoMonedaDto, ResultadoOperacionDto, ResultadoTasaDto, TasaCambioDto,
};
use crate::aplicacion::eliminar_tasa::{
    eliminar_tasa as ejecutar_eliminar_tasa, ErrorEliminarTasa,
};
use crate::aplicacion::listar_historial_tasas::{
    listar_historial_tasas as ejecutar_listar_historial_tasas, ErrorListarHistorialTasas,
};
use crate::aplicacion::listar_monedas::{
    self as ejecutar_listar_monedas, ErrorListarMonedas,
};
use crate::aplicacion::obtener_detalle_cuenta::{
    obtener_detalle_cuenta as ejecutar_obtener_detalle_cuenta, DetalleCuenta,
    ErrorObtenerDetalleCuenta,
};
use crate::aplicacion::registrar_tasa::{
    registrar_tasa as ejecutar_registrar_tasa, ErrorRegistrarTasa,
};
use crate::aplicacion::registrar_movimiento::{
    registrar_movimiento as ejecutar_registrar_movimiento, EntradaRegistrarMovimiento,
    ErrorRegistrarMovimiento,
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

    ejecutar_listar_monedas::listar_monedas_registradas(&*repositorio).map_err(|error| match error {
        ErrorListarMonedas::Dominio(error) => error.to_string(),
        ErrorListarMonedas::Repositorio(error) => error.0,
    })
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

    ejecutar_registrar_tasa(&mut *repositorio, &RelojSistema, &mut ids, entrada).map_err(
        |error| match error {
            ErrorRegistrarTasa::Validacion(error) => ErrorComando::Validacion(error),
            ErrorRegistrarTasa::Repositorio(error) => ErrorComando::Tecnico(error.0),
        },
    )
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
    ejecutar_eliminar_tasa(&mut *repositorio, &RelojSistema, &mut ids, entrada).map_err(
        |error| match error {
            ErrorEliminarTasa::Validacion(error) => ErrorComando::Validacion(error),
            ErrorEliminarTasa::Repositorio(error) => ErrorComando::Tecnico(error.0),
        },
    )
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
    ejecutar_consultar_patrimonio(&*repositorio, &RelojSistema, entrada).map_err(|error| {
        match error {
            ErrorConsultarPatrimonio::Validacion(error) => ErrorComando::Validacion(error),
            ErrorConsultarPatrimonio::Repositorio(error) => ErrorComando::Tecnico(error.0),
        }
    })
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
