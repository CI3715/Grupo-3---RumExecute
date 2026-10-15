use std::sync::Mutex;

use serde::Serialize;
use tauri::State;

use crate::aplicacion::crear_cuenta::{
    crear_cuenta as ejecutar_crear_cuenta, EntradaCrearCuenta, ErrorCrearCuenta,
};
use crate::aplicacion::listar_cuentas::{
    listar_cuentas as ejecutar_listar_cuentas, ErrorListarCuentas, ResumenCuentas,
};
use crate::aplicacion::listar_monedas as ejecutar_listar_monedas;
use crate::dominio::cuenta::Cuenta;
use crate::dominio::errores::ErrorDominio;
use crate::dominio::moneda::Moneda;
use crate::infraestructura::sistema::{GeneradorIdUuid, RelojSistema};
use crate::infraestructura::sqlite_repo::RepositorioLibroSqlite;

#[derive(Debug, Serialize)]
#[serde(untagged)]
pub enum ErrorComando {
    Validacion(ErrorDominio),
    Tecnico(String),
}

#[tauri::command]
pub fn listar_monedas() -> Vec<Moneda> {
    ejecutar_listar_monedas::listar_monedas()
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
