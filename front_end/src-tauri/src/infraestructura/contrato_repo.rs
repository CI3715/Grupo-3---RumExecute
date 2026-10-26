use std::path::PathBuf;

use crate::aplicacion::puertos::RepositorioLibro;
use crate::dominio::categoria::TipoCategoria;
use crate::dominio::cuenta::TipoCuenta;
use crate::dominio::evento::{
    DatosCategoriaCreada, DatosCuentaCreada, DatosMovimientoCorregido, DatosMovimientoRegistrado,
    EventoLibro,
};
use crate::dominio::moneda::CodigoMoneda;
use crate::dominio::monto::Monto;
use crate::dominio::movimiento::TipoMovimiento;
use crate::infraestructura::memoria_repo::RepositorioLibroEnMemoria;
use crate::infraestructura::sqlite_repo::RepositorioLibroSqlite;
use rust_decimal::Decimal;

fn crear_evento(id_evento: &str, id_cuenta: &str) -> EventoLibro {
    EventoLibro::nuevo_cuenta_creada(
        id_evento,
        "2026-10-15T00:00:00.000Z",
        "dispositivo-prueba",
        DatosCuentaCreada {
            id_cuenta: id_cuenta.to_string(),
            nombre: format!("Cuenta {id_cuenta}"),
            tipo: TipoCuenta::Banco,
            moneda: CodigoMoneda::USD,
            saldo_inicial: Monto::parse_saldo_inicial("10,00", CodigoMoneda::USD).unwrap(),
        },
    )
}

fn crear_evento_movimiento() -> EventoLibro {
    EventoLibro::nuevo_movimiento_registrado(
        "evento-movimiento",
        "2026-10-15T01:00:00.000Z",
        "dispositivo-prueba",
        DatosMovimientoRegistrado {
            id_movimiento: "movimiento-1".to_string(),
            id_cuenta: "cuenta-1".to_string(),
            tipo: TipoMovimiento::Gasto,
            monto: Monto::parse_movimiento("500,00", CodigoMoneda::USD).unwrap(),
            categoria: None,
            fecha: "2026-10-15".to_string(),
            descripcion: "Gasto de prueba".to_string(),
        },
    )
}

fn crear_evento_ajuste() -> EventoLibro {
    EventoLibro::nuevo_movimiento_corregido(
        "evento-ajuste",
        "2026-10-15T02:00:00.000Z",
        "dispositivo-prueba",
        DatosMovimientoCorregido {
            id_ajuste: "ajuste-1".to_string(),
            id_cuenta: "cuenta-1".to_string(),
            id_movimiento_original: "movimiento-1".to_string(),
            monto_ajuste: Monto::new(Decimal::new(45_000, 2), CodigoMoneda::USD),
            monto_original: Monto::parse_movimiento("500,00", CodigoMoneda::USD).unwrap(),
            monto_nuevo: Monto::parse_movimiento("50,00", CodigoMoneda::USD).unwrap(),
            fecha: "2026-10-15".to_string(),
            descripcion: "Corrección de prueba".to_string(),
        },
    )
}

fn crear_evento_categoria() -> EventoLibro {
    EventoLibro::nuevo_categoria_creada(
        "evento-categoria",
        "2026-10-15T03:00:00.000Z",
        "dispositivo-prueba",
        DatosCategoriaCreada {
            id_categoria: "suscripciones".to_string(),
            nombre: "Suscripciones".to_string(),
            tipo: TipoCategoria::Gasto,
        },
    )
}

fn verificar_contrato(repo: &mut dyn RepositorioLibro) {
    let primero = crear_evento("evento-1", "cuenta-1");
    let segundo = crear_evento("evento-2", "cuenta-2");
    let movimiento = crear_evento_movimiento();
    let ajuste = crear_evento_ajuste();
    let categoria = crear_evento_categoria();

    repo.guardar(primero.clone()).unwrap();
    repo.guardar(primero.clone()).unwrap();
    repo.guardar(segundo.clone()).unwrap();
    repo.guardar(movimiento.clone()).unwrap();
    repo.guardar(ajuste.clone()).unwrap();
    repo.guardar(categoria.clone()).unwrap();

    assert_eq!(
        repo.listar_eventos().unwrap(),
        vec![primero, segundo, movimiento, ajuste, categoria]
    );
}

#[test]
fn repositorio_en_memoria_cumple_el_contrato_de_eventos() {
    verificar_contrato(&mut RepositorioLibroEnMemoria::nuevo());
}

#[test]
fn repositorio_sqlite_cumple_el_mismo_contrato_de_eventos() {
    let ruta = ruta_temporal("contrato");
    let mut repo = RepositorioLibroSqlite::abrir(&ruta).unwrap();
    verificar_contrato(&mut repo);
    drop(repo);
    std::fs::remove_file(ruta).unwrap();
}

#[test]
fn repositorio_sqlite_conserva_eventos_al_reabrir_la_base() {
    let ruta = ruta_temporal("persistencia");
    let eventos = vec![
        crear_evento("evento-persistente", "cuenta-1"),
        crear_evento_movimiento(),
        crear_evento_ajuste(),
        crear_evento_categoria(),
    ];

    {
        let mut repo = RepositorioLibroSqlite::abrir(&ruta).unwrap();
        for evento in &eventos {
            repo.guardar(evento.clone()).unwrap();
        }
    }

    let repo = RepositorioLibroSqlite::abrir(&ruta).unwrap();
    assert_eq!(repo.listar_eventos().unwrap(), eventos);
    drop(repo);
    std::fs::remove_file(ruta).unwrap();
}

fn ruta_temporal(prefijo: &str) -> PathBuf {
    std::env::temp_dir().join(format!(
        "cuentas_claras_{prefijo}_{}.db",
        uuid::Uuid::new_v4()
    ))
}
