use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager, State};
use uuid::Uuid;

use crate::aplicacion::exportar_libro::{
    exportar_libro, EntradaExportarLibro, ErrorExportarLibro,
};
use crate::aplicacion::importar_copia::{importar_copia, ErrorImportarCopia};
use crate::aplicacion::puertos::sincronizacion::{
    AlmacenSecretosSincronizacion, CredencialesGit, ErrorAlmacenSecretos,
    TransporteSincronizacion,
};
use crate::aplicacion::sincronizar_libro::{sincronizar_libro, ErrorSincronizarLibro};
use crate::dominio::errores::validar_frase_paso;
use crate::dominio::evento::{EventoRechazado, ResultadoFusion, InformeSincronizacion};
use crate::infraestructura::almacen_secretos_keyring::AlmacenSecretosKeyring;
use crate::infraestructura::cifrador_libro::CifradorArgon2XChaCha20;
use crate::infraestructura::serializador_libro::SerializadorTextoLibro;
use crate::infraestructura::sistema::{GeneradorIdUuid, RelojSistema};
use crate::infraestructura::sqlite_repo::{
    DestinoSincronizacion, RepositorioLibroSqlite, TipoDestinoSincronizacion,
};
use crate::infraestructura::transporte_carpeta::TransporteCarpeta;
use crate::infraestructura::transporte_git::TransporteGit;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ResultadoImportacionDto {
    pub eventos_nuevos: usize,
    pub eventos_existentes: usize,
    pub eventos_rechazados: Vec<EventoRechazado>,
}

impl From<ResultadoFusion> for ResultadoImportacionDto {
    fn from(resultado: ResultadoFusion) -> Self {
        Self {
            eventos_nuevos: resultado.eventos_nuevos,
            eventos_existentes: resultado.eventos_existentes,
            eventos_rechazados: resultado.eventos_rechazados,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ResultadoExportacionDto {
    pub cantidad_eventos: usize,
    pub cifrada: bool,
}

#[tauri::command]
pub fn exportar_copia_seguridad(
    ruta: String,
    frase_paso: Option<String>,
    confirmacion_frase_paso: Option<String>,
    repositorio: State<'_, Mutex<RepositorioLibroSqlite>>,
) -> Result<ResultadoExportacionDto, String> {
    let repositorio = repositorio
        .lock()
        .map_err(|_| "No se pudo acceder al libro local.".to_string())?;
    let salida = exportar_libro(
        &*repositorio,
        &RelojSistema,
        &SerializadorTextoLibro,
        &CifradorArgon2XChaCha20,
        EntradaExportarLibro {
            frase_paso,
            confirmacion_frase_paso,
        },
    )
    .map_err(error_exportacion)?;
    escribir_archivo_atomico(Path::new(&ruta), &salida.contenido)?;
    Ok(ResultadoExportacionDto {
        cantidad_eventos: salida.cantidad_eventos,
        cifrada: salida.cifrada,
    })
}

#[tauri::command]
pub fn importar_copia_seguridad(
    ruta: String,
    frase_paso: Option<String>,
    repositorio: State<'_, Mutex<RepositorioLibroSqlite>>,
) -> Result<ResultadoImportacionDto, String> {
    let contenido =
        fs::read(&ruta).map_err(|_| "No se pudo leer el archivo de copia de seguridad.".to_string())?;
    let mut repositorio = repositorio
        .lock()
        .map_err(|_| "No se pudo acceder al libro local.".to_string())?;
    let resultado = importar_copia(
        &mut *repositorio,
        &SerializadorTextoLibro,
        &CifradorArgon2XChaCha20,
        &contenido,
        frase_paso.as_deref(),
    )
    .map_err(error_importacion)?;
    Ok(resultado.into())
}

#[tauri::command]
pub fn obtener_configuracion_sincronizacion(
    repositorio: State<'_, Mutex<RepositorioLibroSqlite>>,
) -> Result<crate::infraestructura::sqlite_repo::EstadoSincronizacionPersonal, String> {
    let repositorio = repositorio
        .lock()
        .map_err(|_| "No se pudo acceder al libro local.".to_string())?;
    repositorio
        .estado_sincronizacion_personal()
        .map_err(|_| "No se pudo leer la configuración de sincronización.".to_string())
}

#[tauri::command]
pub fn configurar_destino_sincronizacion(
    tipo: Option<String>,
    ubicacion: Option<String>,
    repositorio: State<'_, Mutex<RepositorioLibroSqlite>>,
) -> Result<(), String> {
    let destino = match (tipo, ubicacion) {
        (None, None) => None,
        (Some(tipo), Some(ubicacion)) => {
            if ubicacion.trim().is_empty() || ubicacion.chars().any(char::is_control) {
                return Err("La ubicación del destino no es válida.".to_string());
            }
            let tipo = match tipo.as_str() {
                "carpeta" => {
                    if !Path::new(&ubicacion).is_dir() {
                        return Err(
                            "La carpeta de sincronización no existe o no es accesible."
                                .to_string(),
                        );
                    }
                    TipoDestinoSincronizacion::Carpeta
                }
                "git" => {
                    if !remoto_git_valido(&ubicacion) {
                        return Err(
                            "Usa una URL Git HTTPS válida sin usuario ni token en la URL."
                                .to_string(),
                        );
                    }
                    TipoDestinoSincronizacion::Git
                }
                _ => return Err("El tipo de destino debe ser 'carpeta' o 'git'.".to_string()),
            };
            Some(DestinoSincronizacion { tipo, ubicacion })
        }
        _ => return Err("Tipo y ubicación deben configurarse o borrarse juntos.".to_string()),
    };

    let mut repositorio = repositorio
        .lock()
        .map_err(|_| "No se pudo acceder al libro local.".to_string())?;
    repositorio
        .configurar_destino_sincronizacion(destino.as_ref())
        .map_err(|_| "No se pudo guardar la configuración de sincronización.".to_string())
}

#[tauri::command]
pub fn guardar_frase_paso_sincronizacion(frase_paso: String) -> Result<(), String> {
    validar_frase_paso(&frase_paso)
        .map_err(|_| "La frase de paso debe tener al menos 12 caracteres.".to_string())?;
    let mut secretos = AlmacenSecretosKeyring::nuevo();
    secretos
        .guardar_frase_paso(&frase_paso)
        .map_err(error_almacen_secretos)
}

#[tauri::command]
pub fn actualizar_credenciales_git(usuario: String, token: String) -> Result<(), String> {
    if usuario.trim().is_empty() || token.trim().is_empty() {
        return Err("El usuario y el token de Git son obligatorios.".to_string());
    }
    let mut secretos = AlmacenSecretosKeyring::nuevo();
    secretos
        .guardar_credenciales_git(&CredencialesGit { usuario, token })
        .map_err(error_almacen_secretos)
}

#[tauri::command]
pub fn sincronizar_libro_personal(
    app: AppHandle,
    repositorio: State<'_, Mutex<RepositorioLibroSqlite>>,
) -> Result<InformeSincronizacion, String> {
    let mut repositorio = repositorio
        .lock()
        .map_err(|_| "No se pudo acceder al libro local.".to_string())?;
    let estado = repositorio
        .estado_sincronizacion_personal()
        .map_err(|_| "No se pudo leer la configuración de sincronización.".to_string())?;
    let destino = estado
        .destino
        .ok_or_else(|| "Configura primero un destino de sincronización.".to_string())?;
    let secretos = AlmacenSecretosKeyring::nuevo();

    let resultado = match destino.tipo {
        TipoDestinoSincronizacion::Carpeta => {
            let mut transporte = TransporteCarpeta::nuevo(PathBuf::from(destino.ubicacion));
            ejecutar_sincronizacion(
                &mut repositorio,
                &mut transporte,
                &secretos,
                &estado.dispositivo_id,
            )
        }
        TipoDestinoSincronizacion::Git => {
            let credenciales = secretos
                .obtener_credenciales_git()
                .map_err(error_almacen_secretos)?;
            let directorio_app = app
                .path()
                .app_data_dir()
                .map_err(|_| {
                    "No se pudo acceder al almacenamiento local de la aplicación.".to_string()
                })?;
            let mut transporte = TransporteGit::nuevo(
                destino.ubicacion,
                directorio_app.join("sincronizacion_git"),
                credenciales,
            );
            ejecutar_sincronizacion(
                &mut repositorio,
                &mut transporte,
                &secretos,
                &estado.dispositivo_id,
            )
        }
    };
    resultado.map_err(error_sincronizacion)
}

fn ejecutar_sincronizacion(
    repositorio: &mut RepositorioLibroSqlite,
    transporte: &mut dyn TransporteSincronizacion,
    secretos: &dyn AlmacenSecretosSincronizacion,
    dispositivo_id: &str,
) -> Result<InformeSincronizacion, ErrorSincronizarLibro> {
    sincronizar_libro(
        repositorio,
        transporte,
        &SerializadorTextoLibro,
        &CifradorArgon2XChaCha20,
        secretos,
        &RelojSistema,
        &mut GeneradorIdUuid,
        dispositivo_id,
    )
}

fn remoto_git_valido(remoto: &str) -> bool {
    if remoto.trim() != remoto || remoto.chars().any(char::is_control) {
        return false;
    }
    if !(remoto.starts_with("https://") || remoto.starts_with("file://")) {
        return false;
    }
    if remoto.starts_with("file://") {
        return !remoto[7..].is_empty();
    }
    let autoridad = remoto
        .split_once("://")
        .map(|(_, resto)| resto.split('/').next().unwrap_or_default())
        .unwrap_or_default();
    !autoridad.is_empty()
        && !autoridad.contains('@')
        && !remoto.contains('?')
        && !remoto.contains('#')
}

fn escribir_archivo_atomico(ruta: &Path, contenido: &[u8]) -> Result<(), String> {
    if ruta.as_os_str().is_empty() {
        return Err("Selecciona una ruta para guardar la copia.".to_string());
    }
    let directorio = ruta
        .parent()
        .filter(|padre| !padre.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    let nombre = ruta
        .file_name()
        .ok_or_else(|| "La ruta de la copia no es válida.".to_string())?;
    let temporal = directorio.join(format!(
        ".{}.{}.tmp",
        nombre.to_string_lossy(),
        Uuid::new_v4()
    ));
    let resultado = (|| {
        let mut archivo = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temporal)
            .map_err(|_| "No se pudo crear el archivo de copia.".to_string())?;
        archivo
            .write_all(contenido)
            .and_then(|_| archivo.sync_all())
            .map_err(|_| "No se pudo escribir el archivo de copia.".to_string())?;
        fs::rename(&temporal, ruta)
            .map_err(|_| "No se pudo finalizar el archivo de copia.".to_string())
    })();
    if resultado.is_err() {
        let _ = fs::remove_file(temporal);
    }
    resultado
}

fn error_exportacion(error: ErrorExportarLibro) -> String {
    let mensaje = error.to_string();
    match error {
        ErrorExportarLibro::FrasePasoInvalida
        | ErrorExportarLibro::FrasesPasoNoCoinciden
        | ErrorExportarLibro::ConfirmacionIncompleta
        | ErrorExportarLibro::Formato => mensaje,
        ErrorExportarLibro::Cifrado => "No se pudo cifrar la copia de seguridad.".to_string(),
        ErrorExportarLibro::Repositorio(_) => "No se pudo leer el libro local.".to_string(),
    }
}

fn error_importacion(error: ErrorImportarCopia) -> String {
    let mensaje = error.to_string();
    match error {
        ErrorImportarCopia::VersionNoSoportada(_)
        | ErrorImportarCopia::ArchivoMalformado
        | ErrorImportarCopia::FrasePasoInvalida
        | ErrorImportarCopia::FrasePasoRequerida
        | ErrorImportarCopia::FraseIncorrectaODatosDanados => mensaje,
        ErrorImportarCopia::ErrorCifrado => {
            "No se pudo descifrar la copia de seguridad.".to_string()
        }
        ErrorImportarCopia::Repositorio(_) => "No se pudo actualizar el libro local.".to_string(),
    }
}

fn error_sincronizacion(error: ErrorSincronizarLibro) -> String {
    match error {
        ErrorSincronizarLibro::Repositorio(_) => {
            "No se pudo actualizar el libro local.".to_string()
        }
        ErrorSincronizarLibro::AlmacenSecretos => {
            "No se pudo acceder al almacén de secretos del sistema.".to_string()
        }
        otro => otro.to_string(),
    }
}

fn error_almacen_secretos(_: ErrorAlmacenSecretos) -> String {
    "No se pudo acceder al almacén de secretos del sistema.".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_acepta_credenciales_incrustadas_en_la_url_git() {
        assert!(remoto_git_valido("https://git.example.org/grupo/libro.git"));
        assert!(!remoto_git_valido(
            "https://usuario:secreto@git.example.org/grupo/libro.git"
        ));
        assert!(!remoto_git_valido(
            "https://git.example.org/grupo/libro.git?token=secreto"
        ));
        assert!(!remoto_git_valido(
            "http://git.example.org/grupo/libro.git"
        ));
    }
}
