use std::path::PathBuf;

use crate::aplicacion::puertos::RepositorioLibro;
use crate::dominio::cuenta::TipoCuenta;
use crate::dominio::evento::{DatosCuentaCreada, EventoLibro};
use crate::dominio::moneda::CodigoMoneda;
use crate::dominio::monto::Monto;
use crate::infraestructura::memoria_repo::RepositorioLibroEnMemoria;
use crate::infraestructura::sqlite_repo::RepositorioLibroSqlite;

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

fn verificar_contrato(repo: &mut dyn RepositorioLibro) {
    let primero = crear_evento("evento-1", "cuenta-1");
    let segundo = crear_evento("evento-2", "cuenta-2");

    repo.guardar(primero.clone()).unwrap();
    repo.guardar(primero.clone()).unwrap();
    repo.guardar(segundo.clone()).unwrap();

    assert_eq!(repo.listar_eventos().unwrap(), vec![primero, segundo]);
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
    let evento = crear_evento("evento-persistente", "cuenta-persistente");

    {
        let mut repo = RepositorioLibroSqlite::abrir(&ruta).unwrap();
        repo.guardar(evento.clone()).unwrap();
    }

    let repo = RepositorioLibroSqlite::abrir(&ruta).unwrap();
    assert_eq!(repo.listar_eventos().unwrap(), vec![evento]);
    drop(repo);
    std::fs::remove_file(ruta).unwrap();
}

fn ruta_temporal(prefijo: &str) -> PathBuf {
    std::env::temp_dir().join(format!(
        "cuentas_claras_{prefijo}_{}.db",
        uuid::Uuid::new_v4()
    ))
}
