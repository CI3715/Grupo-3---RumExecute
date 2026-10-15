use std::path::Path;

use rusqlite::{params, Connection};

use crate::aplicacion::puertos::{ErrorRepositorio, RepositorioLibro};
use crate::dominio::evento::EventoLibro;

pub struct RepositorioLibroSqlite {
    conexion: Connection,
}

impl RepositorioLibroSqlite {
    pub fn abrir(ruta: impl AsRef<Path>) -> Result<Self, ErrorRepositorio> {
        let conexion = Connection::open(ruta)
            .map_err(|error| ErrorRepositorio(format!("No se pudo abrir SQLite: {error}")))?;

        conexion
            .execute_batch(
                "
                PRAGMA foreign_keys = ON;
                CREATE TABLE IF NOT EXISTS eventos_libro (
                    secuencia INTEGER PRIMARY KEY AUTOINCREMENT,
                    id TEXT NOT NULL UNIQUE,
                    tipo TEXT NOT NULL,
                    momento_registro TEXT NOT NULL,
                    dispositivo_origen TEXT NOT NULL,
                    evento_json TEXT NOT NULL
                );
                CREATE TRIGGER IF NOT EXISTS eventos_libro_no_actualizar
                BEFORE UPDATE ON eventos_libro
                BEGIN
                    SELECT RAISE(ABORT, 'Los eventos del libro son inmutables');
                END;
                CREATE TRIGGER IF NOT EXISTS eventos_libro_no_eliminar
                BEFORE DELETE ON eventos_libro
                BEGIN
                    SELECT RAISE(ABORT, 'Los eventos del libro son inmutables');
                END;
                ",
            )
            .map_err(|error| {
                ErrorRepositorio(format!("No se pudo inicializar el libro SQLite: {error}"))
            })?;

        Ok(Self { conexion })
    }
}

impl RepositorioLibro for RepositorioLibroSqlite {
    fn guardar(&mut self, evento: EventoLibro) -> Result<(), ErrorRepositorio> {
        let evento_json = serde_json::to_string(&evento).map_err(|error| {
            ErrorRepositorio(format!("No se pudo serializar el evento: {error}"))
        })?;

        self.conexion
            .execute(
                "INSERT INTO eventos_libro
                    (id, tipo, momento_registro, dispositivo_origen, evento_json)
                 VALUES (?1, ?2, ?3, ?4, ?5)
                 ON CONFLICT(id) DO NOTHING",
                params![
                    evento.id(),
                    evento.tipo(),
                    evento.momento_registro(),
                    evento.dispositivo_origen(),
                    evento_json,
                ],
            )
            .map_err(|error| ErrorRepositorio(format!("No se pudo guardar el evento: {error}")))?;

        Ok(())
    }

    fn listar_eventos(&self) -> Result<Vec<EventoLibro>, ErrorRepositorio> {
        let mut consulta = self
            .conexion
            .prepare("SELECT evento_json FROM eventos_libro ORDER BY secuencia")
            .map_err(|error| {
                ErrorRepositorio(format!("No se pudieron consultar los eventos: {error}"))
            })?;

        let filas = consulta
            .query_map([], |fila| fila.get::<_, String>(0))
            .map_err(|error| {
                ErrorRepositorio(format!("No se pudieron consultar los eventos: {error}"))
            })?;

        filas
            .map(|fila| {
                let json = fila.map_err(|error| {
                    ErrorRepositorio(format!("No se pudo leer un evento SQLite: {error}"))
                })?;
                serde_json::from_str(&json).map_err(|error| {
                    ErrorRepositorio(format!("No se pudo deserializar un evento SQLite: {error}"))
                })
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dominio::cuenta::TipoCuenta;
    use crate::dominio::evento::DatosCuentaCreada;
    use crate::dominio::moneda::CodigoMoneda;
    use crate::dominio::monto::Monto;

    #[test]
    fn los_eventos_persistidos_no_se_pueden_actualizar_ni_eliminar() {
        let mut repo = RepositorioLibroSqlite::abrir(":memory:").unwrap();
        let moneda = CodigoMoneda::USD;
        repo.guardar(EventoLibro::nuevo_cuenta_creada(
            "evento-1",
            "2026-10-15T00:00:00.000Z",
            "dispositivo-prueba",
            DatosCuentaCreada {
                id_cuenta: "cuenta-1".to_string(),
                nombre: "Cuenta de prueba".to_string(),
                tipo: TipoCuenta::Banco,
                moneda,
                saldo_inicial: Monto::cero(moneda),
            },
        ))
        .unwrap();

        assert!(repo
            .conexion
            .execute("UPDATE eventos_libro SET evento_json = '{}'", [])
            .is_err());
        assert!(repo
            .conexion
            .execute("DELETE FROM eventos_libro", [])
            .is_err());
    }
}
