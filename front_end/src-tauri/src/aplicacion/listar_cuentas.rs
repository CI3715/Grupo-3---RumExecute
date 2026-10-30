use std::collections::HashSet;

use serde::Serialize;

use crate::aplicacion::puertos::{ErrorRepositorio, RepositorioLibro};
use crate::dominio::cuenta::Cuenta;
use crate::dominio::errores::ErrorDominio;
use crate::dominio::evento::EventoLibro;
use crate::dominio::monto::Monto;

/// Resultado de `listar_cuentas`: las cuentas y un subtotal por cada moneda (INV-02).
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ResumenCuentas {
    pub cuentas: Vec<Cuenta>,
    pub subtotales_por_moneda: Vec<Monto>,
}

#[derive(Debug)]
pub enum ErrorListarCuentas {
    Dominio(ErrorDominio),
    Repositorio(ErrorRepositorio),
}

impl From<ErrorDominio> for ErrorListarCuentas {
    fn from(e: ErrorDominio) -> Self {
        ErrorListarCuentas::Dominio(e)
    }
}

impl From<ErrorRepositorio> for ErrorListarCuentas {
    fn from(e: ErrorRepositorio) -> Self {
        ErrorListarCuentas::Repositorio(e)
    }
}

pub fn listar_cuentas(repo: &dyn RepositorioLibro) -> Result<ResumenCuentas, ErrorListarCuentas> {
    let mut vistos: HashSet<String> = HashSet::new();
    let mut cuentas: Vec<Cuenta> = Vec::new();
    let mut subtotales: Vec<Monto> = Vec::new();

    for evento in repo.listar_eventos()? {
        // INV-13: si este evento ya se aplicó, se ignora (aplicarlo dos veces = una vez)
        if !vistos.insert(evento.id().to_string()) {
            continue;
        }

        if let EventoLibro::CuentaCreada { datos, .. } = evento {
            let cuenta = Cuenta {
                id: datos.id_cuenta,
                nombre: datos.nombre,
                tipo: datos.tipo,
                moneda: datos.moneda,
                saldo_inicial: datos.saldo_inicial,
            };

            // INV-02: un subtotal por moneda; nunca se suman monedas distintas
            match subtotales.iter_mut().find(|s| s.moneda == cuenta.moneda) {
                Some(subtotal) => *subtotal = subtotal.sumar(&cuenta.saldo_actual())?,
                None => subtotales.push(cuenta.saldo_actual()),
            }

            cuentas.push(cuenta);
        }
    }

    Ok(ResumenCuentas {
        cuentas,
        subtotales_por_moneda: subtotales,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dominio::cuenta::TipoCuenta;
    use crate::dominio::evento::DatosCuentaCreada;
    use crate::dominio::moneda::CodigoMoneda;
    use crate::infraestructura::memoria_repo::RepositorioLibroEnMemoria;

    fn monto(texto: &str, moneda: CodigoMoneda, decimales: u32) -> Monto {
        Monto::parse_saldo_inicial(texto, moneda, decimales).unwrap()
    }

    fn evento(
        id_evento: &str,
        id_cuenta: &str,
        nombre: &str,
        moneda: CodigoMoneda,
        saldo: &str,
        decimales: u32,
    ) -> EventoLibro {
        let datos = DatosCuentaCreada {
            id_cuenta: id_cuenta.to_string(),
            nombre: nombre.to_string(),
            tipo: TipoCuenta::Banco,
            moneda,
            saldo_inicial: monto(saldo, moneda, decimales),
        };
        EventoLibro::nuevo_cuenta_creada(id_evento, "2026-10-14T20:00:00Z", "disp-test", datos)
    }

    #[test]
    fn sin_eventos_devuelve_listas_vacias() {
        let repo = RepositorioLibroEnMemoria::nuevo();
        let resumen = listar_cuentas(&repo).unwrap();
        assert!(resumen.cuentas.is_empty());
        assert!(resumen.subtotales_por_moneda.is_empty());
    }

    #[test]
    fn reconstruye_las_cuentas_en_orden_de_creacion() {
        let mut repo = RepositorioLibroEnMemoria::nuevo();
        repo.guardar(evento("e1", "c1", "Ahorros", CodigoMoneda::USD, "100,00", 2))
            .unwrap();
        repo.guardar(evento("e2", "c2", "Efectivo", CodigoMoneda::VES, "20,50", 2))
            .unwrap();

        let resumen = listar_cuentas(&repo).unwrap();

        assert_eq!(resumen.cuentas.len(), 2);
        assert_eq!(resumen.cuentas[0].id, "c1");
        assert_eq!(resumen.cuentas[0].nombre, "Ahorros");
        assert_eq!(resumen.cuentas[1].id, "c2");
        assert_eq!(resumen.cuentas[1].moneda, CodigoMoneda::VES);
    }

    #[test]
    fn subtotales_separados_por_moneda_sin_mezclarlas() {
        // INV-02 y Criterio de aceptación #4
        let mut repo = RepositorioLibroEnMemoria::nuevo();
        repo.guardar(evento("e1", "c1", "Ahorros", CodigoMoneda::USD, "100,00", 2))
            .unwrap();
        repo.guardar(evento("e2", "c2", "Caja", CodigoMoneda::USD, "50,00", 2))
            .unwrap();
        repo.guardar(evento("e3", "c3", "Efectivo", CodigoMoneda::VES, "20,00", 2))
            .unwrap();

        let resumen = listar_cuentas(&repo).unwrap();

        assert_eq!(resumen.subtotales_por_moneda.len(), 2);
        assert!(resumen
            .subtotales_por_moneda
            .contains(&monto("150,00", CodigoMoneda::USD, 2)));
        assert!(resumen
            .subtotales_por_moneda
            .contains(&monto("20,00", CodigoMoneda::VES, 2)));
    }

    #[test]
    fn inv13_aplicar_dos_veces_el_mismo_evento_equivale_a_aplicarlo_una() {
        let mut repo_una_vez = RepositorioLibroEnMemoria::nuevo();
        repo_una_vez
            .guardar(evento("e1", "c1", "Ahorros", CodigoMoneda::USD, "100,00", 2))
            .unwrap();

        let mut repo_dos_veces = RepositorioLibroEnMemoria::nuevo();
        repo_dos_veces
            .guardar(evento("e1", "c1", "Ahorros", CodigoMoneda::USD, "100,00", 2))
            .unwrap();
        repo_dos_veces
            .guardar(evento("e1", "c1", "Ahorros", CodigoMoneda::USD, "100,00", 2))
            .unwrap();

        let una = listar_cuentas(&repo_una_vez).unwrap();
        let dos = listar_cuentas(&repo_dos_veces).unwrap();

        assert_eq!(dos.cuentas.len(), 1);
        assert_eq!(una, dos); // mismo estado: la cuenta y el subtotal no se duplican
    }
}
