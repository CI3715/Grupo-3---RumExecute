use crate::dominio::errores::ErrorDominio;
use crate::dominio::evento::EventoLibro;
use crate::dominio::moneda::{CatalogoMonedas, CodigoMoneda, Moneda};
use crate::dominio::tasa_cambio::{ParMonedas, TasaCambio};

#[derive(Debug, Clone)]
pub(crate) struct TasaIdentificada {
    pub id: String,
    pub tasa: TasaCambio,
    pub orden_registro: u64,
    pub eliminada: bool,
}

pub(crate) fn catalogo_desde_eventos(
    eventos: &[EventoLibro],
) -> Result<CatalogoMonedas, ErrorDominio> {
    let mut catalogo = CatalogoMonedas::nuevo();
    for evento in eventos {
        if let EventoLibro::MonedaCreada { datos, .. } = evento {
            let moneda = Moneda::nueva_propia(
                datos.codigo.as_str(),
                datos.nombre.clone(),
                datos.simbolo.clone(),
                datos.decimales,
            )?;
            if catalogo.obtener(&moneda.codigo).is_some() {
                return Err(ErrorDominio::new(
                    "codigo",
                    format!(
                        "Ya existe una moneda registrada con el código '{}'.",
                        moneda.codigo
                    ),
                ));
            }
            catalogo.registrar_moneda(moneda);
        }
    }
    Ok(catalogo)
}

pub(crate) fn historial_tasas_desde_eventos(
    eventos: &[EventoLibro],
) -> Result<Vec<TasaIdentificada>, ErrorDominio> {
    let mut tasas: Vec<TasaIdentificada> = Vec::new();
    for (orden, evento) in eventos.iter().enumerate() {
        match evento {
            EventoLibro::TasaRegistrada { id, datos, .. } => {
                tasas.push(TasaIdentificada {
                    id: id.clone(),
                    tasa: TasaCambio {
                        par: datos.par_monedas,
                        valor: datos.valor,
                        fecha: datos.fecha.clone(),
                        fuente: datos.fuente.clone(),
                    },
                    orden_registro: orden as u64,
                    eliminada: false,
                });
            }
            EventoLibro::TasaEliminada { datos, .. } => {
                let indice = match datos.id_tasa.as_deref() {
                    Some(id_tasa) => tasas
                        .iter()
                        .position(|tasa| tasa.id == id_tasa && !tasa.eliminada),
                    None => tasas.iter().rposition(|tasa| {
                        tasa.tasa.par == datos.par_monedas
                            && tasa.tasa.fecha == datos.fecha
                            && tasa.tasa.fuente == datos.fuente
                            && !tasa.eliminada
                    }),
                };
                let indice = indice.ok_or_else(|| {
                    ErrorDominio::new(
                        "id_tasa",
                        "El evento de eliminación referencia una tasa inexistente o ya eliminada.",
                    )
                })?;
                tasas[indice].eliminada = true;
            }
            _ => {}
        }
    }
    Ok(tasas)
}

pub(crate) fn tasas_activas_desde_eventos(
    eventos: &[EventoLibro],
) -> Result<Vec<TasaIdentificada>, ErrorDominio> {
    Ok(historial_tasas_desde_eventos(eventos)?
        .into_iter()
        .filter(|tasa| !tasa.eliminada)
        .collect())
}

pub(crate) fn moneda_base_desde_eventos(eventos: &[EventoLibro]) -> CodigoMoneda {
    eventos
        .iter()
        .filter_map(|evento| match evento {
            EventoLibro::MonedaBaseCambiada { datos, .. } => Some(datos.nueva_moneda),
            _ => None,
        })
        .last()
        .unwrap_or(CodigoMoneda::USD)
}

pub(crate) fn validar_par_en_catalogo(
    catalogo: &CatalogoMonedas,
    origen: CodigoMoneda,
    destino: CodigoMoneda,
) -> Result<ParMonedas, ErrorDominio> {
    if catalogo.obtener(&origen).is_none() {
        return Err(ErrorDominio::moneda_no_encontrada(origen.as_str()));
    }
    if catalogo.obtener(&destino).is_none() {
        return Err(ErrorDominio::moneda_no_encontrada(destino.as_str()));
    }
    ParMonedas::nuevo(origen, destino)
}
