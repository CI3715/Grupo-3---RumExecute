use crate::dominio::moneda::{catalogo_monedas, Moneda};

/// Caso de uso: listar las monedas del catálogo base (RF-CAM-01).
pub fn listar_monedas() -> Vec<Moneda> {
    catalogo_monedas()
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
