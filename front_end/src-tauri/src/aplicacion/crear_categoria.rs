use crate::aplicacion::puertos::{ErrorRepositorio, GeneradorId, Reloj, RepositorioLibro};
use crate::dominio::categoria::{
    catalogo_categorias, validar_nueva_categoria_no_duplicada, Categoria, TipoCategoria,
};
use crate::dominio::errores::ErrorDominio;
use crate::dominio::evento::{DatosCategoriaCreada, EventoLibro};
use serde::{Deserialize, Serialize};

const DISPOSITIVO_LOCAL: &str = "dispositivo-local";

/// Entrada para crear una nueva categoría personalizada (RF-LIB-07).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EntradaCrearCategoria {
    pub nombre: String,
    pub tipo: String, // "gasto" o "ingreso"
}

#[derive(Debug)]
pub enum ErrorCrearCategoria {
    Validacion(ErrorDominio),
    Repositorio(ErrorRepositorio),
}

impl From<ErrorDominio> for ErrorCrearCategoria {
    fn from(e: ErrorDominio) -> Self {
        ErrorCrearCategoria::Validacion(e)
    }
}

impl From<ErrorRepositorio> for ErrorCrearCategoria {
    fn from(e: ErrorRepositorio) -> Self {
        ErrorCrearCategoria::Repositorio(e)
    }
}

/// Caso de uso: Crear una categoría nueva indicando nombre y tipo (RF-LIB-07).
///
/// Reglas de negocio:
/// - No puede haber dos categorías del mismo tipo con el mismo nombre (sin distinguir mayúsculas, espacios ni acentos).
/// - El nombre no puede estar vacío ni contener solo espacios.
/// - El tipo debe ser estrictamente "gasto" o "ingreso".
/// - Emite y guarda el evento inmutable `CategoriaCreada`.
pub fn crear_categoria(
    repo: &mut dyn RepositorioLibro,
    reloj: &dyn Reloj,
    ids: &mut dyn GeneradorId,
    entrada: EntradaCrearCategoria,
) -> Result<Categoria, ErrorCrearCategoria> {
    let tipo = match entrada.tipo.trim().to_lowercase().as_str() {
        "gasto" => TipoCategoria::Gasto,
        "ingreso" => TipoCategoria::Ingreso,
        _ => {
            return Err(ErrorDominio::tipo_invalido(format!(
                "El tipo de categoría '{}' no es válido. Opciones permitidas: 'gasto', 'ingreso'.",
                entrada.tipo
            ))
            .into());
        }
    };

    // Reconstruir todas las categorías existentes (de fábrica + previamente creadas)
    let mut categorias_existentes = catalogo_categorias();

    for evento in repo.listar_eventos()? {
        if let EventoLibro::CategoriaCreada { datos, .. } = evento {
            categorias_existentes.push(Categoria {
                id: datos.id_categoria,
                nombre: datos.nombre,
                tipo: datos.tipo,
            });
        }
    }

    // Validar que no exista otra categoría del mismo tipo con el mismo nombre
    validar_nueva_categoria_no_duplicada(&entrada.nombre, tipo, &categorias_existentes)?;

    let id_categoria = ids.nuevo_id();
    let categoria = Categoria::nueva(id_categoria.clone(), &entrada.nombre, tipo)?;

    let id_evento = ids.nuevo_id();
    let momento_actual = reloj.ahora();
    let datos_evento = DatosCategoriaCreada {
        id_categoria,
        nombre: categoria.nombre.clone(),
        tipo,
    };

    let evento = EventoLibro::nuevo_categoria_creada(
        id_evento,
        momento_actual,
        DISPOSITIVO_LOCAL,
        datos_evento,
    );

    repo.guardar(evento)?;

    Ok(categoria)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infraestructura::memoria_repo::RepositorioLibroEnMemoria;

    struct RelojFijo;
    impl Reloj for RelojFijo {
        fn ahora(&self) -> String {
            "2026-10-24T12:00:00Z".to_string()
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
            Prueba {
                repo: RepositorioLibroEnMemoria::nuevo(),
                ids: IdsSecuenciales { contador: 0 },
            }
        }

        fn crear(&mut self, nombre: &str, tipo: &str) -> Result<Categoria, ErrorCrearCategoria> {
            let entrada = EntradaCrearCategoria {
                nombre: nombre.to_string(),
                tipo: tipo.to_string(),
            };
            crear_categoria(&mut self.repo, &RelojFijo, &mut self.ids, entrada)
        }
    }

    #[test]
    fn crea_categoria_nueva_y_persiste_evento() {
        let mut p = Prueba::nueva();

        let cat = p
            .crear("Cursos Online", "gasto")
            .expect("Debe crear la categoría correctamente");

        assert_eq!(cat.nombre, "Cursos Online");
        assert_eq!(cat.tipo, TipoCategoria::Gasto);

        let eventos = p.repo.listar_eventos().unwrap();
        assert_eq!(eventos.len(), 1);

        let EventoLibro::CategoriaCreada { datos, .. } = &eventos[0] else {
            panic!("Se esperaba un evento CategoriaCreada");
        };
        assert_eq!(datos.nombre, "Cursos Online");
        assert_eq!(datos.tipo, TipoCategoria::Gasto);
    }

    #[test]
    fn rechaza_nombre_duplicado_del_mismo_tipo_sin_distinguir_mayusculas() {
        let mut p = Prueba::nueva();

        // 1. Ya existe en fábrica "Alimentación" de tipo Gasto
        let err_fabrica = p
            .crear("alimentacion", "gasto")
            .expect_err("Debe rechazar categoría duplicada existente en fábrica");

        let ErrorCrearCategoria::Validacion(e) = err_fabrica else {
            panic!("Se esperaba error de validación");
        };
        assert_eq!(e.campo, "categoria");
        assert!(e.mensaje.contains("Ya existe"));

        // 2. Crear una nueva y luego intentar crearla repetida
        p.crear("Freelance", "ingreso").unwrap();

        let err_repetida = p
            .crear("  freelance  ", "ingreso")
            .expect_err("Debe rechazar categoría duplicada creada previamente");

        let ErrorCrearCategoria::Validacion(e2) = err_repetida else {
            panic!("Se esperaba error de validación");
        };
        assert_eq!(e2.campo, "categoria");
        assert!(e2.mensaje.contains("Ya existe"));
    }

    #[test]
    fn permite_mismo_nombre_en_distintos_tipos() {
        let mut p = Prueba::nueva();

        // Crear "Comisiones" en Gasto y luego en Ingreso
        let g = p.crear("Comisiones", "gasto");
        let i = p.crear("Comisiones", "ingreso");

        assert!(g.is_ok());
        assert!(i.is_ok());
    }

    #[test]
    fn rechaza_nombre_vacio_o_tipo_invalido() {
        let mut p = Prueba::nueva();

        let err_vacio = p.crear("   ", "gasto").unwrap_err();
        let ErrorCrearCategoria::Validacion(e1) = err_vacio else {
            panic!("Se esperaba error de validación");
        };
        assert_eq!(e1.campo, "categoria");

        let err_tipo = p.crear("Válida", "invalido").unwrap_err();
        let ErrorCrearCategoria::Validacion(e2) = err_tipo else {
            panic!("Se esperaba error de validación");
        };
        assert_eq!(e2.campo, "tipo");
    }

    #[test]
    fn rechaza_nombre_duplicado_con_tildes_y_mayusculas() {
        let mut p = Prueba::nueva();

        // "Educación" existe de fábrica como Gasto
        let err = p.crear("EDUCACION", "gasto").unwrap_err();
        let ErrorCrearCategoria::Validacion(e) = err else {
            panic!("Se esperaba error de validación");
        };
        assert_eq!(e.campo, "categoria");
        assert!(e.mensaje.contains("Ya existe"));

        // Creamos "Música" como Gasto y luego intentamos "musica" y "MÚSICA"
        p.crear("Música", "gasto").unwrap();

        let err2 = p.crear("musica", "gasto").unwrap_err();
        let ErrorCrearCategoria::Validacion(e2) = err2 else {
            panic!("Se esperaba error de validación");
        };
        assert_eq!(e2.campo, "categoria");

        let err3 = p.crear("MÚSICA", "gasto").unwrap_err();
        let ErrorCrearCategoria::Validacion(e3) = err3 else {
            panic!("Se esperaba error de validación");
        };
        assert_eq!(e3.campo, "categoria");
    }

    #[test]
    fn permite_crear_categoria_con_nombre_igual_a_fabrica_pero_de_distinto_tipo() {
        let mut p = Prueba::nueva();

        // "Educación" es de fábrica en Gasto, pero el usuario la quiere como Ingreso (ej. da clases)
        let cat = p
            .crear("Educación", "ingreso")
            .expect("Debe permitir el mismo nombre en un tipo distinto");
        assert_eq!(cat.nombre, "Educación");
        assert_eq!(cat.tipo, TipoCategoria::Ingreso);
    }

    #[test]
    fn base_de_datos_sqlite_crea_las_categorias_y_las_guarda_persistentemente() {
        use crate::aplicacion::listar_categorias::listar_categorias;
        use crate::infraestructura::sqlite_repo::RepositorioLibroSqlite;

        let ruta = std::env::temp_dir().join(format!(
            "cuentas_claras_cat_sqlite_{}.db",
            uuid::Uuid::new_v4()
        ));

        // 1. Crear la categoría en SQLite
        {
            let mut repo = RepositorioLibroSqlite::abrir(&ruta).unwrap();
            let mut ids = IdsSecuenciales { contador: 0 };
            let entrada = EntradaCrearCategoria {
                nombre: "Gimnasio".to_string(),
                tipo: "gasto".to_string(),
            };
            let cat = crear_categoria(&mut repo, &RelojFijo, &mut ids, entrada)
                .expect("Debe guardar la categoría en SQLite");
            assert_eq!(cat.nombre, "Gimnasio");
            assert_eq!(cat.tipo, TipoCategoria::Gasto);
        }

        // 2. Cerrar la conexión y reabrir desde el archivo en disco
        {
            let repo = RepositorioLibroSqlite::abrir(&ruta).unwrap();
            let todas = listar_categorias(&repo).expect("Debe leer eventos de SQLite");

            // Debe contener las 11 originales de fábrica + la nueva "Gimnasio"
            assert_eq!(todas.len(), 12);
            let creada = todas.iter().find(|c| c.nombre == "Gimnasio").unwrap();
            assert_eq!(creada.tipo, TipoCategoria::Gasto);
        }

        // 3. Reabrir y verificar que no permita crear un duplicado
        {
            let mut repo = RepositorioLibroSqlite::abrir(&ruta).unwrap();
            let mut ids = IdsSecuenciales { contador: 10 };
            let entrada_repetida = EntradaCrearCategoria {
                nombre: "  GIMNASIO  ".to_string(),
                tipo: "gasto".to_string(),
            };
            let err = crear_categoria(&mut repo, &RelojFijo, &mut ids, entrada_repetida)
                .expect_err("Debe rechazar categoría duplicada en SQLite existente");
            let ErrorCrearCategoria::Validacion(e) = err else {
                panic!("Se esperaba error de validación");
            };
            assert_eq!(e.campo, "categoria");
            assert!(e.mensaje.contains("Ya existe"));
        }

        std::fs::remove_file(ruta).unwrap();
    }
}
