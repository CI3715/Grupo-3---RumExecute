use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AsignarCuentaGrupoEntrada {
    pub grupo_id: String,
    pub cuenta_id: String,
    pub moneda_cuenta: String,
    pub moneda_grupo: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResultadoAsignarCuentaDto {
    pub grupo_id: String,
    pub cuenta_id: String,
    pub partes_aplicadas: usize,
}

pub trait RepositorioAsignacionCuenta: Send + Sync {
    fn asignar_y_aplicar_pendientes(
        &mut self,
        grupo_id: &str,
        cuenta_id: &str,
        moneda: &str,
        momento_actual: &str,
        dispositivo: &str,
    ) -> Result<usize, String>;
}

pub struct AsignarCuentaGrupo<'a> {
    repositorio: &'a mut dyn RepositorioAsignacionCuenta,
}

impl<'a> AsignarCuentaGrupo<'a> {
    pub fn new(repositorio: &'a mut dyn RepositorioAsignacionCuenta) -> Self {
        Self { repositorio }
    }

    pub fn ejecutar(
        &mut self,
        entrada: AsignarCuentaGrupoEntrada,
        momento_actual: &str,
        dispositivo: &str,
    ) -> Result<ResultadoAsignarCuentaDto, String> {
        if entrada.grupo_id.trim().is_empty() {
            return Err("El id del grupo es obligatorio".into());
        }
        if entrada.cuenta_id.trim().is_empty() {
            return Err("El id de la cuenta es obligatorio".into());
        }

        // Invariante INV-18: Solo se permiten cuentas en la moneda exacta del grupo
        if entrada.moneda_cuenta.trim().to_uppercase() != entrada.moneda_grupo.trim().to_uppercase() {
            return Err(format!(
                "Moneda incompatible (INV-18): la cuenta está en {} y el grupo en {}",
                entrada.moneda_cuenta, entrada.moneda_grupo
            ));
        }

        let aplicadas = self.repositorio.asignar_y_aplicar_pendientes(
            &entrada.grupo_id,
            &entrada.cuenta_id,
            &entrada.moneda_cuenta.to_uppercase(),
            momento_actual,
            dispositivo,
        )?;

        Ok(ResultadoAsignarCuentaDto {
            grupo_id: entrada.grupo_id,
            cuenta_id: entrada.cuenta_id,
            partes_aplicadas: aplicadas,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockRepoAsignacion {
        asignaciones: Vec<(String, String)>,
    }

    impl RepositorioAsignacionCuenta for MockRepoAsignacion {
        fn asignar_y_aplicar_pendientes(
            &mut self,
            grupo_id: &str,
            cuenta_id: &str,
            _moneda: &str,
            _momento: &str,
            _disp: &str,
        ) -> Result<usize, String> {
            self.asignaciones.push((grupo_id.into(), cuenta_id.into()));
            Ok(2) // Simula que aplicó 2 partes que estaban pendientes
        }
    }

    #[test]
    fn test_rechaza_cuenta_con_distinta_moneda_inv_18() {
        let mut repo = MockRepoAsignacion { asignaciones: vec![] };
        let mut caso_uso = AsignarCuentaGrupo::new(&mut repo);

        let res = caso_uso.ejecutar(
            AsignarCuentaGrupoEntrada {
                grupo_id: "g-1".into(),
                cuenta_id: "c-bs".into(),
                moneda_cuenta: "VES".into(),
                moneda_grupo: "USD".into(),
            },
            "2026-11-22T10:00:00Z",
            "app-desktop",
        );

        assert!(res.is_err());
        assert!(res.unwrap_err().contains("INV-18"));
        assert_eq!(repo.asignaciones.len(), 0);
    }

    #[test]
    fn test_misma_moneda_asigna_y_aplica_pendientes() {
        let mut repo = MockRepoAsignacion { asignaciones: vec![] };
        let mut caso_uso = AsignarCuentaGrupo::new(&mut repo);

        let res = caso_uso.ejecutar(
            AsignarCuentaGrupoEntrada {
                grupo_id: "g-1".into(),
                cuenta_id: "c-usd".into(),
                moneda_cuenta: "USD".into(),
                moneda_grupo: "USD".into(),
            },
            "2026-11-22T10:00:00Z",
            "app-desktop",
        ).unwrap();

        assert_eq!(res.partes_aplicadas, 2);
        assert_eq!(repo.asignaciones.len(), 1);
    }
}