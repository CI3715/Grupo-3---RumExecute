use crate::aplicacion::puertos::{ErrorRepositorio, RepositorioLibro};
use crate::dominio::categoria::{catalogo_categorias, Categoria, TipoCategoria};
use crate::dominio::evento::EventoLibro;

#[derive(Debug)]
pub enum ErrorListarCategorias {
    Repositorio(ErrorRepositorio),
}

impl From<ErrorRepositorio> for ErrorListarCategorias {
    fn from(e: ErrorRepositorio) -> Self {
        ErrorListarCategorias::Repositorio(e)
    }
}

/// Lista todas las categorías disponibles (incluyendo las de fábrica y las creadas por el usuario).
pub fn listar_categorias(
    repo: &dyn RepositorioLibro,
) -> Result<Vec<Categoria>, ErrorListarCategorias> {
    let mut categorias = catalogo_categorias();

    for evento in repo.listar_eventos()? {
        if let EventoLibro::CategoriaCreada { datos, .. } = evento {
            categorias.push(Categoria {
                id: datos.id_categoria,
                nombre: datos.nombre,
                tipo: datos.tipo,
            });
        }
    }

    Ok(categorias)
}

/// Lista únicamente las categorías del tipo especificado (Gasto o Ingreso).
pub fn listar_categorias_por_tipo(
    repo: &dyn RepositorioLibro,
    tipo: TipoCategoria,
) -> Result<Vec<Categoria>, ErrorListarCategorias> {
    let todas = listar_categorias(repo)?;
    Ok(todas.into_iter().filter(|c| c.tipo == tipo).collect())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dominio::evento::DatosCategoriaCreada;
    use crate::infraestructura::memoria_repo::RepositorioLibroEnMemoria;

    #[test]
    fn devuelve_categorias_de_fabrica_mas_creadas() {
        let mut repo = RepositorioLibroEnMemoria::nuevo();

        // Inicialmente solo las 11 de fábrica
        let iniciales = listar_categorias(&repo).unwrap();
        assert_eq!(iniciales.len(), 11);

        // Agregamos una categoría creada
        let evento = EventoLibro::nuevo_categoria_creada(
            "evt-cat",
            "2026-10-24T12:00:00Z",
            "disp-local",
            DatosCategoriaCreada {
                id_categoria: "cripto".to_string(),
                nombre: "Criptomonedas".to_string(),
                tipo: TipoCategoria::Ingreso,
            },
        );
        repo.guardar(evento).unwrap();

        let actualizadas = listar_categorias(&repo).unwrap();
        assert_eq!(actualizadas.len(), 12);
        assert!(actualizadas.iter().any(|c| c.nombre == "Criptomonedas"));

        // Filtrado por tipo
        let ingresos = listar_categorias_por_tipo(&repo, TipoCategoria::Ingreso).unwrap();
        assert_eq!(ingresos.len(), 5); // 4 de fábrica + 1 creada
    }
}
