use crate::aplicacion::dto::{CrearMonedaDto, ResultadoMonedaDto};
use crate::aplicacion::proyeccion_monedas::catalogo_desde_eventos;
use crate::aplicacion::puertos::{ErrorRepositorio, GeneradorId, Reloj, RepositorioLibro};
use crate::dominio::errores::ErrorDominio;
use crate::dominio::evento::{DatosMonedaCreada, EventoLibro};
use crate::dominio::moneda::Moneda;

const DISPOSITIVO_LOCAL: &str = "dispositivo-local";

#[derive(Debug)]
pub enum ErrorCrearMoneda {
    Validacion(ErrorDominio),
    Repositorio(ErrorRepositorio),
}

impl From<ErrorDominio> for ErrorCrearMoneda {
    fn from(error: ErrorDominio) -> Self {
        Self::Validacion(error)
    }
}

impl From<ErrorRepositorio> for ErrorCrearMoneda {
    fn from(error: ErrorRepositorio) -> Self {
        Self::Repositorio(error)
    }
}

pub fn crear_moneda(
    repo: &mut dyn RepositorioLibro,
    reloj: &dyn Reloj,
    ids: &mut dyn GeneradorId,
    entrada: CrearMonedaDto,
) -> Result<ResultadoMonedaDto, ErrorCrearMoneda> {
    let eventos = repo.listar_eventos()?;
    let catalogo = catalogo_desde_eventos(&eventos)?;
    let moneda = Moneda::nueva_propia(
        &entrada.codigo,
        entrada.nombre,
        entrada.simbolo,
        entrada.decimales,
    )?;

    if catalogo.obtener(&moneda.codigo).is_some() {
        return Err(ErrorDominio::new(
            "codigo",
            format!(
                "Ya existe una moneda registrada con el código '{}'.",
                moneda.codigo
            ),
        )
        .into());
    }

    let codigo = moneda.codigo.to_string();
    repo.guardar(EventoLibro::nuevo_moneda_creada(
        ids.nuevo_id(),
        reloj.ahora(),
        DISPOSITIVO_LOCAL,
        DatosMonedaCreada {
            codigo: moneda.codigo,
            nombre: moneda.nombre,
            simbolo: moneda.simbolo,
            decimales: moneda.decimales,
        },
    ))?;

    Ok(ResultadoMonedaDto {
        exito: true,
        codigo,
    })
}
