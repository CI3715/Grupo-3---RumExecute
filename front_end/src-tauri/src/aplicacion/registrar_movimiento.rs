use crate::aplicacion::puertos::{ErrorRepositorio, GeneradorId, Reloj, RepositorioLibro};
use crate::dominio::categoria::{catalogo_categorias, Categoria};
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

    // Reconstruir la cuenta afectada y el catálogo de categorías (fábrica + creadas)
    let mut cuenta_opt: Option<Cuenta> = None;
    let mut categorias_disponibles = catalogo_categorias();

    for evento in repo.listar_eventos()? {
        match evento {
            EventoLibro::CuentaCreada { datos, .. } => {
                if datos.id_cuenta == entrada.id_cuenta {
                    cuenta_opt = Some(Cuenta {
                        id: datos.id_cuenta,
                        nombre: datos.nombre,
                        tipo: datos.tipo,
                        moneda: datos.moneda,
                        saldo_inicial: datos.saldo_inicial,
                    });
                }
            }
            EventoLibro::CategoriaCreada { datos, .. } => {
                categorias_disponibles.push(Categoria {
                    id: datos.id_categoria,
                    nombre: datos.nombre,
                    tipo: datos.tipo,
                });
            }
            _ => {}
        }
    }

    let cuenta = cuenta_opt.ok_or_else(|| {
        ErrorDominio::new(
            "id_cuenta",
            format!(
                "No existe ninguna cuenta registrada con el id '{}'.",
                entrada.id_cuenta
            ),
        )
    })?;

    let monto = Monto::parse_movimiento(&entrada.monto, cuenta.moneda)?;

    // Crear el movimiento según el tipo validando contra el catálogo disponible
    let id_movimiento = ids.nuevo_id();

    let movimiento = match entrada.tipo.trim().to_lowercase().as_str() {
        "gasto" => Movimiento::nuevo_gasto_con_catalogo(
            &id_movimiento,
            &cuenta,
            monto,
            &entrada.categoria,
            &entrada.fecha,
            entrada.descripcion,
            &momento_actual_utc,
            &categorias_disponibles,
        )?,
        "ingreso" => Movimiento::nuevo_ingreso_con_catalogo(
            &id_movimiento,
            &cuenta,
            monto,
            &entrada.categoria,
            &entrada.fecha,
            entrada.descripcion,
            &momento_actual_utc,
            &categorias_disponibles,
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

        fn registrar(
            &mut self,
            entrada: EntradaRegistrarMovimiento,
        ) -> Result<Movimiento, ErrorRegistrarMovimiento> {
            registrar_movimiento(&mut self.repo, &RelojFijo, &mut self.ids, entrada)
        }
    }

    fn entrada(
        id_cuenta: &str,
        tipo: &str,
        monto: &str,
        cat: &str,
        fecha: &str,
    ) -> EntradaRegistrarMovimiento {
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
            .registrar(entrada(
                "cta-principal",
                "gasto",
                "15,50",
                "Alimentación",
                "2026-10-21",
            ))
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
            .registrar(entrada(
                "cta-principal",
                "ingreso",
                "500,00",
                "Salario",
                "2026-10-21",
            ))
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
            .registrar(entrada(
                "cta-inexistente",
                "gasto",
                "10,00",
                "Alimentación",
                "2026-10-21",
            ))
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
            .registrar(entrada(
                "cta-principal",
                "tiponoexistente",
                "10,00",
                "Alimentación",
                "2026-10-21",
            ))
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
            .registrar(entrada(
                "cta-principal",
                "gasto",
                "-15,50",
                "Alimentación",
                "2026-10-21",
            ))
            .expect_err("El dominio debe rechazar monto negativo");

        let ErrorRegistrarMovimiento::Validacion(e) = err else {
            panic!("Se esperaba error de validación");
        };
        assert_eq!(e.campo, "monto");
        assert!(p.repo.listar_eventos().unwrap().len() == 1); // No guardó el evento
    }

    #[test]
    fn rechaza_ingreso_con_fecha_futura_indicando_campo_fecha() {
        let mut p = Prueba::nueva();

        // RelojFijo es 2026-10-21T12:00:00Z, enviamos fecha 2026-10-22
        let err = p
            .registrar(entrada(
                "cta-principal",
                "ingreso",
                "100,00",
                "Salario",
                "2026-10-22",
            ))
            .expect_err("Debe rechazar ingreso con fecha futura");

        let ErrorRegistrarMovimiento::Validacion(e) = err else {
            panic!("Se esperaba error de validación");
        };
        assert_eq!(e.campo, "fecha");
        assert!(e.mensaje.contains("es futura"));
        assert_eq!(p.repo.listar_eventos().unwrap().len(), 1); // No persistió ningún evento
    }

    #[test]
    fn registra_movimiento_con_categoria_creada_por_usuario() {
        let mut p = Prueba::nueva();

        // 1. Guardamos una categoría creada por el usuario
        use crate::dominio::categoria::TipoCategoria;
        use crate::dominio::evento::DatosCategoriaCreada;
        p.repo
            .guardar(EventoLibro::nuevo_categoria_creada(
                "evt-cat",
                "2026-10-21T10:00:00Z",
                "disp",
                DatosCategoriaCreada {
                    id_categoria: "cursos_online".to_string(),
                    nombre: "Cursos Online".to_string(),
                    tipo: TipoCategoria::Gasto,
                },
            ))
            .unwrap();

        // 2. Registramos un gasto usando esa categoría creada
        let mov = p
            .registrar(entrada(
                "cta-principal",
                "gasto",
                "75,00",
                "Cursos Online",
                "2026-10-21",
            ))
            .expect("Debe registrar el movimiento con la categoría creada");

        assert_eq!(mov.categoria.unwrap().nombre, "Cursos Online");
        assert_eq!(p.repo.listar_eventos().unwrap().len(), 3); // cta + cat + mov

        // 3. Si intentamos registrar un ingreso con esa categoría de gasto, debe fallar
        let err = p
            .registrar(entrada(
                "cta-principal",
                "ingreso",
                "50,00",
                "Cursos Online",
                "2026-10-21",
            ))
            .expect_err("Debe rechazar categoría de gasto en un ingreso");

        let ErrorRegistrarMovimiento::Validacion(e) = err else {
            panic!("Se esperaba error de validación");
        };
        assert_eq!(e.campo, "categoria");
        assert!(e.mensaje.contains("no puede ser utilizada"));
    }
}
