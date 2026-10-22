use serde::Serialize;

use crate::aplicacion::puertos::{ErrorRepositorio, RepositorioLibro};
use crate::dominio::cuenta::Cuenta;
use crate::dominio::errores::ErrorDominio;
use crate::dominio::evento::EventoLibro;
use crate::dominio::monto::Monto;
use crate::dominio::movimiento::{
    aplicar_movimiento_a_saldo, Movimiento, TipoMovimiento,
};

/// Respuesta con el detalle proyectado de la cuenta y su historial de movimientos.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct DetalleCuenta {
    pub cuenta: Cuenta,
    pub saldo_actual: Monto,
    pub total_ingresos: Monto,
    pub total_gastos: Monto,
    pub total_ajustes: Monto,
    pub movimientos: Vec<Movimiento>,
}

#[derive(Debug)]
pub enum ErrorObtenerDetalleCuenta {
    Dominio(ErrorDominio),
    Repositorio(ErrorRepositorio),
}

impl From<ErrorDominio> for ErrorObtenerDetalleCuenta {
    fn from(e: ErrorDominio) -> Self {
        ErrorObtenerDetalleCuenta::Dominio(e)
    }
}

impl From<ErrorRepositorio> for ErrorObtenerDetalleCuenta {
    fn from(e: ErrorRepositorio) -> Self {
        ErrorObtenerDetalleCuenta::Repositorio(e)
    }
}

pub fn obtener_detalle_cuenta(
    repo: &dyn RepositorioLibro,
    id_cuenta: &str,
) -> Result<DetalleCuenta, ErrorObtenerDetalleCuenta> {
    let clean_id = id_cuenta.trim();
    if clean_id.is_empty() {
        return Err(ErrorDominio::new(
            "id_cuenta",
            "El identificador de la cuenta no puede estar vacío.",
        )
        .into());
    }

    let mut cuenta_opt: Option<Cuenta> = None;
    let mut movimientos: Vec<Movimiento> = Vec::new();

    // Recorrer los eventos inmutables para recuperar la cuenta y sus movimientos
    for evento in repo.listar_eventos()? {
        match evento {
            EventoLibro::CuentaCreada { datos, .. } => {
                if datos.id_cuenta == clean_id {
                    cuenta_opt = Some(Cuenta {
                        id: datos.id_cuenta,
                        nombre: datos.nombre,
                        tipo: datos.tipo,
                        moneda: datos.moneda,
                        saldo_inicial: datos.saldo_inicial,
                    });
                }
            }
            EventoLibro::MovimientoRegistrado { datos, .. } => {
                if datos.id_cuenta == clean_id {
                    movimientos.push(Movimiento {
                        id: datos.id_movimiento,
                        id_cuenta: datos.id_cuenta,
                        tipo: datos.tipo,
                        monto: datos.monto,
                        categoria: datos.categoria,
                        fecha: datos.fecha,
                        descripcion: datos.descripcion,
                        id_movimiento_referenciado: None,
                    });
                }
            }
            EventoLibro::MovimientoCorregido { datos, .. } => {
                if datos.id_cuenta == clean_id {
                    movimientos.push(Movimiento {
                        id: datos.id_ajuste,
                        id_cuenta: datos.id_cuenta,
                        tipo: TipoMovimiento::Ajuste,
                        monto: datos.monto_ajuste,
                        categoria: None,
                        fecha: datos.fecha,
                        descripcion: datos.descripcion,
                        id_movimiento_referenciado: Some(datos.id_movimiento_original),
                    });
                }
            }
        }
    }

    // Validar que la cuenta existe
    let cuenta = cuenta_opt.ok_or_else(|| {
        ErrorDominio::new(
            "id_cuenta",
            format!("No existe ninguna cuenta registrada con el id '{clean_id}'."),
        )
    })?;

    // Proyectar acumulados por tipo de movimiento y calcular el saldo actual exacto (INV-05)
    let mut saldo_actual = cuenta.saldo_inicial;
    let mut total_ingresos = Monto::cero(cuenta.moneda);
    let mut total_gastos = Monto::cero(cuenta.moneda);
    let mut total_ajustes = Monto::cero(cuenta.moneda);

    for mov in &movimientos {
        saldo_actual = aplicar_movimiento_a_saldo(&saldo_actual, mov)?;

        match mov.tipo {
            TipoMovimiento::Ingreso => {
                total_ingresos = total_ingresos.sumar(&mov.monto)?;
            }
            TipoMovimiento::Gasto => {
                total_gastos = total_gastos.sumar(&mov.monto)?;
            }
            TipoMovimiento::Ajuste => {
                total_ajustes = total_ajustes.sumar(&mov.monto)?;
            }
        }
    }

    Ok(DetalleCuenta {
        cuenta,
        saldo_actual,
        total_ingresos,
        total_gastos,
        total_ajustes,
        movimientos,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dominio::cuenta::TipoCuenta;
    use crate::dominio::evento::{DatosCuentaCreada, DatosMovimientoRegistrado};
    use crate::dominio::moneda::CodigoMoneda;
    use crate::infraestructura::memoria_repo::RepositorioLibroEnMemoria;
    use rust_decimal::Decimal;

    fn crear_cuenta_base(repo: &mut RepositorioLibroEnMemoria, id_cuenta: &str, saldo: &str) {
        let datos = DatosCuentaCreada {
            id_cuenta: id_cuenta.to_string(),
            nombre: "Cuenta Banco".to_string(),
            tipo: TipoCuenta::Banco,
            moneda: CodigoMoneda::USD,
            saldo_inicial: Monto::parse_saldo_inicial(saldo, CodigoMoneda::USD).unwrap(),
        };
        repo.guardar(EventoLibro::nuevo_cuenta_creada(
            "evt-c1",
            "2026-10-21T10:00:00Z",
            "disp-test",
            datos,
        ))
        .unwrap();
    }

    #[test]
    fn proyecta_saldo_y_movimientos_correctamente() {
        let mut repo = RepositorioLibroEnMemoria::nuevo();
        crear_cuenta_base(&mut repo, "cta-1", "100,00");

        // Agregar un gasto de 30.00
        let datos_gasto = DatosMovimientoRegistrado {
            id_movimiento: "mov-1".to_string(),
            id_cuenta: "cta-1".to_string(),
            tipo: TipoMovimiento::Gasto,
            monto: Monto::parse_movimiento("30,00", CodigoMoneda::USD).unwrap(),
            categoria: None,
            fecha: "2026-10-21".to_string(),
            descripcion: "Comida".to_string(),
        };
        repo.guardar(EventoLibro::nuevo_movimiento_registrado(
            "evt-m1",
            "2026-10-21T11:00:00Z",
            "disp-test",
            datos_gasto,
        ))
        .unwrap();

        let detalle = obtener_detalle_cuenta(&repo, "cta-1").unwrap();

        assert_eq!(detalle.saldo_actual.cantidad, Decimal::new(7000, 2)); // 100 - 30 = 70.00
        assert_eq!(detalle.total_gastos.cantidad, Decimal::new(3000, 2));
        assert_eq!(detalle.movimientos.len(), 1);
    }

    #[test]
    fn rechaza_si_la_cuenta_no_existe() {
        let repo = RepositorioLibroEnMemoria::nuevo();
        let err = obtener_detalle_cuenta(&repo, "cta-inexistente").expect_err("Debe fallar");

        let ErrorObtenerDetalleCuenta::Dominio(e) = err else {
            panic!("Se esperaba error de dominio");
        };
        assert_eq!(e.campo, "id_cuenta");
    }
}
