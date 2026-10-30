use crate::aplicacion::dto::{ListarHistorialTasasEntrada, TasaCambioDto};
use crate::aplicacion::proyeccion_monedas::{
    catalogo_desde_eventos, historial_tasas_desde_eventos, validar_par_en_catalogo,
};
use crate::aplicacion::puertos::{ErrorRepositorio, RepositorioLibro};
use crate::dominio::errores::ErrorDominio;
use crate::dominio::moneda::CodigoMoneda;

#[derive(Debug)]
pub enum ErrorListarHistorialTasas {
    Validacion(ErrorDominio),
    Repositorio(ErrorRepositorio),
}

impl From<ErrorDominio> for ErrorListarHistorialTasas {
    fn from(error: ErrorDominio) -> Self {
        Self::Validacion(error)
    }
}

impl From<ErrorRepositorio> for ErrorListarHistorialTasas {
    fn from(error: ErrorRepositorio) -> Self {
        Self::Repositorio(error)
    }
}

pub fn listar_historial_tasas(
    repo: &dyn RepositorioLibro,
    entrada: ListarHistorialTasasEntrada,
) -> Result<Vec<TasaCambioDto>, ErrorListarHistorialTasas> {
    let eventos = repo.listar_eventos()?;
    let catalogo = catalogo_desde_eventos(&eventos)?;
    let origen = CodigoMoneda::parse(&entrada.origen)?;
    let destino = CodigoMoneda::parse(&entrada.destino)?;
    let par = validar_par_en_catalogo(&catalogo, origen, destino)?;

    Ok(historial_tasas_desde_eventos(&eventos)?
        .into_iter()
        .filter(|tasa| tasa.tasa.par == par)
        .map(|tasa| TasaCambioDto {
            id: tasa.id,
            origen: tasa.tasa.par.origen.to_string(),
            destino: tasa.tasa.par.destino.to_string(),
            valor: tasa.tasa.valor.normalize().to_string(),
            fecha_vigencia: tasa.tasa.fecha,
            fuente: tasa.tasa.fuente,
            eliminada: tasa.eliminada,
        })
        .collect())
}
