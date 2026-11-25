use std::collections::HashMap;

use rust_decimal::Decimal;

use crate::aplicacion::puertos::sincronizacion::{
    DocumentoLibro, ErrorFormatoLibro, VERSION_FORMATO_LIBRO,
};
use crate::dominio::categoria::{
    catalogo_categorias, validar_categoria_para_tipo_en, Categoria, TipoCategoria,
};
use crate::dominio::cuenta::Cuenta;
use crate::dominio::evento::EventoLibro;
use crate::dominio::moneda::{CatalogoMonedas, CodigoMoneda, Moneda};
use crate::dominio::monto::Monto;
use crate::dominio::movimiento::{validar_fecha_no_futura, TipoMovimiento};
use crate::dominio::tasa_cambio::{ParMonedas, TasaCambio};

pub(crate) fn validar_documento(
    documento: &DocumentoLibro,
    eventos_contexto: &[EventoLibro],
) -> Result<(), ErrorFormatoLibro> {
    if documento.version_formato != VERSION_FORMATO_LIBRO {
        return Err(ErrorFormatoLibro::VersionNoSoportada(
            documento.version_formato.to_string(),
        ));
    }

    if chrono::DateTime::parse_from_rfc3339(&documento.fecha_exportacion).is_err()
        || documento.cantidad_eventos != documento.eventos.len()
    {
        return Err(ErrorFormatoLibro::ArchivoMalformado);
    }

    let monedas = construir_catalogo_monedas(eventos_contexto)?;
    let categorias = construir_catalogo_categorias(eventos_contexto)?;
    let cuentas = construir_cuentas(eventos_contexto, &monedas)?;

    for evento in &documento.eventos {
        validar_evento(evento, eventos_contexto, &monedas, &categorias, &cuentas)?;
    }

    Ok(())
}

fn construir_catalogo_monedas(
    eventos: &[EventoLibro],
) -> Result<CatalogoMonedas, ErrorFormatoLibro> {
    let mut catalogo = CatalogoMonedas::nuevo();
    for evento in eventos {
        if let EventoLibro::MonedaCreada { datos, .. } = evento {
            if matches!(
                datos.codigo,
                CodigoMoneda::VES | CodigoMoneda::USD | CodigoMoneda::EUR | CodigoMoneda::USDT
            ) {
                return Err(ErrorFormatoLibro::ArchivoMalformado);
            }

            let moneda = Moneda::nueva_propia(
                datos.codigo.as_str(),
                datos.nombre.clone(),
                datos.simbolo.clone(),
                datos.decimales,
            )
            .map_err(|_| ErrorFormatoLibro::ArchivoMalformado)?;
            if moneda.nombre != datos.nombre || moneda.simbolo != datos.simbolo {
                return Err(ErrorFormatoLibro::ArchivoMalformado);
            }

            if let Some(existente) = catalogo.obtener(&moneda.codigo) {
                if existente != &moneda {
                    return Err(ErrorFormatoLibro::ArchivoMalformado);
                }
            } else {
                catalogo.registrar_moneda(moneda);
            }
        }
    }
    Ok(catalogo)
}

fn construir_catalogo_categorias(
    eventos: &[EventoLibro],
) -> Result<Vec<Categoria>, ErrorFormatoLibro> {
    let mut categorias = catalogo_categorias();
    for evento in eventos {
        if let EventoLibro::CategoriaCreada { datos, .. } = evento {
            if datos.id_categoria.trim().is_empty() || datos.nombre.trim().is_empty() {
                return Err(ErrorFormatoLibro::ArchivoMalformado);
            }
            let categoria =
                Categoria::nueva(datos.id_categoria.clone(), datos.nombre.clone(), datos.tipo)
                    .map_err(|_| ErrorFormatoLibro::ArchivoMalformado)?;
            if categoria.id != datos.id_categoria || categoria.nombre != datos.nombre {
                return Err(ErrorFormatoLibro::ArchivoMalformado);
            }
            if let Some(existente) = categorias
                .iter()
                .find(|existente| existente.id == categoria.id)
            {
                if existente != &categoria {
                    return Err(ErrorFormatoLibro::ArchivoMalformado);
                }
            } else {
                categorias.push(categoria);
            }
        }
    }
    Ok(categorias)
}

fn construir_cuentas(
    eventos: &[EventoLibro],
    monedas: &CatalogoMonedas,
) -> Result<HashMap<String, Cuenta>, ErrorFormatoLibro> {
    let mut cuentas = HashMap::new();
    for evento in eventos {
        if let EventoLibro::CuentaCreada { datos, .. } = evento {
            if datos.id_cuenta.trim().is_empty() {
                return Err(ErrorFormatoLibro::ArchivoMalformado);
            }
            let decimales = monedas
                .decimales(&datos.moneda)
                .map_err(|_| ErrorFormatoLibro::ArchivoMalformado)?;
            let saldo = Monto::nuevo_saldo_inicial(
                datos.saldo_inicial.cantidad,
                datos.saldo_inicial.moneda,
                decimales,
            )
            .map_err(|_| ErrorFormatoLibro::ArchivoMalformado)?;
            let cuenta = Cuenta::nueva(
                datos.id_cuenta.clone(),
                datos.nombre.clone(),
                datos.tipo,
                datos.moneda,
                saldo,
            )
            .map_err(|_| ErrorFormatoLibro::ArchivoMalformado)?;
            if cuenta.nombre != datos.nombre {
                return Err(ErrorFormatoLibro::ArchivoMalformado);
            }
            if let Some(existente) = cuentas.get(&cuenta.id) {
                if existente != &cuenta {
                    return Err(ErrorFormatoLibro::ArchivoMalformado);
                }
            } else {
                cuentas.insert(cuenta.id.clone(), cuenta);
            }
        }
    }
    Ok(cuentas)
}

fn validar_evento(
    evento: &EventoLibro,
    eventos_contexto: &[EventoLibro],
    monedas: &CatalogoMonedas,
    categorias: &[Categoria],
    cuentas: &HashMap<String, Cuenta>,
) -> Result<(), ErrorFormatoLibro> {
    evento
        .validar()
        .map_err(|_| ErrorFormatoLibro::ArchivoMalformado)?;

    let tipo_esperado = match evento {
        EventoLibro::CuentaCreada { .. } => "cuenta_creada",
        EventoLibro::MovimientoRegistrado { .. } => "movimiento_registrado",
        EventoLibro::MovimientoCorregido { .. } => "movimiento_corregido",
        EventoLibro::CategoriaCreada { .. } => "categoria_creada",
        EventoLibro::TasaRegistrada { .. } => "tasa_registrada",
        EventoLibro::TasaEliminada { .. } => "tasa_eliminada",
        EventoLibro::MonedaBaseCambiada { .. } => "moneda_base_cambiada",
        EventoLibro::MonedaCreada { .. } => "moneda_creada",
        EventoLibro::GastoDeGrupoAplicado { .. } => "gasto_de_grupo_aplicado",
    };
    if evento.tipo() != tipo_esperado {
        return Err(ErrorFormatoLibro::ArchivoMalformado);
    }

    match evento {
        EventoLibro::CuentaCreada { .. } | EventoLibro::CategoriaCreada { .. } => {}
        EventoLibro::MovimientoRegistrado {
            momento_registro,
            datos,
            ..
        } => validar_movimiento_registrado(
            datos.id_movimiento.as_str(),
            &datos.id_cuenta,
            datos.tipo,
            datos.monto,
            datos.categoria.as_ref(),
            &datos.fecha,
            momento_registro,
            cuentas,
            monedas,
            categorias,
        )?,
        EventoLibro::MovimientoCorregido {
            momento_registro,
            datos,
            ..
        } => validar_movimiento_corregido(datos, momento_registro, eventos_contexto, monedas, cuentas)?,
        EventoLibro::TasaRegistrada {
            momento_registro,
            datos,
            ..
        } => {
            monedas
                .decimales(&datos.par_monedas.origen)
                .map_err(|_| ErrorFormatoLibro::ArchivoMalformado)?;
            monedas
                .decimales(&datos.par_monedas.destino)
                .map_err(|_| ErrorFormatoLibro::ArchivoMalformado)?;
            TasaCambio::nueva(
                datos.par_monedas.origen,
                datos.par_monedas.destino,
                datos.valor,
                &datos.fecha,
                &datos.fuente,
                momento_registro,
            )
            .map_err(|_| ErrorFormatoLibro::ArchivoMalformado)?;
        }
        EventoLibro::TasaEliminada {
            momento_registro,
            datos,
            ..
        } => {
            ParMonedas::nuevo(datos.par_monedas.origen, datos.par_monedas.destino)
                .map_err(|_| ErrorFormatoLibro::ArchivoMalformado)?;
            monedas
                .decimales(&datos.par_monedas.origen)
                .map_err(|_| ErrorFormatoLibro::ArchivoMalformado)?;
            monedas
                .decimales(&datos.par_monedas.destino)
                .map_err(|_| ErrorFormatoLibro::ArchivoMalformado)?;
            validar_fecha_no_futura(&datos.fecha, momento_registro)
                .map_err(|_| ErrorFormatoLibro::ArchivoMalformado)?;
            if datos.fuente.trim().is_empty()
                || matches!(&datos.id_tasa, Some(id) if id.trim().is_empty())
            {
                return Err(ErrorFormatoLibro::ArchivoMalformado);
            }
        }
        EventoLibro::MonedaBaseCambiada { datos, .. } => {
            monedas
                .decimales(&datos.nueva_moneda)
                .map_err(|_| ErrorFormatoLibro::ArchivoMalformado)?;
        }
        EventoLibro::MonedaCreada { datos, .. } => {
            Moneda::nueva_propia(
                datos.codigo.as_str(),
                datos.nombre.clone(),
                datos.simbolo.clone(),
                datos.decimales,
            )
            .map_err(|_| ErrorFormatoLibro::ArchivoMalformado)?;
        }
        EventoLibro::GastoDeGrupoAplicado {
            id,
            momento_registro,
            datos,
            ..
        } => validar_gasto_de_grupo(id, datos, momento_registro, monedas, cuentas)?,
    }

    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn validar_movimiento_registrado(
    id_movimiento: &str,
    id_cuenta: &str,
    tipo: TipoMovimiento,
    monto: Monto,
    categoria: Option<&Categoria>,
    fecha: &str,
    momento_registro: &str,
    cuentas: &HashMap<String, Cuenta>,
    monedas: &CatalogoMonedas,
    categorias: &[Categoria],
) -> Result<(), ErrorFormatoLibro> {
    if id_movimiento.trim().is_empty() {
        return Err(ErrorFormatoLibro::ArchivoMalformado);
    }
    let cuenta = obtener_cuenta(cuentas, id_cuenta)?;
    let decimales = monedas
        .decimales(&cuenta.moneda)
        .map_err(|_| ErrorFormatoLibro::ArchivoMalformado)?;
    if monto.moneda != cuenta.moneda
        || Monto::nuevo_movimiento(monto.cantidad, monto.moneda, decimales).is_err()
    {
        return Err(ErrorFormatoLibro::ArchivoMalformado);
    }
    validar_fecha_no_futura(fecha, momento_registro)
        .map_err(|_| ErrorFormatoLibro::ArchivoMalformado)?;

    let tipo_categoria = match tipo {
        TipoMovimiento::Gasto => TipoCategoria::Gasto,
        TipoMovimiento::Ingreso => TipoCategoria::Ingreso,
        TipoMovimiento::Ajuste => return Err(ErrorFormatoLibro::ArchivoMalformado),
    };
    if let Some(categoria) = categoria {
        if categoria.id.trim().is_empty() || categoria.nombre.trim().is_empty() {
            return Err(ErrorFormatoLibro::ArchivoMalformado);
        }
        let validada = validar_categoria_para_tipo_en(&categoria.id, tipo_categoria, categorias)
            .map_err(|_| ErrorFormatoLibro::ArchivoMalformado)?;
        if &validada != categoria {
            return Err(ErrorFormatoLibro::ArchivoMalformado);
        }
    }
    Ok(())
}

fn validar_movimiento_corregido(
    datos: &crate::dominio::evento::DatosMovimientoCorregido,
    momento_registro: &str,
    eventos_contexto: &[EventoLibro],
    monedas: &CatalogoMonedas,
    cuentas: &HashMap<String, Cuenta>,
) -> Result<(), ErrorFormatoLibro> {
    if datos.id_ajuste.trim().is_empty() || datos.id_movimiento_original.trim().is_empty() {
        return Err(ErrorFormatoLibro::ArchivoMalformado);
    }

    let cuenta = obtener_cuenta(cuentas, &datos.id_cuenta)?;
    let decimales = monedas
        .decimales(&cuenta.moneda)
        .map_err(|_| ErrorFormatoLibro::ArchivoMalformado)?;
    let id_original = datos.id_movimiento_original.as_str();
    let original = eventos_contexto.iter().find_map(|evento| match evento {
        EventoLibro::MovimientoRegistrado {
            datos: original, ..
        } if original.id_movimiento == id_original => Some(original),
        _ => None,
    });
    let original = original.ok_or(ErrorFormatoLibro::ArchivoMalformado)?;
    if original.id_cuenta != datos.id_cuenta
        || original.monto != datos.monto_original
        || original.monto.moneda != cuenta.moneda
        || datos.monto_nuevo.moneda != cuenta.moneda
        || datos.monto_ajuste.moneda != cuenta.moneda
        || datos.monto_original.cantidad <= Decimal::ZERO
        || datos.monto_nuevo.cantidad <= Decimal::ZERO
        || datos.monto_original.cantidad.normalize().scale() > decimales
        || datos.monto_nuevo.cantidad.normalize().scale() > decimales
        || datos.monto_ajuste.cantidad.normalize().scale() > decimales
    {
        return Err(ErrorFormatoLibro::ArchivoMalformado);
    }

    let ajuste_esperado = match original.tipo {
        TipoMovimiento::Gasto => datos
            .monto_original
            .cantidad
            .checked_sub(datos.monto_nuevo.cantidad),
        TipoMovimiento::Ingreso => datos
            .monto_nuevo
            .cantidad
            .checked_sub(datos.monto_original.cantidad),
        TipoMovimiento::Ajuste => None,
    };
    if ajuste_esperado != Some(datos.monto_ajuste.cantidad)
        || datos.monto_ajuste.cantidad == Decimal::ZERO
    {
        return Err(ErrorFormatoLibro::ArchivoMalformado);
    }

    validar_fecha_no_futura(&datos.fecha, momento_registro)
        .map_err(|_| ErrorFormatoLibro::ArchivoMalformado)
}

fn validar_gasto_de_grupo(
    id_evento: &str,
    datos: &crate::dominio::evento::DatosGastoDeGrupoAplicado,
    momento_registro: &str,
    monedas: &CatalogoMonedas,
    cuentas: &HashMap<String, Cuenta>,
) -> Result<(), ErrorFormatoLibro> {
    let id_evento_esperado = format!("gasto_grupo:{}", datos.id_evento_servidor.trim());
    if datos.id_evento_servidor.trim().is_empty()
        || id_evento != id_evento_esperado.as_str()
        || datos.id_movimiento != EventoLibro::id_movimiento_gasto_grupo(&datos.id_evento_servidor)
        || datos.grupo_id.trim().is_empty()
        || datos.nombre_grupo.trim().is_empty()
        || datos.gasto_id.trim().is_empty()
        || datos.concepto.trim().is_empty()
    {
        return Err(ErrorFormatoLibro::ArchivoMalformado);
    }

    let cuenta = obtener_cuenta(cuentas, &datos.id_cuenta)?;
    let decimales = monedas
        .decimales(&cuenta.moneda)
        .map_err(|_| ErrorFormatoLibro::ArchivoMalformado)?;
    if datos.monto.moneda != cuenta.moneda
        || datos.monto.cantidad <= Decimal::ZERO
        || datos.monto.cantidad.normalize().scale() > decimales
    {
        return Err(ErrorFormatoLibro::ArchivoMalformado);
    }
    validar_fecha_no_futura(&datos.fecha, momento_registro)
        .map_err(|_| ErrorFormatoLibro::ArchivoMalformado)
}

fn obtener_cuenta<'a>(
    cuentas: &'a HashMap<String, Cuenta>,
    id_cuenta: &str,
) -> Result<&'a Cuenta, ErrorFormatoLibro> {
    if id_cuenta.trim().is_empty() {
        return Err(ErrorFormatoLibro::ArchivoMalformado);
    }
    cuentas
        .get(id_cuenta)
        .ok_or(ErrorFormatoLibro::ArchivoMalformado)
}
