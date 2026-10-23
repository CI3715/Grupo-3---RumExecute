use crate::aplicacion::puertos::{ErrorRepositorio, GeneradorId, Reloj, RepositorioLibro};
use crate::dominio::cuenta::Cuenta;
use crate::dominio::errores::ErrorDominio;
use crate::dominio::evento::{DatosMovimientoCorregido, EventoLibro};
use crate::dominio::monto::Monto;
use crate::dominio::movimiento::Movimiento;
use serde::{Deserialize, Serialize};

const DISPOSITIVO_LOCAL: &str = "dispositivo-local";

/// Entrada del usuario para corregir un movimiento erróneo mediante un ajuste.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EntradaCorregirMovimiento {
    pub id_movimiento_original: String,
    pub nuevo_monto: String,
    pub fecha: String,
    pub descripcion: String,
}

#[derive(Debug)]
pub enum ErrorCorregirMovimiento {
    Validacion(ErrorDominio),
    Repositorio(ErrorRepositorio),
}

impl From<ErrorDominio> for ErrorCorregirMovimiento {
    fn from(e: ErrorDominio) -> Self {
        ErrorCorregirMovimiento::Validacion(e)
    }
}

impl From<ErrorRepositorio> for ErrorCorregirMovimiento {
    fn from(e: ErrorRepositorio) -> Self {
        ErrorCorregirMovimiento::Repositorio(e)
    }
}

pub fn corregir_movimiento(
    repo: &mut dyn RepositorioLibro,
    reloj: &dyn Reloj,
    ids: &mut dyn GeneradorId,
    entrada: EntradaCorregirMovimiento,
) -> Result<Movimiento, ErrorCorregirMovimiento> {
    let momento_actual_utc = reloj.ahora();

    // Reconstruir el estado leyendo los eventos del libro
    let mut movimiento_original_opt: Option<Movimiento> = None;
    let mut cuentas: Vec<Cuenta> = Vec::new();

    for evento in repo.listar_eventos()? {
        match evento {
            EventoLibro::CuentaCreada { datos, .. } => {
                cuentas.push(Cuenta {
                    id: datos.id_cuenta,
                    nombre: datos.nombre,
                    tipo: datos.tipo,
                    moneda: datos.moneda,
                    saldo_inicial: datos.saldo_inicial,
                });
            }
            EventoLibro::MovimientoRegistrado { datos, .. }
                if datos.id_movimiento == entrada.id_movimiento_original =>
            {
                // Reconstruimos la entidad del movimiento original
                movimiento_original_opt = Some(Movimiento {
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
            _ => {}
        }
    }

    // Verificar que el movimiento original existe
    let movimiento_original = movimiento_original_opt.ok_or_else(|| {
        ErrorDominio::new(
            "id_movimiento_original",
            format!(
                "No se encontró el movimiento original con ID '{}'.",
                entrada.id_movimiento_original
            ),
        )
    })?;

    // Obtener la cuenta asociada al movimiento para conocer su moneda
    let cuenta = cuentas
        .iter()
        .find(|c| c.id == movimiento_original.id_cuenta)
        .ok_or_else(|| {
            ErrorDominio::new(
                "id_cuenta",
                format!(
                    "No se encontró la cuenta asociada al movimiento con ID '{}'.",
                    movimiento_original.id_cuenta
                ),
            )
        })?;

    let monto_correcto = Monto::parse_movimiento(&entrada.nuevo_monto, cuenta.moneda)?;

    // El dominio calcula el ajuste compensatorio e instancia el Movimiento de tipo Ajuste (INV-04)
    let id_ajuste = ids.nuevo_id();
    let movimiento_ajuste = Movimiento::crear_ajuste_por_correccion(
        &id_ajuste,
        cuenta,
        &movimiento_original,
        &monto_correcto,
        &entrada.fecha,
        entrada.descripcion,
        &momento_actual_utc,
    )?;

    // Construir el evento de Event Sourcing (INV-04, INV-13)
    let datos_evento = DatosMovimientoCorregido {
        id_ajuste: movimiento_ajuste.id.clone(),
        id_cuenta: cuenta.id.clone(),
        id_movimiento_original: movimiento_original.id.clone(),
        monto_ajuste: movimiento_ajuste.monto,
        monto_original: movimiento_original.monto,
        monto_nuevo: monto_correcto,
        fecha: movimiento_ajuste.fecha.clone(),
        descripcion: movimiento_ajuste.descripcion.clone(),
    };

    let id_evento = ids.nuevo_id();
    let evento = EventoLibro::nuevo_movimiento_corregido(
        id_evento,
        momento_actual_utc,
        DISPOSITIVO_LOCAL,
        datos_evento,
    );

    repo.guardar(evento)?;

    Ok(movimiento_ajuste)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::aplicacion::registrar_movimiento::{
        registrar_movimiento, EntradaRegistrarMovimiento,
    };
    use crate::dominio::cuenta::TipoCuenta;
    use crate::dominio::evento::DatosCuentaCreada;
    use crate::dominio::moneda::CodigoMoneda;
    use crate::dominio::movimiento::TipoMovimiento;
    use crate::infraestructura::memoria_repo::RepositorioLibroEnMemoria;
    use rust_decimal::Decimal;

    struct RelojFijo;
    impl Reloj for RelojFijo {
        fn ahora(&self) -> String {
            "2026-10-21T14:00:00Z".to_string()
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

            // Crear una cuenta inicial
            let datos_cuenta = DatosCuentaCreada {
                id_cuenta: "cta-1".to_string(),
                nombre: "Banco USD".to_string(),
                tipo: TipoCuenta::Banco,
                moneda: CodigoMoneda::USD,
                saldo_inicial: Monto::cero(CodigoMoneda::USD),
            };
            repo.guardar(EventoLibro::nuevo_cuenta_creada(
                "evt-1",
                "2026-10-21T10:00:00Z",
                "disp-test",
                datos_cuenta,
            ))
            .unwrap();

            let mut ids = IdsSecuenciales { contador: 10 };

            // Registrar un gasto original por 500 USD (id_movimiento = "id-11")
            let entrada_gasto = EntradaRegistrarMovimiento {
                id_cuenta: "cta-1".to_string(),
                tipo: "gasto".to_string(),
                monto: "500,00".to_string(),
                categoria: "Alimentación".to_string(),
                fecha: "2026-10-21".to_string(),
                descripcion: "Cena".to_string(),
            };
            registrar_movimiento(&mut repo, &RelojFijo, &mut ids, entrada_gasto).unwrap();

            Prueba { repo, ids }
        }

        fn corregir(
            &mut self,
            entrada: EntradaCorregirMovimiento,
        ) -> Result<Movimiento, ErrorCorregirMovimiento> {
            corregir_movimiento(&mut self.repo, &RelojFijo, &mut self.ids, entrada)
        }
    }

    #[test]
    fn corrige_gasto_anotado_de_500_a_50_generando_ajuste_positivo_de_450() {
        let mut p = Prueba::nueva();

        let entrada = EntradaCorregirMovimiento {
            id_movimiento_original: "id-11".to_string(),
            nuevo_monto: "50,00".to_string(),
            fecha: "2026-10-21".to_string(),
            descripcion: "Corrección de gasto anotado por 500 a 50".to_string(),
        };

        let ajuste = p.corregir(entrada).expect("Debe crear el ajuste");

        assert_eq!(ajuste.tipo, TipoMovimiento::Ajuste);
        assert_eq!(ajuste.monto.cantidad, Decimal::new(45000, 2)); // +450.00
        assert_eq!(ajuste.id_movimiento_referenciado.as_deref(), Some("id-11"));

        // Se verifica la emisión del evento
        let eventos = p.repo.listar_eventos().unwrap();
        assert_eq!(eventos.len(), 3); // 1 CuentaCreada + 1 MovimientoRegistrado + 1 MovimientoCorregido

        let EventoLibro::MovimientoCorregido { datos, .. } = &eventos[2] else {
            panic!("Se esperaba MovimientoCorregido");
        };
        assert_eq!(datos.id_movimiento_original, "id-11");
        assert_eq!(datos.monto_ajuste.cantidad, Decimal::new(45000, 2));
    }

    #[test]
    fn rechaza_si_el_movimiento_original_no_existe() {
        let mut p = Prueba::nueva();

        let entrada = EntradaCorregirMovimiento {
            id_movimiento_original: "mov-inexistente".to_string(),
            nuevo_monto: "50,00".to_string(),
            fecha: "2026-10-21".to_string(),
            descripcion: "Error".to_string(),
        };

        let err = p
            .corregir(entrada)
            .expect_err("Debe rechazar id inexistente");

        let ErrorCorregirMovimiento::Validacion(e) = err else {
            panic!("Se esperaba error de validación");
        };
        assert_eq!(e.campo, "id_movimiento_original");
    }
}
