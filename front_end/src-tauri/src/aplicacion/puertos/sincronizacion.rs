use std::collections::HashSet;

use serde::{Deserialize, Serialize};

use crate::aplicacion::puertos::{ErrorRepositorio, RepositorioLibro};
use crate::dominio::evento::{EventoLibro, ResultadoFusion};

pub const VERSION_FORMATO_LIBRO: u32 = 1;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DocumentoLibro {
    pub version_formato: u32,
    pub fecha_exportacion: String,
    pub cantidad_eventos: usize,
    pub eventos: Vec<EventoLibro>,
}

impl DocumentoLibro {
    pub fn nuevo(fecha_exportacion: impl Into<String>, eventos: Vec<EventoLibro>) -> Self {
        Self {
            version_formato: VERSION_FORMATO_LIBRO,
            fecha_exportacion: fecha_exportacion.into(),
            cantidad_eventos: eventos.len(),
            eventos,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ErrorFormatoLibro {
    VersionNoSoportada(String),
    ArchivoMalformado,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ErrorCifradoLibro {
    FraseIncorrectaODatosDanados,
    OperacionFallida,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ErrorTransporteSincronizacion {
    DestinoNoDisponible,
    CredencialesGitInvalidas,
    ConflictoRemoto,
    OperacionFallida,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ErrorAlmacenSecretos {
    SecretoNoConfigurado,
    OperacionFallida,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoteSincronizacion {
    pub id: String,
    pub dispositivo_origen: String,
    pub contenido: Vec<u8>,
}

pub struct CredencialesGit {
    pub usuario: String,
    pub token: String,
}

pub trait SerializadorLibro {
    fn serializar(&self, documento: &DocumentoLibro) -> Result<String, ErrorFormatoLibro>;
    fn deserializar(&self, contenido: &str) -> Result<DocumentoLibro, ErrorFormatoLibro>;
}

pub trait CifradorLibro {
    fn esta_cifrado(&self, contenido: &[u8]) -> bool;
    fn cifrar(&self, contenido: &[u8], frase_paso: &str) -> Result<Vec<u8>, ErrorCifradoLibro>;
    fn descifrar(
        &self,
        contenido: &[u8],
        frase_paso: &str,
    ) -> Result<Vec<u8>, ErrorCifradoLibro>;
}

pub trait AlmacenSecretosSincronizacion {
    fn obtener_frase_paso(&self) -> Result<Option<String>, ErrorAlmacenSecretos>;
    fn guardar_frase_paso(&mut self, frase_paso: &str) -> Result<(), ErrorAlmacenSecretos>;
    fn obtener_credenciales_git(&self) -> Result<Option<CredencialesGit>, ErrorAlmacenSecretos>;
    fn guardar_credenciales_git(
        &mut self,
        credenciales: &CredencialesGit,
    ) -> Result<(), ErrorAlmacenSecretos>;
    fn eliminar_credenciales_git(&mut self) -> Result<(), ErrorAlmacenSecretos>;
}

pub trait TransporteSincronizacion {
    /// Devuelve lotes no confirmados por este dispositivo; consultar no debe marcarlos leídos.
    fn traer_lotes_no_leidos(
        &mut self,
        dispositivo_id: &str,
    ) -> Result<Vec<LoteSincronizacion>, ErrorTransporteSincronizacion>;

    /// Prepara un lote inmutable dentro del espacio propio del dispositivo.
    fn enviar_lote(
        &mut self,
        dispositivo_id: &str,
        lote: LoteSincronizacion,
    ) -> Result<(), ErrorTransporteSincronizacion>;

    /// Publica los cambios preparados o informa si el destino cambió y requiere reintento.
    fn publicar_cambios(
        &mut self,
        dispositivo_id: &str,
    ) -> Result<(), ErrorTransporteSincronizacion>;

    /// Confirma lectura solo después de aplicar los eventos al libro local.
    fn confirmar_lotes_leidos(
        &mut self,
        dispositivo_id: &str,
        lotes: &[String],
    ) -> Result<(), ErrorTransporteSincronizacion>;

    /// Descarta únicamente la publicación local pendiente tras un conflicto remoto.
    fn descartar_cambios_no_publicados(
        &mut self,
        dispositivo_id: &str,
    ) -> Result<(), ErrorTransporteSincronizacion>;
}

pub trait RepositorioLibroSincronizable: RepositorioLibro {
    /// Aplica la fusión y guarda el resultado como una única operación atómica.
    fn fusionar_eventos_atomico(
        &mut self,
        entrantes: &[EventoLibro],
    ) -> Result<ResultadoFusion, ErrorRepositorio>;

    /// IDs que ya se recibieron del destino o se publicaron correctamente.
    fn ids_eventos_remotos_conocidos(&self) -> Result<HashSet<String>, ErrorRepositorio>;

    fn registrar_ids_eventos_remotos(
        &mut self,
        ids: &[String],
    ) -> Result<(), ErrorRepositorio>;

    /// Persiste la última sincronización exitosa junto con los IDs que ya están en el destino.
    fn registrar_sincronizacion_exitosa(
        &mut self,
        ids_eventos_remotos: &[String],
        fecha: &str,
    ) -> Result<(), ErrorRepositorio>;
}
