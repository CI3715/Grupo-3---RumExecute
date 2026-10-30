use super::cambiar_moneda_base::cambiar_moneda_base;
use super::consultar_patrimonio::consultar_patrimonio;
use super::crear_cuenta::{crear_cuenta, EntradaCrearCuenta};
use super::crear_moneda::{crear_moneda, ErrorCrearMoneda};
use super::dto::*;
use super::eliminar_tasa::eliminar_tasa;
use super::puertos::*;
use super::registrar_tasa::registrar_tasa;
use crate::dominio::evento::EventoLibro;
use crate::infraestructura::memoria_repo::RepositorioLibroEnMemoria;
use std::cell::Cell;

struct RelojAjustable(Cell<&'static str>);
impl Reloj for RelojAjustable {
    fn ahora(&self) -> String {
        self.0.get().to_string()
    }
}

struct Ids(u32);
impl GeneradorId for Ids {
    fn nuevo_id(&mut self) -> String {
        self.0 += 1;
        format!("id-{}", self.0)
    }
}

const T1: &str = "2026-10-20";
const T2: &str = "2026-11-10";

struct Mundo {
    repo: RepositorioLibroEnMemoria,
    ids: Ids,
    reloj: RelojAjustable,
}

impl Mundo {
    /// Reloj en 2026-10-01: las cuentas nacen antes de t1.
    fn nuevo() -> Self {
        Mundo {
            repo: RepositorioLibroEnMemoria::nuevo(),
            ids: Ids(0),
            reloj: RelojAjustable(Cell::new("2026-10-01T12:00:00Z")),
        }
    }

    fn hoy_es_diciembre(&self) {
        self.reloj.0.set("2026-12-01T12:00:00Z");
    }

    fn cuenta(&mut self, nombre: &str, tipo: &str, moneda: &str, saldo: &str) {
        let e = EntradaCrearCuenta {
            nombre: nombre.to_string(),
            tipo: tipo.to_string(),
            moneda: moneda.to_string(),
            saldo_inicial: saldo.to_string(),
        };
        crear_cuenta(&mut self.repo, &self.reloj, &mut self.ids, e).unwrap();
    }

    fn tasa(
        &mut self,
        o: &str,
        d: &str,
        v: &str,
        f: &str,
    ) -> Result<ResultadoTasaDto, super::registrar_tasa::ErrorRegistrarTasa> {
        let e = RegistrarTasaDto {
            origen: o.into(),
            destino: d.into(),
            valor: v.into(),
            fecha_vigencia: f.into(),
            fuente: "Prueba".into(),
        };
        registrar_tasa(&mut self.repo, &self.reloj, &mut self.ids, e)
    }

    fn base(&mut self, codigo: &str) {
        let e = CambiarMonedaBaseDto {
            nuevo_codigo: codigo.into(),
        };
        cambiar_moneda_base(&mut self.repo, &self.reloj, &mut self.ids, e).unwrap();
    }

    fn patrimonio(&self, fecha: &str) -> PatrimonioDto {
        let e = ConsultarPatrimonioEntrada {
            fecha_consulta: Some(fecha.into()),
        };
        consultar_patrimonio(&self.repo, &self.reloj, e).unwrap()
    }
}

/// Escenario oficial de la Sección 2.3 (RF-CAM-03, RF-CAM-04)
#[test]
fn escenario_2_3_completo() {
    let mut m = Mundo::nuevo();
    m.cuenta("Banesco", "banco", "VES", "1000,00");
    m.cuenta("Efectivo", "efectivo", "USD", "50,00");
    m.hoy_es_diciembre();

    // Sin tasa en t1: cuentas y subtotales sí, consolidado no, alerta con par y fecha
    let p = m.patrimonio(T1);
    assert_eq!(p.moneda_base, "USD");
    assert_eq!(p.cuentas.len(), 2);
    assert_eq!(p.subtotales.len(), 2);
    assert_eq!(p.total_consolidado, None);
    let f = p.tasa_faltante.expect("debe haber alerta");
    assert_eq!(f.moneda_origen, "VES");
    assert_eq!(f.moneda_destino, "USD");
    assert_eq!(f.fecha_requerida, T1);

    // Tasa t1 = 0.025 -> 1000*0.025 + 50 = 75.00
    m.tasa("VES", "USD", "0.025", T1).unwrap();
    let p = m.patrimonio(T1);
    assert_eq!(p.total_consolidado.as_deref(), Some("75.00"));
    assert!(p.tasa_faltante.is_none());

    // Tasa t2 = 0.020 -> a t1 sigue 0.025; a t2 aplica 0.020 (20 + 50 = 70)
    m.tasa("VES", "USD", "0.020", T2).unwrap();
    assert_eq!(m.patrimonio(T1).total_consolidado.as_deref(), Some("75.00"));
    assert_eq!(m.patrimonio(T2).total_consolidado.as_deref(), Some("70.00"));
}

/// RF-CAM-04: cuenta en EUR sin tasa
#[test]
fn rf_cam_04_cuenta_eur_sin_tasa_reporta_par_y_fecha() {
    let mut m = Mundo::nuevo();
    m.cuenta("Banesco", "banco", "VES", "1000,00");
    m.cuenta("Efectivo", "efectivo", "USD", "50,00");
    m.cuenta("Euros", "efectivo", "EUR", "10,00");
    m.hoy_es_diciembre();
    m.tasa("VES", "USD", "0.025", T1).unwrap();

    let p = m.patrimonio(T1);
    assert_eq!(p.cuentas.len(), 3, "no se omite ninguna cuenta");
    assert_eq!(p.total_consolidado, None);
    let f = p.tasa_faltante.unwrap();
    assert_eq!(f.moneda_origen, "EUR");
    assert_eq!(f.moneda_destino, "USD");
    assert_eq!(f.fecha_requerida, T1);
    assert!(f.mensaje.contains("EUR"));
}

/// RF-CAM-03: inversa 1/valor sin registrar USD->VES
#[test]
fn tasa_inversa_al_cambiar_base_a_ves() {
    let mut m = Mundo::nuevo();
    m.cuenta("Banesco", "banco", "VES", "1000,00");
    m.cuenta("Efectivo", "efectivo", "USD", "50,00");
    m.hoy_es_diciembre();
    m.tasa("VES", "USD", "0.025", T1).unwrap();
    m.base("VES");

    let p = m.patrimonio(T1);
    assert_eq!(p.moneda_base, "VES");
    // 50 USD * (1/0.025 = 40) + 1000 = 3000.00
    assert_eq!(p.total_consolidado.as_deref(), Some("3000.00"));
}

/// Sin triangulación: VES->USD y EUR->USD no implican VES->EUR
#[test]
fn no_hay_conversion_triangular() {
    let mut m = Mundo::nuevo();
    m.cuenta("Banesco", "banco", "VES", "1000,00");
    m.cuenta("Euros", "efectivo", "EUR", "10,00");
    m.hoy_es_diciembre();
    m.tasa("VES", "USD", "0.025", T1).unwrap();
    m.tasa("EUR", "USD", "1.10", T1).unwrap();
    m.base("EUR");

    let p = m.patrimonio(T1);
    assert_eq!(p.total_consolidado, None);
    let f = p.tasa_faltante.unwrap();
    assert_eq!(f.moneda_origen, "VES");
    assert_eq!(f.moneda_destino, "EUR");
}

/// RF-CAM-02: cambiar la base no muta saldos originales
#[test]
fn cambiar_base_no_muta_saldos_originales() {
    let mut m = Mundo::nuevo();
    m.cuenta("Banesco", "banco", "VES", "1000,00");
    m.cuenta("Efectivo", "efectivo", "USD", "50,00");
    m.hoy_es_diciembre();
    let antes: Vec<_> = m
        .patrimonio(T1)
        .cuentas
        .iter()
        .map(|c| {
            (
                c.id.clone(),
                c.saldo_original.clone(),
                c.moneda_original.clone(),
            )
        })
        .collect();
    m.base("EUR");
    let despues: Vec<_> = m
        .patrimonio(T1)
        .cuentas
        .iter()
        .map(|c| {
            (
                c.id.clone(),
                c.saldo_original.clone(),
                c.moneda_original.clone(),
            )
        })
        .collect();
    assert_eq!(antes, despues);
}

/// RF-CAM-06: tasa eliminada deja de aplicar
#[test]
fn tasa_eliminada_ya_no_aplica() {
    let mut m = Mundo::nuevo();
    m.cuenta("Banesco", "banco", "VES", "1000,00");
    m.hoy_es_diciembre();
    let id = m.tasa("VES", "USD", "0.025", T1).unwrap().id_tasa;
    assert_eq!(m.patrimonio(T1).total_consolidado.as_deref(), Some("25.00"));

    eliminar_tasa(
        &mut m.repo,
        &m.reloj,
        &mut m.ids,
        EliminarTasaDto { id_tasa: id },
    )
    .unwrap();
    let p = m.patrimonio(T1);
    assert_eq!(p.total_consolidado, None);
    assert!(p.tasa_faltante.is_some());
}

/// Validaciones de RegistrarTasa
#[test]
fn rechaza_tasas_invalidas() {
    let mut m = Mundo::nuevo();
    m.hoy_es_diciembre();
    assert!(
        m.tasa("VES", "USD", "0.025", "2027-01-01").is_err(),
        "fecha futura"
    );
    assert!(m.tasa("VES", "USD", "0", T1).is_err(), "valor cero");
    assert!(m.tasa("VES", "USD", "-1", T1).is_err(), "valor negativo");
    assert!(m.tasa("USD", "USD", "1", T1).is_err(), "monedas iguales");
    assert!(
        m.tasa("VES", "USD", "0.123456789", T1).is_err(),
        "más de 8 decimales"
    );
}

// ---------- crear_moneda (RF-CAM-01) ----------

impl Mundo {
    fn moneda(&mut self, codigo: &str, nombre: &str, simbolo: &str, decimales: u32) -> bool {
        let e = CrearMonedaDto {
            codigo: codigo.into(),
            nombre: nombre.into(),
            simbolo: simbolo.into(),
            decimales,
        };
        match crear_moneda(&mut self.repo, &self.reloj, &mut self.ids, e) {
            Ok(_) => true,
            Err(ErrorCrearMoneda::Validacion(_)) => false,
            Err(ErrorCrearMoneda::Repositorio(_)) => panic!("error de repositorio inesperado"),
        }
    }

    fn n_eventos(&self) -> usize {
        self.repo.listar_eventos().unwrap().len()
    }
}

#[test]
fn crear_moneda_valida_emite_evento_moneda_creada() {
    let mut m = Mundo::nuevo();
    let antes = m.n_eventos();
    assert!(m.moneda("CAD", "Dólar Canadiense", "C$", 2));
    let eventos = m.repo.listar_eventos().unwrap();
    assert_eq!(eventos.len(), antes + 1);
    assert!(matches!(
        eventos.last().unwrap(),
        EventoLibro::MonedaCreada { .. }
    ));
}

#[test]
fn crear_moneda_acepta_decimales_en_los_limites_0_y_8() {
    let mut m = Mundo::nuevo();
    assert!(m.moneda("CER", "Cero decimales", "C", 0));
    assert!(m.moneda("OCH", "Ocho decimales", "O", 8));
}

#[test]
fn crear_moneda_rechaza_codigo_duplicado() {
    let mut m = Mundo::nuevo();
    assert!(m.moneda("CAD", "Dólar Canadiense", "C$", 2));
    let antes = m.n_eventos();
    assert!(!m.moneda("CAD", "Otro nombre", "X", 2));
    assert_eq!(m.n_eventos(), antes, "un rechazo no emite eventos");
}

#[test]
fn crear_moneda_no_permite_pisar_monedas_de_fabrica() {
    let mut m = Mundo::nuevo();
    let antes = m.n_eventos();
    for codigo in ["VES", "USD", "EUR", "USDT"] {
        assert!(
            !m.moneda(codigo, "Intento", "X", 2),
            "{codigo} es de fábrica"
        );
    }
    assert_eq!(m.n_eventos(), antes);
}

#[test]
fn crear_moneda_rechaza_codigos_invalidos() {
    let mut m = Mundo::nuevo();
    for codigo in ["cad", "AB", "ABCDEF", "C4D", "CÁD", "", "CA D"] {
        assert!(
            !m.moneda(codigo, "Nombre", "X", 2),
            "código inválido: {codigo:?}"
        );
    }
}

#[test]
fn crear_moneda_rechaza_nombre_simbolo_y_decimales_invalidos() {
    let mut m = Mundo::nuevo();
    assert!(!m.moneda("AAA", "", "A", 2), "nombre vacío");
    assert!(!m.moneda("AAB", &"N".repeat(51), "A", 2), "nombre > 50");
    assert!(!m.moneda("AAC", "Nombre", "", 2), "símbolo vacío");
    assert!(!m.moneda("AAD", "Nombre", "ABCDEF", 2), "símbolo > 5");
    assert!(!m.moneda("AAE", "Nombre", "A", 9), "decimales > 8");
    assert_eq!(m.n_eventos(), 0, "ningún rechazo emite eventos");
}

#[test]
fn moneda_propia_puede_usarse_como_base() {
    let mut m = Mundo::nuevo();
    assert!(m.moneda("CAD", "Dólar Canadiense", "C$", 2));
    m.base("CAD"); // hace unwrap: falla si CAD no se reconoce
}
