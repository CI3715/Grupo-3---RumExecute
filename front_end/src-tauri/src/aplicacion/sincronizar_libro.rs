use std::collections::HashSet;

use thiserror::Error;

use crate::aplicacion::puertos::sincronizacion::{
    AlmacenSecretosSincronizacion, CifradorLibro, DocumentoLibro, ErrorAlmacenSecretos,
    ErrorCifradoLibro, ErrorFormatoLibro, ErrorTransporteSincronizacion,
    LoteSincronizacion, RepositorioLibroSincronizable, SerializadorLibro,
    TransporteSincronizacion,
};
use crate::aplicacion::puertos::{ErrorRepositorio, GeneradorId, Reloj};
use crate::aplicacion::sincronizacion::validar_documento;
use crate::dominio::errores::validar_frase_paso;
use crate::dominio::dispositivo::DispositivoId;
use crate::dominio::evento::{EventoRechazado, InformeSincronizacion};

const INTENTOS_MAXIMOS_PUBLICACION: usize = 3;

#[derive(Debug, Error)]
pub enum ErrorSincronizarLibro {
    #[error("El identificador del dispositivo no puede estar vacío.")]
    DispositivoInvalido,
    #[error("No hay una frase de paso configurada para la sincronización.")]
    FrasePasoNoConfigurada,
    #[error("La frase de paso configurada no cumple los requisitos mínimos.")]
    FrasePasoInvalida,
    #[error("Frase de paso incorrecta o archivo dañado.")]
    FraseIncorrectaODatosDanados,
    #[error("No se pudo cifrar o descifrar un lote de sincronización.")]
    Cifrado,
    #[error("La versión de formato {0} no está soportada.")]
    VersionNoSoportada(String),
    #[error("Un lote recibido está mal formado.")]
    LoteMalFormado,
    #[error("No se pudo acceder al almacén de secretos.")]
    AlmacenSecretos,
    #[error("No se pudo sincronizar el libro local: {0}")]
    Repositorio(String),
    #[error("El destino de sincronización no está disponible o no hay conexión.")]
    DestinoNoDisponible,
    #[error("Las credenciales de Git no son válidas. Actualízalas en la configuración.")]
    CredencialesGitInvalidas,
    #[error("No se pudo completar la operación de sincronización.")]
    ErrorTransporte,
    #[error("El destino cambió repetidamente; vuelve a intentar la sincronización.")]
    ConflictosRepetidos,
}

pub fn sincronizar_libro(
    repo: &mut dyn RepositorioLibroSincronizable,
    transporte: &mut dyn TransporteSincronizacion,
    serializador: &dyn SerializadorLibro,
    cifrador: &dyn CifradorLibro,
    secretos: &dyn AlmacenSecretosSincronizacion,
    reloj: &dyn Reloj,
    ids: &mut dyn GeneradorId,
    dispositivo_id: &str,
) -> Result<InformeSincronizacion, ErrorSincronizarLibro> {
    if dispositivo_id.trim().is_empty() {
        return Err(ErrorSincronizarLibro::DispositivoInvalido);
    }

    let frase_paso = secretos
        .obtener_frase_paso()
        .map_err(mapear_error_secretos)?
        .ok_or(ErrorSincronizarLibro::FrasePasoNoConfigurada)?;
    validar_frase_paso(&frase_paso).map_err(|_| ErrorSincronizarLibro::FrasePasoInvalida)?;

    let mut eventos_recibidos = 0usize;
    let mut eventos_enviados = HashSet::new();
    let mut rechazados = Vec::<EventoRechazado>::new();

    for intento in 0..INTENTOS_MAXIMOS_PUBLICACION {
        let lotes = transporte
            .traer_lotes_no_leidos(dispositivo_id)
            .map_err(mapear_error_transporte)?;

        let mut documentos = Vec::with_capacity(lotes.len());
        let mut ids_lotes = Vec::with_capacity(lotes.len());
        for lote in &lotes {
            if lote.id.trim().is_empty() || DispositivoId::nuevo(&lote.dispositivo_origen).is_err()
            {
                return Err(ErrorSincronizarLibro::LoteMalFormado);
            }
            let texto = descifrar_lote(cifrador, &lote.contenido, &frase_paso)?;
            let documento = serializador
                .deserializar(&texto)
                .map_err(mapear_error_formato)?;
            documentos.push(documento);
            ids_lotes.push(lote.id.clone());
        }

        let eventos_locales_antes = repo
            .listar_eventos()
            .map_err(|error| ErrorSincronizarLibro::Repositorio(error.0))?;
        let mut eventos_contexto = eventos_locales_antes;
        eventos_contexto.extend(
            documentos
                .iter()
                .flat_map(|documento| documento.eventos.iter().cloned()),
        );
        for documento in &documentos {
            validar_documento(documento, &eventos_contexto)
                .map_err(mapear_error_formato)?;
        }
        let eventos_entrantes = documentos
            .into_iter()
            .flat_map(|documento| documento.eventos)
            .collect::<Vec<_>>();

        let resultado_fusion = repo
            .fusionar_eventos_atomico(&eventos_entrantes)
            .map_err(mapear_error_repositorio)?;
        eventos_recibidos += resultado_fusion.eventos_nuevos;
        rechazados.extend(resultado_fusion.eventos_rechazados);

        let ids_recibidos = eventos_entrantes
            .iter()
            .map(|evento| evento.id().to_string())
            .collect::<Vec<_>>();
        repo.registrar_ids_eventos_remotos(&ids_recibidos)
            .map_err(mapear_error_repositorio)?;
        transporte
            .confirmar_lotes_leidos(dispositivo_id, &ids_lotes)
            .map_err(mapear_error_transporte)?;

        let mut ids_remotos = repo
            .ids_eventos_remotos_conocidos()
            .map_err(mapear_error_repositorio)?;
        ids_remotos.extend(ids_recibidos);

        let eventos_locales = repo
            .listar_eventos()
            .map_err(|error| ErrorSincronizarLibro::Repositorio(error.0))?;
        let eventos_salientes = eventos_locales
            .iter()
            .cloned()
            .filter(|evento| !ids_remotos.contains(evento.id()))
            .collect::<Vec<_>>();
        let ids_salientes = eventos_salientes
            .iter()
            .map(|evento| evento.id().to_string())
            .collect::<Vec<_>>();

        if !eventos_salientes.is_empty() {
            let documento = DocumentoLibro::nuevo(reloj.ahora(), eventos_salientes.clone());
            validar_documento(&documento, &eventos_locales)
                .map_err(mapear_error_formato)?;
            let texto = serializador
                .serializar(&documento)
                .map_err(|_| ErrorSincronizarLibro::LoteMalFormado)?;
            let contenido = cifrador
                .cifrar(texto.as_bytes(), &frase_paso)
                .map_err(|_| ErrorSincronizarLibro::Cifrado)?;
            transporte
                .enviar_lote(
                    dispositivo_id,
                    LoteSincronizacion {
                        id: ids.nuevo_id(),
                        dispositivo_origen: dispositivo_id.to_string(),
                        contenido,
                    },
                )
                .map_err(mapear_error_transporte)?;
        }

        match transporte.publicar_cambios(dispositivo_id) {
            Ok(()) => {
                ids_remotos.extend(ids_salientes.iter().cloned());
                let fecha_ultima_sincronizacion = reloj.ahora();
                let todos_los_ids = ids_remotos.into_iter().collect::<Vec<_>>();
                repo.registrar_sincronizacion_exitosa(
                    &todos_los_ids,
                    &fecha_ultima_sincronizacion,
                )
                .map_err(mapear_error_repositorio)?;

                eventos_enviados.extend(ids_salientes);
                return Ok(InformeSincronizacion {
                    enviados: eventos_enviados.len(),
                    recibidos: eventos_recibidos,
                    rechazados,
                    fecha_ultima_sincronizacion,
                });
            }
            Err(ErrorTransporteSincronizacion::ConflictoRemoto) => {
                transporte
                    .descartar_cambios_no_publicados(dispositivo_id)
                    .map_err(mapear_error_transporte)?;
                if intento + 1 == INTENTOS_MAXIMOS_PUBLICACION {
                    return Err(ErrorSincronizarLibro::ConflictosRepetidos);
                }
            }
            Err(error) => return Err(mapear_error_transporte(error)),
        }
    }

    Err(ErrorSincronizarLibro::ConflictosRepetidos)
}

fn descifrar_lote(
    cifrador: &dyn CifradorLibro,
    contenido: &[u8],
    frase_paso: &str,
) -> Result<String, ErrorSincronizarLibro> {
    let descifrado = cifrador
        .descifrar(contenido, frase_paso)
        .map_err(mapear_error_cifrado)?;
    String::from_utf8(descifrado).map_err(|_| ErrorSincronizarLibro::LoteMalFormado)
}

fn mapear_error_formato(error: ErrorFormatoLibro) -> ErrorSincronizarLibro {
    match error {
        ErrorFormatoLibro::VersionNoSoportada(version) => {
            ErrorSincronizarLibro::VersionNoSoportada(version)
        }
        ErrorFormatoLibro::ArchivoMalformado => ErrorSincronizarLibro::LoteMalFormado,
    }
}

fn mapear_error_cifrado(error: ErrorCifradoLibro) -> ErrorSincronizarLibro {
    match error {
        ErrorCifradoLibro::FraseIncorrectaODatosDanados => {
            ErrorSincronizarLibro::FraseIncorrectaODatosDanados
        }
        ErrorCifradoLibro::OperacionFallida => ErrorSincronizarLibro::Cifrado,
    }
}

fn mapear_error_secretos(_error: ErrorAlmacenSecretos) -> ErrorSincronizarLibro {
    ErrorSincronizarLibro::AlmacenSecretos
}

fn mapear_error_repositorio(error: ErrorRepositorio) -> ErrorSincronizarLibro {
    ErrorSincronizarLibro::Repositorio(error.0)
}

fn mapear_error_transporte(
    error: ErrorTransporteSincronizacion,
) -> ErrorSincronizarLibro {
    match error {
        ErrorTransporteSincronizacion::DestinoNoDisponible => {
            ErrorSincronizarLibro::DestinoNoDisponible
        }
        ErrorTransporteSincronizacion::CredencialesGitInvalidas => {
            ErrorSincronizarLibro::CredencialesGitInvalidas
        }
        ErrorTransporteSincronizacion::ConflictoRemoto => {
            ErrorSincronizarLibro::ConflictosRepetidos
        }
        ErrorTransporteSincronizacion::OperacionFallida => {
            ErrorSincronizarLibro::ErrorTransporte
        }
    }
}
