use std::str::FromStr;

use crate::aplicacion::puertos::{ErrorRepositorio, GeneradorId, Reloj, RepositorioLibro};
use crate::dominio::cuenta::{Cuenta, TipoCuenta};
use crate::dominio::errores::ErrorDominio;
use crate::dominio::evento::{DatosCuentaCreada, EventoLibro};
use crate::dominio::moneda::CodigoMoneda;
use crate::dominio::monto::Monto;
use serde::{Deserialize, Serialize};

const DISPOSITIVO_LOCAL: &str = "dispositivo-local";

// Entrada del usuario, incluye todos los campos que introduce para la nueva cuenta
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EntradaCrearCuenta {
    pub nombre: String,
    pub tipo: String,
    pub moneda: String,
    pub saldo_inicial: String,
}

#[derive(Debug)]
pub enum ErrorCrearCuenta {
    Validacion(ErrorDominio), // dato inválido o nombre repetido: lleva campo y mensaje
    Repositorio(ErrorRepositorio), // falla técnica: sin campo
}

impl From<ErrorDominio> for ErrorCrearCuenta {
    fn from(e: ErrorDominio) -> Self {
        ErrorCrearCuenta::Validacion(e)
    }
}

impl From<ErrorRepositorio> for ErrorCrearCuenta {
    fn from(e: ErrorRepositorio) -> Self {
        ErrorCrearCuenta::Repositorio(e)
    }
}

pub fn crear_cuenta(
    repo: &mut dyn RepositorioLibro,
    reloj: &dyn Reloj,
    ids: &mut dyn GeneradorId,
    entrada: EntradaCrearCuenta,
) -> Result<Cuenta, ErrorCrearCuenta> {
    // 1. Convertir los textos en tipos del dominio (el dominio valida)
    let moneda = CodigoMoneda::from_str(&entrada.moneda)?;
    let tipo = TipoCuenta::from_str(&entrada.tipo)?;
    let saldo = Monto::parse_saldo_inicial(&entrada.saldo_inicial, moneda)?;

    // 2. Nombre repetido (sin mayúsculas/minúsculas ni espacios extremos)
    let nombre_nuevo = entrada.nombre.trim().to_lowercase();
    for evento in repo.listar_eventos()? {
        if let EventoLibro::CuentaCreada { datos, .. } = evento {
            if datos.nombre.trim().to_lowercase() == nombre_nuevo {
                return Err(
                    ErrorDominio::nombre_invalido("Ya existe una cuenta con ese nombre.").into(),
                );
            }
        }
    }

    // 3. Crear la cuenta (el dominio valida el nombre vacío)
    let cuenta = Cuenta::nueva(ids.nuevo_id(), entrada.nombre, tipo, moneda, saldo)?;

    // 4. Emitir el evento y guardarlo
    let datos = DatosCuentaCreada {
        id_cuenta: cuenta.id.clone(),
        nombre: cuenta.nombre.clone(),
        tipo: cuenta.tipo,                   // Tiene derive Copy, se copia solo
        moneda: cuenta.moneda,               // Tiene derive Copy, se copia solo
        saldo_inicial: cuenta.saldo_inicial, // Tiene derive Copy, se copia solo
    };
    let evento =
        EventoLibro::nuevo_cuenta_creada(ids.nuevo_id(), reloj.ahora(), DISPOSITIVO_LOCAL, datos);
    repo.guardar(evento)?;

    Ok(cuenta)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infraestructura::memoria_repo::RepositorioLibroEnMemoria;
    use rust_decimal::Decimal;

    // ---------- Dobles de prueba (reloj e ids reproducibles) ----------

    struct RelojFijo;

    impl Reloj for RelojFijo {
        fn ahora(&self) -> String {
            "2026-10-14T20:00:00Z".to_string()
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

    // ---------- Ayudas para escribir los tests más cortos ----------

    struct Prueba {
        repo: RepositorioLibroEnMemoria,
        ids: IdsSecuenciales,
    }

    impl Prueba {
        fn nueva() -> Self {
            Prueba {
                repo: RepositorioLibroEnMemoria::nuevo(),
                ids: IdsSecuenciales { contador: 0 },
            }
        }

        fn crear(&mut self, entrada: EntradaCrearCuenta) -> Result<Cuenta, ErrorCrearCuenta> {
            crear_cuenta(&mut self.repo, &RelojFijo, &mut self.ids, entrada)
        }

        fn eventos(&self) -> Vec<EventoLibro> {
            self.repo.listar_eventos().unwrap()
        }
    }

    fn entrada(nombre: &str, tipo: &str, moneda: &str, saldo: &str) -> EntradaCrearCuenta {
        EntradaCrearCuenta {
            nombre: nombre.to_string(),
            tipo: tipo.to_string(),
            moneda: moneda.to_string(),
            saldo_inicial: saldo.to_string(),
        }
    }

    /// Devuelve el `campo` del error de validación; falla si el resultado fue otra cosa.
    fn campo_del_error(resultado: Result<Cuenta, ErrorCrearCuenta>) -> String {
        match resultado {
            Err(ErrorCrearCuenta::Validacion(e)) => e.campo,
            otro => panic!("se esperaba un error de validación, pero fue: {:?}", otro),
        }
    }

    // ---------- Caso feliz ----------

    #[test]
    fn crea_cuenta_valida_y_guarda_el_evento_cuenta_creada() {
        let mut p = Prueba::nueva();

        let cuenta = p
            .crear(entrada("Ahorros", "banco", "USD", "300,00"))
            .expect("la cuenta válida debe crearse");

        assert_eq!(cuenta.nombre, "Ahorros");
        assert_eq!(cuenta.tipo, TipoCuenta::Banco);
        assert_eq!(cuenta.moneda, CodigoMoneda::USD);
        assert_eq!(cuenta.saldo_inicial.cantidad, Decimal::new(30000, 2));

        let eventos = p.eventos();
        assert_eq!(eventos.len(), 1);
        let EventoLibro::CuentaCreada {
            tipo,
            momento_registro,
            dispositivo_origen,
            datos,
            ..
        } = &eventos[0]
        else {
            panic!("Se esperaba CuentaCreada");
        };
        assert_eq!(tipo, "cuenta_creada");
        assert_eq!(momento_registro, "2026-10-14T20:00:00Z");
        assert_eq!(dispositivo_origen, DISPOSITIVO_LOCAL);
        assert_eq!(datos.id_cuenta, cuenta.id);
        assert_eq!(datos.nombre, "Ahorros");
    }

    #[test]
    fn crea_cuentas_en_ves_usd_y_usdt() {
        // Criterio de aceptación #2
        let mut p = Prueba::nueva();

        assert!(p
            .crear(entrada("Efectivo Bs", "efectivo", "VES", "1250,50"))
            .is_ok());
        assert!(p
            .crear(entrada("Banco USD", "banco", "USD", "300,00"))
            .is_ok());
        assert!(p
            .crear(entrada(
                "Billetera USDT",
                "billetera_digital",
                "USDT",
                "150,00"
            ))
            .is_ok());

        assert_eq!(p.eventos().len(), 3);
    }

    #[test]
    fn acepta_saldo_inicial_cero() {
        let mut p = Prueba::nueva();
        assert!(p.crear(entrada("Vacía", "efectivo", "VES", "0")).is_ok());
    }

    // ---------- Validaciones: nombre ----------

    #[test]
    fn rechaza_nombre_vacio_indicando_el_campo_nombre() {
        let mut p = Prueba::nueva();
        let resultado = p.crear(entrada("", "banco", "USD", "10,00"));
        assert_eq!(campo_del_error(resultado), "nombre");
        assert!(p.eventos().is_empty());
    }

    #[test]
    fn rechaza_nombre_con_solo_espacios() {
        let mut p = Prueba::nueva();
        let resultado = p.crear(entrada("   ", "banco", "USD", "10,00"));
        assert_eq!(campo_del_error(resultado), "nombre");
        assert!(p.eventos().is_empty());
    }

    #[test]
    fn rechaza_nombre_repetido() {
        let mut p = Prueba::nueva();
        assert!(p.crear(entrada("Ahorros", "banco", "USD", "10,00")).is_ok());

        let resultado = p.crear(entrada("Ahorros", "efectivo", "VES", "5,00"));

        assert_eq!(campo_del_error(resultado), "nombre");
        assert_eq!(p.eventos().len(), 1); // la segunda no se guardó
    }

    #[test]
    fn rechaza_nombre_repetido_sin_distinguir_mayusculas_ni_espacios_extremos() {
        let mut p = Prueba::nueva();
        assert!(p.crear(entrada("Ahorros", "banco", "USD", "10,00")).is_ok());

        let resultado = p.crear(entrada("  AHORROS  ", "banco", "USD", "10,00"));

        assert_eq!(campo_del_error(resultado), "nombre");
        assert_eq!(p.eventos().len(), 1);
    }

    // ---------- Validaciones: moneda, tipo y saldo ----------

    #[test]
    fn rechaza_moneda_desconocida_indicando_el_campo_moneda() {
        let mut p = Prueba::nueva();
        let resultado = p.crear(entrada("Cripto", "billetera_digital", "BTC", "1,00"));
        assert_eq!(campo_del_error(resultado), "moneda");
        assert!(p.eventos().is_empty());
    }

    #[test]
    fn rechaza_tipo_invalido_indicando_el_campo_tipo() {
        let mut p = Prueba::nueva();
        let resultado = p.crear(entrada("Visa", "tarjeta_credito", "USD", "1,00"));
        assert_eq!(campo_del_error(resultado), "tipo");
        assert!(p.eventos().is_empty());
    }

    #[test]
    fn rechaza_saldo_negativo_indicando_el_campo_saldo_inicial() {
        let mut p = Prueba::nueva();
        let resultado = p.crear(entrada("Deuda", "banco", "USD", "-10,00"));
        assert_eq!(campo_del_error(resultado), "saldo_inicial");
        assert!(p.eventos().is_empty());
    }

    #[test]
    fn rechaza_saldo_con_mas_decimales_que_la_moneda() {
        // Criterio de aceptación #3: 10,555 USD
        let mut p = Prueba::nueva();
        let resultado = p.crear(entrada("Preciso", "banco", "USD", "10,555"));
        assert_eq!(campo_del_error(resultado), "saldo_inicial");
        assert!(p.eventos().is_empty());
    }

    #[test]
    fn rechaza_saldo_que_no_es_un_numero() {
        let mut p = Prueba::nueva();
        let resultado = p.crear(entrada("Raro", "banco", "USD", "abc"));
        assert_eq!(campo_del_error(resultado), "saldo_inicial");
        assert!(p.eventos().is_empty());
    }
}
