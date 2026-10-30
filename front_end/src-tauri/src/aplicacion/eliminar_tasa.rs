use crate::aplicacion::dto::{EliminarTasaDto, ResultadoOperacionDto};
use crate::aplicacion::proyeccion_monedas::tasas_activas_desde_eventos;
use crate::aplicacion::puertos::{ErrorRepositorio, GeneradorId, Reloj, RepositorioLibro};
use crate::dominio::errores::ErrorDominio;
use crate::dominio::evento::{DatosTasaEliminada, EventoLibro};

const DISPOSITIVO_LOCAL: &str = "dispositivo-local";

#[derive(Debug)]
pub enum ErrorEliminarTasa {
    Validacion(ErrorDominio),
    Repositorio(ErrorRepositorio),
}

impl From<ErrorDominio> for ErrorEliminarTasa {
    fn from(error: ErrorDominio) -> Self {
        Self::Validacion(error)
    }
}

impl From<ErrorRepositorio> for ErrorEliminarTasa {
    fn from(error: ErrorRepositorio) -> Self {
        Self::Repositorio(error)
    }
}

pub fn eliminar_tasa(
    repo: &mut dyn RepositorioLibro,
    reloj: &dyn Reloj,
    ids: &mut dyn GeneradorId,
    entrada: EliminarTasaDto,
) -> Result<ResultadoOperacionDto, ErrorEliminarTasa> {
    let id_tasa = entrada.id_tasa.trim();
    if id_tasa.is_empty() {
        return Err(ErrorDominio::new(
            "id_tasa",
            "El identificador de la tasa no puede estar vacío.",
        )
        .into());
    }

    let eventos = repo.listar_eventos()?;
    let tasa = tasas_activas_desde_eventos(&eventos)?
        .into_iter()
        .find(|tasa| tasa.id == id_tasa)
        .ok_or_else(|| {
            ErrorDominio::new(
                "id_tasa",
                format!("No existe una tasa activa con el identificador '{id_tasa}'."),
            )
        })?;

    let evento = EventoLibro::nuevo_tasa_eliminada(
        ids.nuevo_id(),
        reloj.ahora(),
        DISPOSITIVO_LOCAL,
        DatosTasaEliminada {
            par_monedas: tasa.tasa.par,
            fecha: tasa.tasa.fecha,
            fuente: tasa.tasa.fuente,
            id_tasa: Some(tasa.id),
        },
    );
    repo.guardar(evento)?;

    Ok(ResultadoOperacionDto { exito: true })
}
