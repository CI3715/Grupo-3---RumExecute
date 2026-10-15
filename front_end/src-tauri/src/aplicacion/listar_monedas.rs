use crate::dominio::moneda::{catalogo_monedas, Moneda};

/// Caso de uso: Listar las monedas disponibles en el catálogo base.
/// 
/// Este caso de uso consulta el catálogo de monedas de fábrica definido en el dominio
/// (VES, USD, EUR, USDT) para exponerlo a los adaptadores de entrada (Tauri IPC).
pub struct CasoUsoListarMonedas;

impl CasoUsoListarMonedas {
    pub fn new() -> Self {
        Self
    }

    /// Ejecuta el caso de uso devolviendo la lista de monedas del catálogo.
    pub fn ejecutar(&self) -> Vec<Moneda> {
        catalogo_monedas()
    }
}

impl Default for CasoUsoListarMonedas {
    fn default() -> Self {
        Self::new()
    }
}

/// Función autónoma alternativa en caso de preferencia por estilo funcional directo.
pub fn listar_monedas() -> Vec<Moneda> {
    catalogo_monedas()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn debe_retornar_las_cuatro_monedas_del_catalogo_base() {
        let caso_de_uso = CasoUsoListarMonedas::new();
        let monedas = caso_de_uso.ejecutar();

        assert_eq!(monedas.len(), 4);
        
        let codigos: Vec<String> = monedas.into_iter().map(|m| m.codigo.to_string()).collect();
        assert!(codigos.contains(&"VES".to_string()));
        assert!(codigos.contains(&"USD".to_string()));
        assert!(codigos.contains(&"EUR".to_string()));
        assert!(codigos.contains(&"USDT".to_string()));
    }
}