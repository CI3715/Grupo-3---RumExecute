use crate::aplicacion::dto::{
    ListarHistorialTasasEntrada, TasaCambioDto,
};
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
    let origen_limpio = entrada.origen.trim();
    let destino_limpio = entrada.destino.trim();

    let par_filtro = match (origen_limpio.is_empty(), destino_limpio.is_empty()) {
        (true, true) => None,
        (false, false) => {
            let origen = CodigoMoneda::parse(origen_limpio)?;
            let destino = CodigoMoneda::parse(destino_limpio)?;
            Some(validar_par_en_catalogo(&catalogo, origen, destino)?)
        }
        (false, true) => {
            return Err(ErrorDominio::new(
                "destino",
                "Debe especificar la moneda destino si indica la moneda origen.",
            )
            .into());
        }
        (true, false) => {
            return Err(ErrorDominio::new(
                "origen",
                "Debe especificar la moneda origen si indica la moneda destino.",
            )
            .into());
        }
    };

    Ok(historial_tasas_desde_eventos(&eventos)?
        .into_iter()
        .filter(|tasa| match par_filtro {
            Some(par) => tasa.tasa.par == par,
            None => true,
        })
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
