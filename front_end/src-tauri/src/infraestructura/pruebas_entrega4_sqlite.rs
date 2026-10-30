use std::cell::Cell;
use std::path::PathBuf;

use rusqlite::Connection;

use crate::aplicacion::cambiar_moneda_base::cambiar_moneda_base;
use crate::aplicacion::consultar_patrimonio::consultar_patrimonio;
use crate::aplicacion::crear_cuenta::{crear_cuenta, EntradaCrearCuenta};
use crate::aplicacion::crear_moneda::crear_moneda;
use crate::aplicacion::dto::*;
use crate::aplicacion::eliminar_tasa::eliminar_tasa;
use crate::aplicacion::puertos::*;
use crate::aplicacion::registrar_tasa::registrar_tasa;
use crate::dominio::evento::EventoLibro;
use crate::infraestructura::sqlite_repo::RepositorioLibroSqlite;

struct Reloj2026(Cell<&'static str>);
impl Reloj for Reloj2026 {
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

fn ruta_temporal(nombre: &str) -> PathBuf {
    let ruta = std::env::temp_dir().join(format!("cc_e4_{}_{}.db", nombre, std::process::id()));
    let _ = std::fs::remove_file(&ruta);
    ruta
}

fn patrimonio(repo: &RepositorioLibroSqlite, reloj: &Reloj2026, fecha: &str) -> PatrimonioDto {
    let e = ConsultarPatrimonioEntrada {
        fecha_consulta: Some(fecha.into()),
    };
    consultar_patrimonio(repo, reloj, e).unwrap()
}

/// Ejecuta un recorrido que emite los 4 eventos nuevos sobre SQLite real.
fn poblar(repo: &mut RepositorioLibroSqlite, reloj: &Reloj2026, ids: &mut Ids) -> String {
    for (n, t, m, s) in [
        ("Banesco", "banco", "VES", "1000,00"),
        ("Efectivo", "efectivo", "USD", "50,00"),
    ] {
        let e = EntradaCrearCuenta {
            nombre: n.into(),
            tipo: t.into(),
            moneda: m.into(),
            saldo_inicial: s.into(),
        };
        crear_cuenta(repo, reloj, ids, e).unwrap();
    }
    reloj.0.set("2026-12-01T12:00:00Z");

    // MonedaCreada
    crear_moneda(
        repo,
        reloj,
        ids,
        CrearMonedaDto {
            codigo: "CAD".into(),
            nombre: "Dólar Canadiense".into(),
            simbolo: "C$".into(),
            decimales: 2,
        },
    )
    .unwrap();

    // TasaRegistrada (la buena y una errónea que luego se elimina)
    registrar_tasa(
        repo,
        reloj,
        ids,
        RegistrarTasaDto {
            origen: "VES".into(),
            destino: "USD".into(),
            valor: "0.025".into(),
            fecha_vigencia: T1.into(),
            fuente: "Prueba".into(),
        },
    )
    .unwrap();
    let mala = registrar_tasa(
        repo,
        reloj,
        ids,
        RegistrarTasaDto {
            origen: "VES".into(),
            destino: "USD".into(),
            valor: "9.99".into(),
            fecha_vigencia: "2026-10-25".into(),
            fuente: "Error".into(),
        },
    )
    .unwrap();

    // TasaEliminada
    eliminar_tasa(
        repo,
        reloj,
        ids,
        EliminarTasaDto {
            id_tasa: mala.id_tasa.clone(),
        },
    )
    .unwrap();

    // MonedaBaseCambiada (ida y vuelta, la base final es USD)
    cambiar_moneda_base(
        repo,
        reloj,
        ids,
        CambiarMonedaBaseDto {
            nuevo_codigo: "EUR".into(),
        },
    )
    .unwrap();
    cambiar_moneda_base(
        repo,
        reloj,
        ids,
        CambiarMonedaBaseDto {
            nuevo_codigo: "USD".into(),
        },
    )
    .unwrap();

    mala.id_tasa
}

/// INV-13: los eventos nuevos se guardan y NO se pueden UPDATE ni DELETE.
#[test]
fn eventos_nuevos_son_inmutables_en_sqlite() {
    let ruta = ruta_temporal("inmutables");
    let reloj = Reloj2026(Cell::new("2026-10-01T12:00:00Z"));
    let mut ids = Ids(0);
    let mut repo = RepositorioLibroSqlite::abrir(&ruta).unwrap();
    poblar(&mut repo, &reloj, &mut ids);

    // Los 4 tipos nuevos realmente quedaron en el libro
    let eventos = repo.listar_eventos().unwrap();
    assert!(eventos
        .iter()
        .any(|e| matches!(e, EventoLibro::MonedaCreada { .. })));
    assert!(eventos
        .iter()
        .any(|e| matches!(e, EventoLibro::MonedaBaseCambiada { .. })));
    assert!(eventos
        .iter()
        .any(|e| matches!(e, EventoLibro::TasaRegistrada { .. })));
    assert!(eventos
        .iter()
        .any(|e| matches!(e, EventoLibro::TasaEliminada { .. })));

    // Segunda conexión al mismo archivo: los triggers deben rechazar todo
    let otra = Connection::open(&ruta).unwrap();
    let filas: Vec<(String, String)> = otra
        .prepare("SELECT id, tipo FROM eventos_libro")
        .unwrap()
        .query_map([], |f| Ok((f.get(0)?, f.get(1)?)))
        .unwrap()
        .map(|r| r.unwrap())
        .collect();
    assert_eq!(filas.len(), eventos.len());

    for (id, tipo) in &filas {
        assert!(
            otra.execute(
                "UPDATE eventos_libro SET evento_json = '{}' WHERE id = ?1",
                [id]
            )
            .is_err(),
            "UPDATE debe fallar sobre {tipo} ({id})"
        );
        assert!(
            otra.execute("DELETE FROM eventos_libro WHERE id = ?1", [id])
                .is_err(),
            "DELETE debe fallar sobre {tipo} ({id})"
        );
    }
    assert_eq!(
        repo.listar_eventos().unwrap(),
        eventos,
        "el libro no cambió"
    );

    drop(otra);
    drop(repo);
    let _ = std::fs::remove_file(&ruta);
}

/// Escenario 2.3 sobre SQLite real + persistencia al cerrar y reabrir la base.
#[test]
fn escenario_2_3_sobre_sqlite_persiste_al_reabrir() {
    let ruta = ruta_temporal("reabrir");
    let reloj = Reloj2026(Cell::new("2026-10-01T12:00:00Z"));
    let mut ids = Ids(0);

    let id_mala = {
        let mut repo = RepositorioLibroSqlite::abrir(&ruta).unwrap();
        let id = poblar(&mut repo, &reloj, &mut ids);
        let p = patrimonio(&repo, &reloj, T1);
        assert_eq!(p.moneda_base, "USD");
        assert_eq!(p.total_consolidado.as_deref(), Some("75.00"));
        id
    }; // repo se cierra aquí

    // Reabrir: todo se reconstruye por replay de eventos
    let repo = RepositorioLibroSqlite::abrir(&ruta).unwrap();
    let p = patrimonio(&repo, &reloj, T1);
    assert_eq!(p.moneda_base, "USD", "MonedaBaseCambiada se reconstruye");
    assert_eq!(
        p.total_consolidado.as_deref(),
        Some("75.00"),
        "TasaRegistrada se reconstruye"
    );
    assert!(p.tasa_faltante.is_none());

    // La tasa eliminada sigue eliminada tras reabrir (a 2026-10-25 aplica la de 0.025, no la de 9.99)
    let p = patrimonio(&repo, &reloj, "2026-10-25");
    assert_eq!(
        p.total_consolidado.as_deref(),
        Some("75.00"),
        "TasaEliminada se reconstruye ({id_mala})"
    );

    // La moneda propia sobrevive: se puede usar como base
    let mut repo = repo;
    cambiar_moneda_base(
        &mut repo,
        &reloj,
        &mut ids,
        CambiarMonedaBaseDto {
            nuevo_codigo: "CAD".into(),
        },
    )
    .expect("CAD debe existir tras reabrir");

    drop(repo);
    let _ = std::fs::remove_file(&ruta);
}
