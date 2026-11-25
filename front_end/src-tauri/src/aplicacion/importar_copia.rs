use thiserror::Error;

use crate::aplicacion::puertos::sincronizacion::{
    CifradorLibro, ErrorCifradoLibro, ErrorFormatoLibro, RepositorioLibroSincronizable,
    SerializadorLibro,
};
use crate::aplicacion::puertos::ErrorRepositorio;
use crate::aplicacion::sincronizacion::validar_documento;
use crate::dominio::errores::validar_frase_paso;
use crate::dominio::evento::ResultadoFusion;

#[derive(Debug, Error)]
pub enum ErrorImportarCopia {
    #[error("La versión de formato {0} no está soportada.")]
    VersionNoSoportada(String),
    #[error("El archivo de copia de seguridad está mal formado.")]
    ArchivoMalformado,
    #[error("La frase de paso debe tener al menos 12 caracteres y no estar vacía.")]
    FrasePasoInvalida,
    #[error("Se requiere la frase de paso para abrir esta copia cifrada.")]
    FrasePasoRequerida,
    #[error("Frase de paso incorrecta o archivo dañado.")]
    FraseIncorrectaODatosDanados,
    #[error("No se pudo descifrar la copia de seguridad.")]
    ErrorCifrado,
    #[error("No se pudo actualizar el libro local: {0}")]
    Repositorio(String),
}

pub fn importar_copia(
    repo: &mut dyn RepositorioLibroSincronizable,
    serializador: &dyn SerializadorLibro,
    cifrador: &dyn CifradorLibro,
    contenido: &[u8],
    frase_paso: Option<&str>,
) -> Result<ResultadoFusion, ErrorImportarCopia> {
    let texto = if cifrador.esta_cifrado(contenido) {
        let frase = frase_paso.ok_or(ErrorImportarCopia::FrasePasoRequerida)?;
        validar_frase_paso(frase).map_err(|_| ErrorImportarCopia::FrasePasoInvalida)?;
        let descifrado = cifrador.descifrar(contenido, frase).map_err(mapear_error_cifrado)?;
        String::from_utf8(descifrado).map_err(|_| ErrorImportarCopia::FraseIncorrectaODatosDanados)?
    } else {
        String::from_utf8(contenido.to_vec())
            .map_err(|_| ErrorImportarCopia::ArchivoMalformado)?
    };

    let documento = serializador
        .deserializar(&texto)
        .map_err(mapear_error_formato)?;
    let eventos_locales = repo
        .listar_eventos()
        .map_err(|error| ErrorImportarCopia::Repositorio(error.0))?;
    let mut eventos_contexto = eventos_locales;
    eventos_contexto.extend(documento.eventos.iter().cloned());
    validar_documento(&documento, &eventos_contexto).map_err(mapear_error_formato)?;

    repo.fusionar_eventos_atomico(&documento.eventos)
        .map_err(mapear_error_repositorio)
}

fn mapear_error_formato(error: ErrorFormatoLibro) -> ErrorImportarCopia {
    match error {
        ErrorFormatoLibro::VersionNoSoportada(version) => {
            ErrorImportarCopia::VersionNoSoportada(version)
        }
        ErrorFormatoLibro::ArchivoMalformado => ErrorImportarCopia::ArchivoMalformado,
    }
}

fn mapear_error_cifrado(error: ErrorCifradoLibro) -> ErrorImportarCopia {
    match error {
        ErrorCifradoLibro::FraseIncorrectaODatosDanados => {
            ErrorImportarCopia::FraseIncorrectaODatosDanados
        }
        ErrorCifradoLibro::OperacionFallida => ErrorImportarCopia::ErrorCifrado,
    }
}

fn mapear_error_repositorio(error: ErrorRepositorio) -> ErrorImportarCopia {
    ErrorImportarCopia::Repositorio(error.0)
}
