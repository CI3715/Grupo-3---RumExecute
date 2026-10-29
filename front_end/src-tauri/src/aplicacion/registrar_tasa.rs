use std::str::FromStr;

use rust_decimal::Decimal;

use crate::aplicacion::dto::{RegistrarTasaDto, ResultadoTasaDto};
use crate::aplicacion::proyeccion_monedas::{catalogo_desde_eventos, validar_par_en_catalogo};
use crate::aplicacion::puertos::{ErrorRepositorio, GeneradorId, Reloj, RepositorioLibro};
use crate::dominio::errores::ErrorDominio;
use crate::dominio::evento::{DatosTasaRegistrada, EventoLibro};
use crate::dominio::moneda::CodigoMoneda;
use crate::dominio::tasa_cambio::TasaCambio;

const DISPOSITIVO_LOCAL: &str = "dispositivo-local";

#[derive(Debug)]
pub enum ErrorRegistrarTasa {
    Validacion(ErrorDominio),
    Repositorio(ErrorRepositorio),
}

impl From<ErrorDominio> for ErrorRegistrarTasa {
    fn from(error: ErrorDominio) -> Self {
        Self::Validacion(error)
    }
}

impl From<ErrorRepositorio> for ErrorRegistrarTasa {
    fn from(error: ErrorRepositorio) -> Self {
        Self::Repositorio(error)
    }
}

pub fn registrar_tasa(
    repo: &mut dyn RepositorioLibro,
    reloj: &dyn Reloj,
    ids: &mut dyn GeneradorId,
    entrada: RegistrarTasaDto,
) -> Result<ResultadoTasaDto, ErrorRegistrarTasa> {
    let eventos = repo.listar_eventos()?;
    let catalogo = catalogo_desde_eventos(&eventos)?;
    let origen = CodigoMoneda::parse(&entrada.origen)?;
    let destino = CodigoMoneda::parse(&entrada.destino)?;
    let par = validar_par_en_catalogo(&catalogo, origen, destino)?;
    let valor = Decimal::from_str(entrada.valor.trim()).map_err(|_| {
        ErrorDominio::new("valor", "El valor de la tasa no es un decimal válido.")
    })?;
    let momento_actual = reloj.ahora();
    let tasa = TasaCambio::nueva(
        par.origen,
        par.destino,
        valor,
        &entrada.fecha_vigencia,
        &entrada.fuente,
        &momento_actual,
    )?;

    let id_tasa = ids.nuevo_id();
    let evento = EventoLibro::nuevo_tasa_registrada(
        id_tasa.clone(),
        momento_actual,
        DISPOSITIVO_LOCAL,
        DatosTasaRegistrada {
            par_monedas: tasa.par,
            valor: tasa.valor,
            fecha: tasa.fecha,
            fuente: tasa.fuente,
        },
    );
    repo.guardar(evento)?;

    Ok(ResultadoTasaDto {
        exito: true,
        id_tasa,
    })
}
