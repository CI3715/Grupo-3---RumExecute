use crate::aplicacion::dto::ConsultarPatrimonioEntrada;
use crate::aplicacion::proyeccion_monedas::{
    catalogo_desde_eventos, moneda_base_desde_eventos, tasas_activas_desde_eventos,
};
use crate::aplicacion::puertos::{ErrorRepositorio, Reloj, RepositorioLibro};
use crate::dominio::cuenta::Cuenta;
use crate::dominio::errores::ErrorDominio;
use crate::dominio::evento::EventoLibro;
use crate::dominio::monto::Monto;
use crate::dominio::movimiento::{
    aplicar_movimiento_a_saldo, Movimiento, TipoMovimiento, validar_fecha_no_futura,
};
use rust_decimal::Decimal;
use std::collections::HashSet;

use crate::aplicacion::dto::{
    CuentaPatrimonioDto, CuentaPatrimonioProyeccion, ErrorValidacionDto, MonedaBaseProyeccion,
    PatrimonioDto, SubtotalPatrimonioDto, TasaCambioProyeccion, TasaFaltanteDto, TasaUsadaDto,
};

#[derive(Debug)]
pub enum ErrorConsultarPatrimonio {
    Validacion(ErrorDominio),
    Repositorio(ErrorRepositorio),
}

impl From<ErrorDominio> for ErrorConsultarPatrimonio {
    fn from(error: ErrorDominio) -> Self {
        Self::Validacion(error)
    }
}

impl From<ErrorRepositorio> for ErrorConsultarPatrimonio {
    fn from(error: ErrorRepositorio) -> Self {
        Self::Repositorio(error)
    }
}

struct CuentaProyectada {
    cuenta: Cuenta,
    saldo: Monto,
}

pub fn consultar_patrimonio(
    repo: &dyn RepositorioLibro,
    reloj: &dyn Reloj,
    entrada: ConsultarPatrimonioEntrada,
) -> Result<PatrimonioDto, ErrorConsultarPatrimonio> {
    let eventos = repo.listar_eventos()?;
    let ahora = reloj.ahora();
    let fecha_consulta = entrada
        .fecha_consulta
        .unwrap_or_else(|| ahora.chars().take(10).collect());
    if fecha_consulta.len() != 10 {
        return Err(ErrorDominio::fecha_invalida(
            "La fecha de consulta debe tener el formato YYYY-MM-DD.",
        )
        .into());
    }
    validar_fecha_no_futura(&fecha_consulta, &ahora)?;

    let catalogo = catalogo_desde_eventos(&eventos)?;
    let codigo_base = moneda_base_desde_eventos(&eventos);
    let decimales_base = catalogo.decimales(&codigo_base)?;
    let cuentas = reconstruir_cuentas(&eventos, &fecha_consulta)?;
    let tasas = tasas_activas_desde_eventos(&eventos)?
        .into_iter()
        .map(|identificada| TasaCambioProyeccion {
            id: identificada.id,
            origen: identificada.tasa.par.origen.to_string(),
            destino: identificada.tasa.par.destino.to_string(),
            valor: identificada.tasa.valor,
            fecha_vigencia: identificada.tasa.fecha,
            fuente: identificada.tasa.fuente,
            orden_registro: identificada.orden_registro,
        })
        .collect::<Vec<_>>();

    let cuentas = cuentas
        .into_iter()
        .map(|proyectada| CuentaPatrimonioProyeccion {
            id: proyectada.cuenta.id,
            nombre: proyectada.cuenta.nombre,
            tipo: proyectada.cuenta.tipo.as_str().to_string(),
            saldo: proyectada.saldo.cantidad,
            moneda: proyectada.saldo.moneda.to_string(),
        })
        .collect::<Vec<_>>();

    calcular_patrimonio(
        &fecha_consulta,
        &MonedaBaseProyeccion {
            codigo: codigo_base.to_string(),
            decimales: decimales_base,
        },
        &cuentas,
        &tasas,
    )
    .map_err(|error| {
        ErrorConsultarPatrimonio::Validacion(ErrorDominio::new(error.campo, error.mensaje))
    })
}

fn reconstruir_cuentas(
    eventos: &[EventoLibro],
    fecha_consulta: &str,
) -> Result<Vec<CuentaProyectada>, ErrorDominio> {
    let mut cuentas: Vec<CuentaProyectada> = Vec::new();
    let mut eventos_vistos = HashSet::new();
    for evento in eventos {
        if !eventos_vistos.insert(evento.id().to_string()) {
            continue;
        }

        match evento {
            EventoLibro::CuentaCreada {
                datos,
                momento_registro,
                ..
            } if fecha_del_dia(momento_registro) <= fecha_consulta => {
                cuentas.push(CuentaProyectada {
                    cuenta: Cuenta {
                        id: datos.id_cuenta.clone(),
                        nombre: datos.nombre.clone(),
                        tipo: datos.tipo,
                        moneda: datos.moneda,
                        saldo_inicial: datos.saldo_inicial,
                    },
                    saldo: datos.saldo_inicial,
                })
            }
            EventoLibro::MovimientoRegistrado { datos, .. }
                if fecha_del_dia(&datos.fecha) <= fecha_consulta =>
            {
                let movimiento = Movimiento {
                    id: datos.id_movimiento.clone(),
                    id_cuenta: datos.id_cuenta.clone(),
                    tipo: datos.tipo,
                    monto: datos.monto,
                    categoria: datos.categoria.clone(),
                    fecha: datos.fecha.clone(),
                    descripcion: datos.descripcion.clone(),
                    id_movimiento_referenciado: None,
                };
                aplicar_a_cuenta(&mut cuentas, &movimiento)?;
            }
            EventoLibro::MovimientoCorregido { datos, .. }
                if fecha_del_dia(&datos.fecha) <= fecha_consulta =>
            {
                let movimiento = Movimiento {
                    id: datos.id_ajuste.clone(),
                    id_cuenta: datos.id_cuenta.clone(),
                    tipo: TipoMovimiento::Ajuste,
                    monto: datos.monto_ajuste,
                    categoria: None,
                    fecha: datos.fecha.clone(),
                    descripcion: datos.descripcion.clone(),
                    id_movimiento_referenciado: Some(datos.id_movimiento_original.clone()),
                };
                aplicar_a_cuenta(&mut cuentas, &movimiento)?;
            }
            _ => {}
        }
    }
    Ok(cuentas)
}

fn aplicar_a_cuenta(
    cuentas: &mut [CuentaProyectada],
    movimiento: &Movimiento,
) -> Result<(), ErrorDominio> {
    let cuenta = cuentas
        .iter_mut()
        .find(|cuenta| cuenta.cuenta.id == movimiento.id_cuenta)
        .ok_or_else(|| {
            ErrorDominio::new(
                "id_cuenta",
                format!(
                    "El movimiento '{}' referencia una cuenta inexistente.",
                    movimiento.id
                ),
            )
        })?;
    cuenta.saldo = aplicar_movimiento_a_saldo(&cuenta.saldo, movimiento)?;
    Ok(())
}

fn fecha_del_dia(fecha: &str) -> &str {
    fecha.get(..10).unwrap_or(fecha)
}

#[derive(Debug, Clone, Copy)]
struct TasaSeleccionada<'a> {
    tasa: &'a TasaCambioProyeccion,
    inversa: bool,
}

pub fn calcular_patrimonio(
    fecha_consulta: &str,
    moneda_base: &MonedaBaseProyeccion,
    cuentas: &[CuentaPatrimonioProyeccion],
    tasas: &[TasaCambioProyeccion],
) -> Result<PatrimonioDto, ErrorValidacionDto> {
    validar_fecha(fecha_consulta, "fecha_consulta")?;
    validar_codigo_moneda(&moneda_base.codigo, "moneda_base")?;
    if moneda_base.decimales > 8 {
        return Err(error_validacion(
            "moneda_base",
            "La moneda base no puede admitir más de 8 decimales.",
        ));
    }

    let mut subtotales: Vec<(String, Decimal)> = Vec::new();
    for cuenta in cuentas {
        validar_codigo_moneda(&cuenta.moneda, "moneda_original")?;
        if let Some((_, total)) = subtotales
            .iter_mut()
            .find(|(moneda, _)| moneda == &cuenta.moneda)
        {
            *total = total.checked_add(cuenta.saldo).ok_or_else(|| {
                error_validacion("saldo", "El subtotal excede la precisión decimal permitida.")
            })?;
        } else {
            subtotales.push((cuenta.moneda.clone(), cuenta.saldo));
        }
    }

    for tasa in tasas {
        validar_codigo_moneda(&tasa.origen, "origen")?;
        validar_codigo_moneda(&tasa.destino, "destino")?;
        validar_fecha(&tasa.fecha_vigencia, "fecha_vigencia")?;
        if tasa.origen == tasa.destino {
            return Err(error_validacion(
                "par_moneda",
                "El origen y el destino de una tasa deben ser distintos.",
            ));
        }
        if tasa.valor <= Decimal::ZERO {
            return Err(error_validacion(
                "valor",
                "El valor de la tasa debe ser mayor que cero.",
            ));
        }
        if tasa.fuente.trim().is_empty() {
            return Err(error_validacion(
                "fuente",
                "La fuente de la tasa no puede estar vacía.",
            ));
        }
    }

    let mut cuentas_dto = Vec::with_capacity(cuentas.len());
    let mut tasas_usadas: Vec<TasaUsadaDto> = Vec::new();
    let mut total = Decimal::ZERO;
    let mut tasa_faltante = None;

    for cuenta in cuentas {
        let seleccion = seleccionar_tasa(
            &cuenta.moneda,
            &moneda_base.codigo,
            fecha_consulta,
            tasas,
        );

        if cuenta.moneda == moneda_base.codigo {
            total = total.checked_add(cuenta.saldo).ok_or_else(|| {
                error_validacion(
                    "total_consolidado",
                    "El total consolidado excede la precisión decimal permitida.",
                )
            })?;
            cuentas_dto.push(CuentaPatrimonioDto {
                id: cuenta.id.clone(),
                nombre: cuenta.nombre.clone(),
                tipo: cuenta.tipo.clone(),
                saldo_original: cuenta.saldo.to_string(),
                moneda_original: cuenta.moneda.clone(),
                tasa_aplicada: Some("1".to_string()),
                saldo_convertido: Some(cuenta.saldo.to_string()),
            });
            continue;
        }

        if let Some(seleccion) = seleccion {
            let factor = if seleccion.inversa {
                Decimal::ONE
                    .checked_div(seleccion.tasa.valor)
                    .ok_or_else(|| {
                        error_validacion(
                            "valor",
                            "No se pudo calcular la tasa inversa con precisión decimal.",
                        )
                    })?
            } else {
                seleccion.tasa.valor
            };
            let convertido = cuenta.saldo.checked_mul(factor).ok_or_else(|| {
                error_validacion(
                    "saldo_convertido",
                    "La conversión excede la precisión decimal permitida.",
                )
            })?;
            total = total.checked_add(convertido).ok_or_else(|| {
                error_validacion(
                    "total_consolidado",
                    "El total consolidado excede la precisión decimal permitida.",
                )
            })?;

            if !tasas_usadas.iter().any(|usada| {
                usada.moneda_origen == seleccion.tasa.origen
                    && usada.moneda_destino == seleccion.tasa.destino
                    && usada.fecha_vigencia == seleccion.tasa.fecha_vigencia
                    && usada.valor_registrado == seleccion.tasa.valor.normalize().to_string()
                    && usada.inversa == seleccion.inversa
            }) {
                tasas_usadas.push(TasaUsadaDto {
                    moneda_origen: seleccion.tasa.origen.clone(),
                    moneda_destino: seleccion.tasa.destino.clone(),
                    valor: factor.normalize().to_string(),
                    valor_registrado: seleccion.tasa.valor.normalize().to_string(),
                    fecha_vigencia: seleccion.tasa.fecha_vigencia.clone(),
                    fuente: seleccion.tasa.fuente.clone(),
                    inversa: seleccion.inversa,
                });
            }

            cuentas_dto.push(CuentaPatrimonioDto {
                id: cuenta.id.clone(),
                nombre: cuenta.nombre.clone(),
                tipo: cuenta.tipo.clone(),
                saldo_original: cuenta.saldo.to_string(),
                moneda_original: cuenta.moneda.clone(),
                tasa_aplicada: Some(factor.normalize().to_string()),
                saldo_convertido: Some(convertido.to_string()),
            });
        } else {
            if cuenta.saldo != Decimal::ZERO && tasa_faltante.is_none() {
                tasa_faltante = Some(TasaFaltanteDto {
                    moneda_origen: cuenta.moneda.clone(),
                    moneda_destino: moneda_base.codigo.clone(),
                    fecha_requerida: fecha_consulta.to_string(),
                    mensaje: format!(
                        "No se puede calcular el patrimonio consolidado al {fecha_consulta} porque falta la tasa de cambio para el par {} -> {}.",
                        cuenta.moneda, moneda_base.codigo
                    ),
                });
            }

            cuentas_dto.push(CuentaPatrimonioDto {
                id: cuenta.id.clone(),
                nombre: cuenta.nombre.clone(),
                tipo: cuenta.tipo.clone(),
                saldo_original: cuenta.saldo.to_string(),
                moneda_original: cuenta.moneda.clone(),
                tasa_aplicada: None,
                saldo_convertido: if cuenta.saldo == Decimal::ZERO {
                    Some("0".to_string())
                } else {
                    None
                },
            });
        }
    }

    let subtotales = subtotales
        .into_iter()
        .map(|(moneda, total)| SubtotalPatrimonioDto {
            moneda,
            total: total.to_string(),
        })
        .collect();

    let total_consolidado = if tasa_faltante.is_none() {
        Some(formatear_decimal(
            total.round_dp(moneda_base.decimales),
            moneda_base.decimales,
        ))
    } else {
        None
    };

    Ok(PatrimonioDto {
        fecha_consulta: fecha_consulta.to_string(),
        moneda_base: moneda_base.codigo.clone(),
        cuentas: cuentas_dto,
        subtotales,
        tasas_usadas,
        total_consolidado,
        tasa_faltante,
    })
}

fn seleccionar_tasa<'a>(
    origen: &str,
    destino: &str,
    fecha_consulta: &str,
    tasas: &'a [TasaCambioProyeccion],
) -> Option<TasaSeleccionada<'a>> {
    if let Some(tasa) = tasa_vigente(tasas, origen, destino, fecha_consulta) {
        return Some(TasaSeleccionada {
            tasa,
            inversa: false,
        });
    }

    tasa_vigente(tasas, destino, origen, fecha_consulta).map(|tasa| TasaSeleccionada {
        tasa,
        inversa: true,
    })
}

fn tasa_vigente<'a>(
    tasas: &'a [TasaCambioProyeccion],
    origen: &str,
    destino: &str,
    fecha_consulta: &str,
) -> Option<&'a TasaCambioProyeccion> {
    tasas
        .iter()
        .filter(|tasa| {
            tasa.origen == origen
                && tasa.destino == destino
                && tasa.fecha_vigencia.as_str() <= fecha_consulta
        })
        .max_by(|izquierda, derecha| {
            izquierda
                .fecha_vigencia
                .cmp(&derecha.fecha_vigencia)
                .then_with(|| izquierda.orden_registro.cmp(&derecha.orden_registro))
        })
}

fn validar_codigo_moneda(codigo: &str, campo: &str) -> Result<(), ErrorValidacionDto> {
    let valido = (3..=5).contains(&codigo.len())
        && codigo
            .bytes()
            .all(|caracter| caracter.is_ascii_uppercase());
    if valido {
        Ok(())
    } else {
        Err(error_validacion(
            campo,
            "El código de moneda debe tener de 3 a 5 letras mayúsculas ASCII.",
        ))
    }
}

fn validar_fecha(fecha: &str, campo: &str) -> Result<(), ErrorValidacionDto> {
    let partes: Vec<_> = fecha.split('-').collect();
    if partes.len() != 3 || partes[0].len() != 4 || partes[1].len() != 2 || partes[2].len() != 2 {
        return Err(error_validacion(
            campo,
            "La fecha debe tener el formato YYYY-MM-DD.",
        ));
    }

    let anio = partes[0]
        .parse::<u32>()
        .map_err(|_| error_validacion(campo, "El año de la fecha no es válido."))?;
    let mes = partes[1]
        .parse::<u32>()
        .map_err(|_| error_validacion(campo, "El mes de la fecha no es válido."))?;
    let dia = partes[2]
        .parse::<u32>()
        .map_err(|_| error_validacion(campo, "El día de la fecha no es válido."))?;

    let dias_mes = match mes {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if es_bisiesto(anio) => 29,
        2 => 28,
        _ => 0,
    };
    if dia == 0 || dia > dias_mes {
        return Err(error_validacion(campo, "La fecha indicada no existe."));
    }
    Ok(())
}

fn es_bisiesto(anio: u32) -> bool {
    anio.is_multiple_of(400) || (anio.is_multiple_of(4) && !anio.is_multiple_of(100))
}

fn formatear_decimal(valor: Decimal, decimales: u32) -> String {
    let texto = valor.to_string();
    let (entero, fraccion) = texto.split_once('.').unwrap_or((&texto, ""));
    if decimales == 0 {
        return entero.to_string();
    }
    let fraccion = &fraccion[..fraccion.len().min(decimales as usize)];
    format!(
        "{entero}.{fraccion}{}",
        "0".repeat(decimales as usize - fraccion.len())
    )
}

fn error_validacion(campo: &str, mensaje: &str) -> ErrorValidacionDto {
    ErrorValidacionDto {
        campo: campo.to_string(),
        mensaje: mensaje.to_string(),
    }
}
