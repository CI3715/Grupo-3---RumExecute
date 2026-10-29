use crate::aplicacion::dto::{CambiarMonedaBaseDto, ResultadoMonedaBaseDto};
use crate::aplicacion::proyeccion_monedas::catalogo_desde_eventos;
use crate::aplicacion::puertos::{ErrorRepositorio, GeneradorId, Reloj, RepositorioLibro};
use crate::dominio::errores::ErrorDominio;
use crate::dominio::evento::{DatosMonedaBaseCambiada, EventoLibro};
use crate::dominio::moneda::CodigoMoneda;

const DISPOSITIVO_LOCAL: &str = "dispositivo-local";

#[derive(Debug)]
pub enum ErrorCambiarMonedaBase {
    Validacion(ErrorDominio),
    Repositorio(ErrorRepositorio),
}

impl From<ErrorDominio> for ErrorCambiarMonedaBase {
    fn from(error: ErrorDominio) -> Self {
        Self::Validacion(error)
    }
}

impl From<ErrorRepositorio> for ErrorCambiarMonedaBase {
    fn from(error: ErrorRepositorio) -> Self {
        Self::Repositorio(error)
    }
}

pub fn cambiar_moneda_base(
    repo: &mut dyn RepositorioLibro,
    reloj: &dyn Reloj,
    ids: &mut dyn GeneradorId,
    entrada: CambiarMonedaBaseDto,
) -> Result<ResultadoMonedaBaseDto, ErrorCambiarMonedaBase> {
    let eventos = repo.listar_eventos()?;
    let catalogo = catalogo_desde_eventos(&eventos)?;
    let codigo = CodigoMoneda::parse(&entrada.nuevo_codigo)?;
    if catalogo.obtener(&codigo).is_none() {
        return Err(ErrorDominio::moneda_no_encontrada(codigo.as_str()).into());
    }

    repo.guardar(EventoLibro::nuevo_moneda_base_cambiada(
        ids.nuevo_id(),
        reloj.ahora(),
        DISPOSITIVO_LOCAL,
        DatosMonedaBaseCambiada {
            nueva_moneda: codigo,
        },
    ))?;

    Ok(ResultadoMonedaBaseDto {
        exito: true,
        moneda_base: codigo.to_string(),
    })
}
