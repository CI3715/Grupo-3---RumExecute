use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fmt;
use std::str::FromStr;

use super::errores::ErrorDominio;

/// Tipo de movimiento o categoría según la especificación (RF-LIB-07).
/// Las categorías están divididas estrictamente entre Gasto e Ingreso.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TipoCategoria {
    Gasto,
    Ingreso,
}

/// Alias para compatibilidad conceptual con movimientos contables.
pub type TipoMovimiento = TipoCategoria;

impl TipoCategoria {
    pub fn as_str(&self) -> &'static str {
        match self {
            TipoCategoria::Gasto => "gasto",
            TipoCategoria::Ingreso => "ingreso",
        }
    }
}

impl fmt::Display for TipoCategoria {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl FromStr for TipoCategoria {
    type Err = ErrorDominio;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_lowercase().as_str() {
            "gasto" => Ok(TipoCategoria::Gasto),
            "ingreso" => Ok(TipoCategoria::Ingreso),
            _ => Err(ErrorDominio::tipo_invalido(format!(
                "El tipo de categoría '{}' no es válido. Opciones permitidas: 'gasto', 'ingreso'.",
                s
            ))),
        }
    }
}

/// Entidad Categoría del catálogo de fábrica o personalizadas (RF-LIB-07).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Categoria {
    pub id: String,
    pub nombre: String,
    pub tipo: TipoCategoria,
}

impl Categoria {
    /// Crea una nueva categoría validando que el nombre no esté vacío.
    pub fn nueva(
        id: impl Into<String>,
        nombre: impl Into<String>,
        tipo: TipoCategoria,
    ) -> Result<Self, ErrorDominio> {
        let id = id.into().trim().to_string();
        let nombre = nombre.into().trim().to_string();

        if nombre.is_empty() {
            return Err(ErrorDominio::categoria_invalida(
                "El nombre de la categoría no puede estar vacío ni contener solo espacios.",
            ));
        }

        let final_id = if id.is_empty() {
            slug_de_texto(&nombre)
        } else {
            id
        };

        Ok(Self {
            id: final_id,
            nombre,
            tipo,
        })
    }
}

/// Normaliza una cadena eliminando espacios extras, convirtiendo a minúsculas
/// y sustituyendo tildes comunes para comparaciones insensibles a variantes ortográficas.
pub fn normalizar_para_comparacion(texto: &str) -> String {
    texto
        .trim()
        .to_lowercase()
        .chars()
        .map(|c| match c {
            'á' | 'à' | 'ä' | 'â' => 'a',
            'é' | 'è' | 'ë' | 'ê' => 'e',
            'í' | 'ì' | 'ï' | 'î' => 'i',
            'ó' | 'ò' | 'ö' | 'ô' => 'o',
            'ú' | 'ù' | 'ü' | 'û' => 'u',
            _ => c,
        })
        .collect()
}

/// Genera un identificador slug simple a partir de un nombre.
fn slug_de_texto(texto: &str) -> String {
    normalizar_para_comparacion(texto)
        .replace(' ', "_")
        .chars()
        .filter(|c| c.is_alphanumeric() || *c == '_')
        .collect()
}

/// Catálogo oficial de categorías de fábrica (RF-LIB-07):
///
/// Gasto:
/// - Alimentación
/// - Transporte
/// - Servicios
/// - Salud
/// - Educación
/// - Entretenimiento
/// - Otros gastos
///
/// Ingreso:
/// - Salario
/// - Ventas
/// - Regalos
/// - Otros ingresos
pub fn catalogo_categorias() -> Vec<Categoria> {
    vec![
        // Gastos
        Categoria {
            id: "alimentacion".to_string(),
            nombre: "Alimentación".to_string(),
            tipo: TipoCategoria::Gasto,
        },
        Categoria {
            id: "transporte".to_string(),
            nombre: "Transporte".to_string(),
            tipo: TipoCategoria::Gasto,
        },
        Categoria {
            id: "servicios".to_string(),
            nombre: "Servicios".to_string(),
            tipo: TipoCategoria::Gasto,
        },
        Categoria {
            id: "salud".to_string(),
            nombre: "Salud".to_string(),
            tipo: TipoCategoria::Gasto,
        },
        Categoria {
            id: "educacion".to_string(),
            nombre: "Educación".to_string(),
            tipo: TipoCategoria::Gasto,
        },
        Categoria {
            id: "entretenimiento".to_string(),
            nombre: "Entretenimiento".to_string(),
            tipo: TipoCategoria::Gasto,
        },
        Categoria {
            id: "otros_gastos".to_string(),
            nombre: "Otros gastos".to_string(),
            tipo: TipoCategoria::Gasto,
        },
        // Ingresos
        Categoria {
            id: "salario".to_string(),
            nombre: "Salario".to_string(),
            tipo: TipoCategoria::Ingreso,
        },
        Categoria {
            id: "ventas".to_string(),
            nombre: "Ventas".to_string(),
            tipo: TipoCategoria::Ingreso,
        },
        Categoria {
            id: "regalos".to_string(),
            nombre: "Regalos".to_string(),
            tipo: TipoCategoria::Ingreso,
        },
        Categoria {
            id: "otros_ingresos".to_string(),
            nombre: "Otros ingresos".to_string(),
            tipo: TipoCategoria::Ingreso,
        },
    ]
}

/// Retorna únicamente las categorías de gasto del catálogo de fábrica.
pub fn catalogo_gastos() -> Vec<Categoria> {
    catalogo_por_tipo(TipoCategoria::Gasto)
}

/// Retorna únicamente las categorías de ingreso del catálogo de fábrica.
pub fn catalogo_ingresos() -> Vec<Categoria> {
    catalogo_por_tipo(TipoCategoria::Ingreso)
}

/// Retorna las categorías del catálogo oficial filtradas por tipo.
pub fn catalogo_por_tipo(tipo: TipoCategoria) -> Vec<Categoria> {
    catalogo_categorias()
        .into_iter()
        .filter(|c| c.tipo == tipo)
        .collect()
}

/// Busca una categoría en un catálogo dado por id, nombre exacto o nombre normalizado
/// (sin distinguir mayúsculas, minúsculas ni acentos).
///
/// Si no existe, devuelve un `ErrorDominio` con `campo: "categoria"`.
pub fn buscar_categoria_en(
    identificador: &str,
    catalogo: &[Categoria],
) -> Result<Categoria, ErrorDominio> {
    let clean = identificador.trim();
    if clean.is_empty() {
        return Err(ErrorDominio::categoria_invalida(
            "El nombre o identificador de la categoría no puede estar vacío.",
        ));
    }

    let clean_norm = normalizar_para_comparacion(clean);

    catalogo
        .iter()
        .find(|c| {
            c.id.eq_ignore_ascii_case(clean)
                || c.nombre.trim().eq_ignore_ascii_case(clean)
                || normalizar_para_comparacion(&c.id) == clean_norm
                || normalizar_para_comparacion(&c.nombre) == clean_norm
        })
        .cloned()
        .ok_or_else(|| {
            ErrorDominio::categoria_invalida(format!(
                "La categoría '{}' no pertenece al catálogo oficial de categorías.",
                clean
            ))
        })
}

/// Valida que la categoría exista en el catálogo provisto y que coincida exactamente
/// con el tipo de movimiento esperado (Gasto o Ingreso).
pub fn validar_categoria_para_tipo_en(
    identificador: &str,
    tipo_esperado: TipoCategoria,
    catalogo: &[Categoria],
) -> Result<Categoria, ErrorDominio> {
    let categoria = buscar_categoria_en(identificador, catalogo)?;

    if categoria.tipo != tipo_esperado {
        return Err(ErrorDominio::categoria_invalida(format!(
            "La categoría '{}' corresponde al tipo '{}' y no puede ser utilizada para un movimiento de tipo '{}'.",
            categoria.nombre, categoria.tipo, tipo_esperado
        )));
    }

    Ok(categoria)
}

/// Busca una categoría en el catálogo de fábrica por id, nombre exacto o nombre normalizado.
pub fn buscar_categoria(identificador: &str) -> Result<Categoria, ErrorDominio> {
    buscar_categoria_en(identificador, &catalogo_categorias())
}

/// Valida que la categoría exista en el catálogo de fábrica y coincida con el tipo esperado.
pub fn validar_categoria_para_tipo(
    identificador: &str,
    tipo_esperado: TipoCategoria,
) -> Result<Categoria, ErrorDominio> {
    validar_categoria_para_tipo_en(identificador, tipo_esperado, &catalogo_categorias())
}

/// Valida que no exista ya una categoría del mismo tipo con el mismo nombre (RF-LIB-07).
///
/// La comparación es insensible a mayúsculas, espacios y acentos.
pub fn validar_nueva_categoria_no_duplicada(
    nombre: &str,
    tipo: TipoCategoria,
    categorias_existentes: &[Categoria],
) -> Result<(), ErrorDominio> {
    let clean = nombre.trim();
    if clean.is_empty() {
        return Err(ErrorDominio::categoria_invalida(
            "El nombre de la categoría no puede estar vacío.",
        ));
    }

    let norm = normalizar_para_comparacion(clean);
    for cat in categorias_existentes {
        if cat.tipo == tipo && normalizar_para_comparacion(&cat.nombre) == norm {
            return Err(ErrorDominio::categoria_invalida(format!(
                "Ya existe una categoría de {} con el nombre '{}'. Elige un nombre distinto.",
                tipo, cat.nombre
            )));
        }
    }

    Ok(())
}

/// Valida una colección de categorías asegurando que no existan nombres duplicados
/// dentro del mismo tipo (Gasto o Ingreso).
///
/// La comparación es insensible a mayúsculas, minúsculas, espacios externos y acentos.
pub fn validar_sin_duplicados_mismo_tipo(categorias: &[Categoria]) -> Result<(), ErrorDominio> {
    let mut vistas: HashSet<(TipoCategoria, String)> = HashSet::new();

    for cat in categorias {
        let nombre_trim = cat.nombre.trim();
        if nombre_trim.is_empty() {
            return Err(ErrorDominio::categoria_invalida(
                "El nombre de la categoría no puede estar vacío.",
            ));
        }

        let clave = (cat.tipo, normalizar_para_comparacion(nombre_trim));
        if !vistas.insert(clave) {
            return Err(ErrorDominio::categoria_invalida(format!(
                "No se permiten categorías duplicadas del mismo tipo: '{}' ({}) ya existe en el catálogo.",
                nombre_trim, cat.tipo
            )));
        }
    }

    Ok(())
}

/// Valida que una nueva categoría propuesta no repita el nombre de ninguna categoría existente
/// del mismo tipo.
pub fn validar_categoria_no_repetida(
    existentes: &[Categoria],
    nueva: &Categoria,
) -> Result<(), ErrorDominio> {
    let nombre_nuevo_norm = normalizar_para_comparacion(&nueva.nombre);
    if nombre_nuevo_norm.is_empty() {
        return Err(ErrorDominio::categoria_invalida(
            "El nombre de la categoría no puede estar vacío.",
        ));
    }

    for cat in existentes {
        if cat.tipo == nueva.tipo && normalizar_para_comparacion(&cat.nombre) == nombre_nuevo_norm {
            return Err(ErrorDominio::categoria_invalida(format!(
                "Ya existe una categoría '{}' para movimientos de tipo '{}'.",
                cat.nombre, nueva.tipo
            )));
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_catalogo_completo_contiene_11_categorias() {
        let catalogo = catalogo_categorias();
        assert_eq!(catalogo.len(), 11);
    }

    #[test]
    fn test_catalogo_gastos_contiene_las_7_categorias_oficiales() {
        let gastos = catalogo_gastos();
        assert_eq!(gastos.len(), 7);

        let nombres_esperados = [
            "Alimentación",
            "Transporte",
            "Servicios",
            "Salud",
            "Educación",
            "Entretenimiento",
            "Otros gastos",
        ];

        for esperado in nombres_esperados {
            assert!(
                gastos
                    .iter()
                    .any(|c| c.nombre == esperado && c.tipo == TipoCategoria::Gasto),
                "Falta la categoría de gasto '{}'",
                esperado
            );
        }
    }

    #[test]
    fn test_catalogo_ingresos_contiene_las_4_categorias_oficiales() {
        let ingresos = catalogo_ingresos();
        assert_eq!(ingresos.len(), 4);

        let nombres_esperados = ["Salario", "Ventas", "Regalos", "Otros ingresos"];

        for esperado in nombres_esperados {
            assert!(
                ingresos
                    .iter()
                    .any(|c| c.nombre == esperado && c.tipo == TipoCategoria::Ingreso),
                "Falta la categoría de ingreso '{}'",
                esperado
            );
        }
    }

    #[test]
    fn test_catalogo_de_fabrica_no_tiene_nombres_repetidos_del_mismo_tipo() {
        let catalogo = catalogo_categorias();
        assert!(validar_sin_duplicados_mismo_tipo(&catalogo).is_ok());
    }

    #[test]
    fn test_buscar_categoria_por_nombre_exacto() {
        let cat = buscar_categoria("Alimentación").expect("Debe encontrar Alimentación");
        assert_eq!(cat.id, "alimentacion");
        assert_eq!(cat.tipo, TipoCategoria::Gasto);

        let cat_ingreso = buscar_categoria("Salario").expect("Debe encontrar Salario");
        assert_eq!(cat_ingreso.id, "salario");
        assert_eq!(cat_ingreso.tipo, TipoCategoria::Ingreso);
    }

    #[test]
    fn test_buscar_categoria_por_id_slug() {
        let cat = buscar_categoria("otros_gastos").expect("Debe encontrar por slug");
        assert_eq!(cat.nombre, "Otros gastos");
        assert_eq!(cat.tipo, TipoCategoria::Gasto);

        let cat_ing = buscar_categoria("otros_ingresos").expect("Debe encontrar por slug");
        assert_eq!(cat_ing.nombre, "Otros ingresos");
        assert_eq!(cat_ing.tipo, TipoCategoria::Ingreso);
    }

    #[test]
    fn test_buscar_categoria_insensible_a_mayusculas_y_espacios() {
        let cat = buscar_categoria("  tRaNsPoRtE  ")
            .expect("Debe ser insensible a mayúsculas y espacios");
        assert_eq!(cat.nombre, "Transporte");

        let cat2 = buscar_categoria("VENTAS").expect("Debe encontrar VENTAS");
        assert_eq!(cat2.nombre, "Ventas");
    }

    #[test]
    fn test_buscar_categoria_tolerante_a_tildes() {
        let cat = buscar_categoria("alimentacion").expect("Debe tolerar falta de tilde");
        assert_eq!(cat.nombre, "Alimentación");

        let cat2 = buscar_categoria("Educacion").expect("Debe tolerar falta de tilde");
        assert_eq!(cat2.nombre, "Educación");
    }

    #[test]
    fn test_buscar_categoria_inexistente_falla_con_campo_categoria() {
        let err = buscar_categoria("Criptomonedas").expect_err("Debe fallar categoría desconocida");
        assert_eq!(err.campo, "categoria");
        assert!(err.mensaje.contains("no pertenece al catálogo oficial"));
    }

    #[test]
    fn test_buscar_categoria_vacia_falla_con_campo_categoria() {
        let err = buscar_categoria("   ").expect_err("Debe fallar cadena vacía");
        assert_eq!(err.campo, "categoria");
    }

    #[test]
    fn test_validar_categoria_para_tipo_correcto() {
        let cat = validar_categoria_para_tipo("Salud", TipoCategoria::Gasto).unwrap();
        assert_eq!(cat.nombre, "Salud");
        assert_eq!(cat.tipo, TipoCategoria::Gasto);

        let cat_ing = validar_categoria_para_tipo("Regalos", TipoCategoria::Ingreso).unwrap();
        assert_eq!(cat_ing.nombre, "Regalos");
        assert_eq!(cat_ing.tipo, TipoCategoria::Ingreso);
    }

    #[test]
    fn test_validar_categoria_tipo_equivocado_rechaza_indicando_campo_categoria() {
        // Tratar de usar una categoría de Ingreso (Salario) en un Gasto
        let err_gasto = validar_categoria_para_tipo("Salario", TipoCategoria::Gasto)
            .expect_err("Debe rechazar categoría de ingreso en gasto");
        assert_eq!(err_gasto.campo, "categoria");
        assert!(err_gasto.mensaje.contains("corresponde al tipo 'ingreso'"));

        // Tratar de usar una categoría de Gasto (Educación) en un Ingreso
        let err_ingreso = validar_categoria_para_tipo("Educación", TipoCategoria::Ingreso)
            .expect_err("Debe rechazar categoría de gasto en ingreso");
        assert_eq!(err_ingreso.campo, "categoria");
        assert!(err_ingreso.mensaje.contains("corresponde al tipo 'gasto'"));
    }

    #[test]
    fn test_validar_sin_duplicados_mismo_tipo_detecta_duplicados() {
        let mut categorias = catalogo_categorias();
        // Agregamos una categoría duplicada con diferente casing y espacios
        categorias.push(Categoria {
            id: "alimentacion_2".to_string(),
            nombre: "  alimentación  ".to_string(),
            tipo: TipoCategoria::Gasto,
        });

        let err = validar_sin_duplicados_mismo_tipo(&categorias)
            .expect_err("Debe fallar al haber categorías duplicadas del mismo tipo");
        assert_eq!(err.campo, "categoria");
        assert!(err
            .mensaje
            .contains("No se permiten categorías duplicadas del mismo tipo"));
    }

    #[test]
    fn test_validar_sin_duplicados_permite_mismo_nombre_en_distintos_tipos() {
        // En diferentes tipos (Gasto vs Ingreso) no cuenta como duplicado del mismo tipo
        let categorias = vec![
            Categoria {
                id: "otros_gasto".to_string(),
                nombre: "Otros".to_string(),
                tipo: TipoCategoria::Gasto,
            },
            Categoria {
                id: "otros_ingreso".to_string(),
                nombre: "Otros".to_string(),
                tipo: TipoCategoria::Ingreso,
            },
        ];

        assert!(validar_sin_duplicados_mismo_tipo(&categorias).is_ok());
    }

    #[test]
    fn test_validar_categoria_no_repetida() {
        let existentes = catalogo_categorias();

        // Intento de agregar "Transporte" de nuevo en Gasto
        let duplicada = Categoria {
            id: "transporte_nuevo".to_string(),
            nombre: "transporte".to_string(),
            tipo: TipoCategoria::Gasto,
        };
        let err = validar_categoria_no_repetida(&existentes, &duplicada)
            .expect_err("Debe rechazar categoría repetida del mismo tipo");
        assert_eq!(err.campo, "categoria");

        // Agregar una categoría con nombre nuevo en Gasto
        let valida = Categoria {
            id: "viajes".to_string(),
            nombre: "Viajes".to_string(),
            tipo: TipoCategoria::Gasto,
        };
        assert!(validar_categoria_no_repetida(&existentes, &valida).is_ok());
    }

    #[test]
    fn test_creacion_categoria_nueva_valida_nombre_no_vacio() {
        let err = Categoria::nueva("test", "   ", TipoCategoria::Gasto)
            .expect_err("Debe fallar con nombre vacío");
        assert_eq!(err.campo, "categoria");

        let ok = Categoria::nueva("", "Viajes", TipoCategoria::Gasto)
            .expect("Debe generar id a partir del nombre si id está vacío");
        assert_eq!(ok.id, "viajes");
        assert_eq!(ok.nombre, "Viajes");
    }

    #[test]
    fn test_tipo_categoria_from_str_y_display() {
        assert_eq!(
            TipoCategoria::from_str("gasto").unwrap(),
            TipoCategoria::Gasto
        );
        assert_eq!(
            TipoCategoria::from_str("GASTO").unwrap(),
            TipoCategoria::Gasto
        );
        assert_eq!(
            TipoCategoria::from_str("ingreso").unwrap(),
            TipoCategoria::Ingreso
        );
        assert_eq!(
            TipoCategoria::from_str("Ingreso").unwrap(),
            TipoCategoria::Ingreso
        );

        let err = TipoCategoria::from_str("invalido").expect_err("Debe fallar tipo desconocido");
        assert_eq!(err.campo, "tipo");

        assert_eq!(TipoCategoria::Gasto.to_string(), "gasto");
        assert_eq!(TipoCategoria::Ingreso.to_string(), "ingreso");
    }

    #[test]
    fn test_serializacion_categoria() {
        let cat = Categoria {
            id: "salud".to_string(),
            nombre: "Salud".to_string(),
            tipo: TipoCategoria::Gasto,
        };
        let json = serde_json::to_string(&cat).unwrap();
        assert!(json.contains("\"id\":\"salud\""));
        assert!(json.contains("\"nombre\":\"Salud\""));
        assert!(json.contains("\"tipo\":\"gasto\""));

        let deserializada: Categoria = serde_json::from_str(&json).unwrap();
        assert_eq!(cat, deserializada);
    }
}
