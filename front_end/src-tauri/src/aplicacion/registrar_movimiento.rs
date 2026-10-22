use crate::aplicacion::puertos::{ErrorRepositorio, GeneradorId, Reloj, RepositorioLibro};
use crate::dominio::cuenta::Cuenta;
use crate::dominio::errores::ErrorDominio;
use crate::dominio::evento::{DatosMovimientoRegistrado, EventoLibro};
use crate::dominio::monto::Monto;
use crate::dominio::movimiento::Movimiento;
use serde::{Deserialize, Serialize};

const DISPOSITIVO_LOCAL: &str = "dispositivo-local";

/// Entrada del usuario para registrar un gasto o ingreso.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EntradaRegistrarMovimiento {
    pub id_cuenta: String,
    pub tipo: String,
    pub monto: String,
    pub categoria: String,
    pub fecha: String,
    pub descripcion: String,
}

#[derive(Debug)]
pub enum ErrorRegistrarMovimiento {
    Validacion(ErrorDominio),
    Repositorio(ErrorRepositorio),
}

impl From<ErrorDominio> for ErrorRegistrarMovimiento {
    fn from(e: ErrorDominio) -> Self {
        ErrorRegistrarMovimiento::Validacion(e)
    }
}

impl From<ErrorRepositorio> for ErrorRegistrarMovimiento {
    fn from(e: ErrorRepositorio) -> Self {
        ErrorRegistrarMovimiento::Repositorio(e)
    }
}

pub fn registrar_movimiento(
    repo: &mut dyn RepositorioLibro,
    reloj: &dyn Reloj,
    ids: &mut dyn GeneradorId,
    entrada: EntradaRegistrarMovimiento,
) -> Result<Movimiento, ErrorRegistrarMovimiento> {
    let momento_actual_utc = reloj.ahora();

    // Buscar la cuenta afectada reconstruyéndola a partir de los eventos del libro
    let mut cuenta_opt: Option<Cuenta> = None;

    for evento in repo.listar_eventos()? {
        if let EventoLibro::CuentaCreada { datos, .. } = evento {
            if datos.id_cuenta == entrada.id_cuenta {
                cuenta_opt = Some(Cuenta {
                    id: datos.id_cuenta,
                    nombre: datos.nombre,
                    tipo: datos.tipo,
                    moneda: datos.moneda,
                    saldo_inicial: datos.saldo_inicial,
                });
                break;
            }
        }
    }

    let cuenta = cuenta_opt.ok_or_else(|| {
        ErrorDominio::new(
            "id_cuenta",
            format!("No existe ninguna cuenta registrada con el id '{}'.", entrada.id_cuenta),
        )
    })?;

    let monto = Monto::parse_movimiento(&entrada.monto, cuenta.moneda)?;

    // Crear el movimiento según el tipo (el dominio valida el resto de invariantes)
    let id_movimiento = ids.nuevo_id();

    let movimiento = match entrada.tipo.trim().to_lowercase().as_str() {
        "gasto" => Movimiento::nuevo_gasto(
            &id_movimiento,
            &cuenta,
            monto,
            &entrada.categoria,
            &entrada.fecha,
            entrada.descripcion,
            &momento_actual_utc,
        )?,
        "ingreso" => Movimiento::nuevo_ingreso(
            &id_movimiento,
            &cuenta,
            monto,
            &entrada.categoria,
            &entrada.fecha,
            entrada.descripcion,
            &momento_actual_utc,
        )?,
        _ => {
            return Err(ErrorDominio::tipo_invalido(format!(
                "El tipo de movimiento '{}' no es válido. Opciones permitidas: 'gasto', 'ingreso'.",
                entrada.tipo
            ))
            .into());
        }
    };

    // Emitir el evento de Event Sourcing
    let datos_evento = DatosMovimientoRegistrado {
        id_movimiento: movimiento.id.clone(),
        id_cuenta: movimiento.id_cuenta.clone(),
        tipo: movimiento.tipo,
        monto: movimiento.monto,
        categoria: movimiento.categoria.clone(),
        fecha: movimiento.fecha.clone(),
        descripcion: movimiento.descripcion.clone(),
    };

    let id_evento = ids.nuevo_id();
    let evento = EventoLibro::nuevo_movimiento_registrado(
        id_evento,
        momento_actual_utc,
        DISPOSITIVO_LOCAL,
        datos_evento,
    );

    repo.guardar(evento)?;

    Ok(movimiento)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dominio::cuenta::TipoCuenta;
    use crate::dominio::evento::DatosCuentaCreada;
    use crate::dominio::moneda::CodigoMoneda;
    use crate::dominio::movimiento::TipoMovimiento;
    use crate::infraestructura::memoria_repo::RepositorioLibroEnMemoria;
    use rust_decimal::Decimal;

    struct RelojFijo;
    impl Reloj for RelojFijo {
        fn ahora(&self) -> String {
            "2026-10-21T12:00:00Z".to_string()
        }
    }

    struct IdsSecuenciales {
        contador: u32,
    }

    impl GeneradorId for IdsSecuenciales {
        fn nuevo_id(&mut self) -> String {
            self.contador += 1;
            format!("id-{}", self.contador)
        }
    }

    struct Prueba {
        repo: RepositorioLibroEnMemoria,
        ids: IdsSecuenciales,
    }

    impl Prueba {
        fn nueva() -> Self {
            let mut repo = RepositorioLibroEnMemoria::nuevo();
            // Creamos una cuenta base para las pruebas
            let datos_cuenta = DatosCuentaCreada {
                id_cuenta: "cta-principal".to_string(),
                nombre: "Banco USD".to_string(),
                tipo: TipoCuenta::Banco,
                moneda: CodigoMoneda::USD,
                saldo_inicial: Monto::cero(CodigoMoneda::USD),
            };
            repo.guardar(EventoLibro::nuevo_cuenta_creada(
                "evt-init",
                "2026-10-21T10:00:00Z",
                "disp-test",
                datos_cuenta,
            ))
            .unwrap();

            Prueba {
                repo,
                ids: IdsSecuenciales { contador: 0 },
            }
        }

        fn registrar(&mut self, entrada: EntradaRegistrarMovimiento) -> Result<Movimiento, ErrorRegistrarMovimiento> {
            registrar_movimiento(&mut self.repo, &RelojFijo, &mut self.ids, entrada)
        }
    }

    fn entrada(id_cuenta: &str, tipo: &str, monto: &str, cat: &str, fecha: &str) -> EntradaRegistrarMovimiento {
        EntradaRegistrarMovimiento {
            id_cuenta: id_cuenta.to_string(),
            tipo: tipo.to_string(),
            monto: monto.to_string(),
            categoria: cat.to_string(),
            fecha: fecha.to_string(),
            descripcion: "Movimiento de prueba".to_string(),
        }
    }

    #[test]
    fn registra_gasto_valido_y_guarda_evento() {
        let mut p = Prueba::nueva();

        let mov = p
            .registrar(entrada("cta-principal", "gasto", "15,50", "Alimentación", "2026-10-21"))
            .expect("Debe registrar el gasto correctamente");

        assert_eq!(mov.tipo, TipoMovimiento::Gasto);
        assert_eq!(mov.monto.cantidad, Decimal::new(1550, 2));

        let eventos = p.repo.listar_eventos().unwrap();
        assert_eq!(eventos.len(), 2); // 1 cuenta_creada + 1 movimiento_registrado

        let EventoLibro::MovimientoRegistrado { datos, .. } = &eventos[1] else {
            panic!("Se esperaba MovimientoRegistrado");
        };
        assert_eq!(datos.id_cuenta, "cta-principal");
        assert_eq!(datos.monto.cantidad, Decimal::new(1550, 2));
    }
    
    #[test]
    fn registra_ingreso_valido_y_guarda_evento() {
        let mut p = Prueba::nueva();

        let mov = p
            .registrar(entrada("cta-principal", "ingreso", "500,00", "Salario", "2026-10-21"))
            .expect("Debe registrar el ingreso correctamente");

        assert_eq!(mov.tipo, TipoMovimiento::Ingreso);
        assert_eq!(mov.monto.cantidad, Decimal::new(50000, 2));

        let eventos = p.repo.listar_eventos().unwrap();
        assert_eq!(eventos.len(), 2);
    }

    #[test]
    fn rechaza_si_la_cuenta_no_existe() {
        let mut p = Prueba::nueva();
        let err = p
            .registrar(entrada("cta-inexistente", "gasto", "10,00", "Alimentación", "2026-10-21"))
            .expect_err("Debe rechazar cuenta inexistente");

        let ErrorRegistrarMovimiento::Validacion(e) = err else {
            panic!("Se esperaba error de validación");
        };
        assert_eq!(e.campo, "id_cuenta");
    }

    #[test]
    fn rechaza_tipo_de_movimiento_desconocido() {
        let mut p = Prueba::nueva();
        let err = p
            .registrar(entrada("cta-principal", "tiponoexistente", "10,00", "Alimentación", "2026-10-21"))
            .expect_err("Debe rechazar tipo invalido");

        let ErrorRegistrarMovimiento::Validacion(e) = err else {
            panic!("Se esperaba error de validación");
        };
        assert_eq!(e.campo, "tipo");
    }

    #[test]
    fn propaga_error_de_validacion_del_dominio_por_monto_invalido() {
        let mut p = Prueba::nueva();

        // Enviamos un monto negativo
        let err = p
            .registrar(entrada("cta-principal", "gasto", "-15,50", "Alimentación", "2026-10-21"))
            .expect_err("El dominio debe rechazar monto negativo");

        let ErrorRegistrarMovimiento::Validacion(e) = err else {
            panic!("Se esperaba error de validación");
        };
        assert_eq!(e.campo, "monto");
        assert!(p.repo.listar_eventos().unwrap().len() == 1); // No guardó el evento
    }
}
