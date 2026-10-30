use crate::dominio::moneda::{catalogo_monedas, Moneda};
use crate::aplicacion::proyeccion_monedas::catalogo_desde_eventos;
use crate::aplicacion::puertos::{ErrorRepositorio, RepositorioLibro};
use crate::dominio::errores::ErrorDominio;
use crate::dominio::evento::EventoLibro;

/// Caso de uso: listar las monedas del catálogo base (RF-CAM-01).
pub fn listar_monedas() -> Vec<Moneda> {
    catalogo_monedas()
}

#[derive(Debug)]
pub enum ErrorListarMonedas {
    Dominio(ErrorDominio),
    Repositorio(ErrorRepositorio),
}

impl From<ErrorDominio> for ErrorListarMonedas {
    fn from(error: ErrorDominio) -> Self {
        Self::Dominio(error)
    }
}

impl From<ErrorRepositorio> for ErrorListarMonedas {
    fn from(error: ErrorRepositorio) -> Self {
        Self::Repositorio(error)
    }
}

pub fn listar_monedas_registradas(
    repo: &dyn RepositorioLibro,
) -> Result<Vec<Moneda>, ErrorListarMonedas> {
    let eventos: Vec<EventoLibro> = repo.listar_eventos()?;
    let mut monedas = catalogo_desde_eventos(&eventos)?.listar_todas();
    monedas.sort_by(|izquierda, derecha| izquierda.codigo.as_str().cmp(derecha.codigo.as_str()));
    Ok(monedas)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn devuelve_las_cuatro_monedas_base_con_dos_decimales() {
        let monedas = listar_monedas();
        assert_eq!(monedas.len(), 4);

        let codigos: Vec<String> = monedas.iter().map(|m| m.codigo.to_string()).collect();
        for esperado in ["VES", "USD", "EUR", "USDT"] {
            assert!(codigos.contains(&esperado.to_string()));
        }
        assert!(monedas.iter().all(|m| m.decimales == 2));
    }
}
