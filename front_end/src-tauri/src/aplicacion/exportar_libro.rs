use thiserror::Error;

use crate::aplicacion::puertos::sincronizacion::{
    CifradorLibro, DocumentoLibro, ErrorCifradoLibro, ErrorFormatoLibro, SerializadorLibro,
};
use crate::aplicacion::puertos::{ErrorRepositorio, Reloj, RepositorioLibro};
use crate::aplicacion::sincronizacion::validar_documento;
use crate::dominio::errores::validar_frase_paso;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct EntradaExportarLibro {
    pub frase_paso: Option<String>,
    pub confirmacion_frase_paso: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CopiaExportada {
    pub contenido: Vec<u8>,
    pub cantidad_eventos: usize,
    pub cifrada: bool,
}

#[derive(Debug, Error)]
pub enum ErrorExportarLibro {
    #[error("No se pudo leer el libro local: {0}")]
    Repositorio(String),
    #[error("No se pudo generar el archivo de copia de seguridad.")]
    Formato,
    #[error("La frase de paso no cumple los requisitos mínimos.")]
    FrasePasoInvalida,
    #[error("Las frases de paso no coinciden.")]
    FrasesPasoNoCoinciden,
    #[error("Debe proporcionar ambas entradas de frase de paso o ninguna.")]
    ConfirmacionIncompleta,
    #[error("No se pudo cifrar la copia de seguridad.")]
    Cifrado,
}

impl From<ErrorRepositorio> for ErrorExportarLibro {
    fn from(error: ErrorRepositorio) -> Self {
        Self::Repositorio(error.0)
    }
}

pub fn exportar_libro(
    repo: &dyn RepositorioLibro,
    reloj: &dyn Reloj,
    serializador: &dyn SerializadorLibro,
    cifrador: &dyn CifradorLibro,
    entrada: EntradaExportarLibro,
) -> Result<CopiaExportada, ErrorExportarLibro> {
    let eventos = repo.listar_eventos()?;
    let documento = DocumentoLibro::nuevo(reloj.ahora(), eventos);
    validar_documento(&documento, &documento.eventos).map_err(mapear_error_formato)?;
    let texto = serializador
        .serializar(&documento)
        .map_err(|_| ErrorExportarLibro::Formato)?;

    match (entrada.frase_paso, entrada.confirmacion_frase_paso) {
        (None, None) => Ok(CopiaExportada {
            contenido: texto.into_bytes(),
            cantidad_eventos: documento.cantidad_eventos,
            cifrada: false,
        }),
        (Some(frase), Some(confirmacion)) => {
            validar_frase_paso(&frase).map_err(|_| ErrorExportarLibro::FrasePasoInvalida)?;
            if frase != confirmacion {
                return Err(ErrorExportarLibro::FrasesPasoNoCoinciden);
            }

            let contenido = cifrador
                .cifrar(texto.as_bytes(), &frase)
                .map_err(mapear_error_cifrado)?;
            Ok(CopiaExportada {
                contenido,
                cantidad_eventos: documento.cantidad_eventos,
                cifrada: true,
            })
        }
        _ => Err(ErrorExportarLibro::ConfirmacionIncompleta),
    }
}

fn mapear_error_formato(error: ErrorFormatoLibro) -> ErrorExportarLibro {
    match error {
        ErrorFormatoLibro::VersionNoSoportada(_) | ErrorFormatoLibro::ArchivoMalformado => {
            ErrorExportarLibro::Formato
        }
    }
}

fn mapear_error_cifrado(_error: ErrorCifradoLibro) -> ErrorExportarLibro {
    ErrorExportarLibro::Cifrado
}
