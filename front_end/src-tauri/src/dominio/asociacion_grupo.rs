use serde::{Deserialize, Serialize};

use super::cuenta::Cuenta;
use super::errores::ErrorDominio;
use super::moneda::CodigoMoneda;

/// Asociación local entre un grupo compartido y una cuenta personal.
///
/// Esta información nunca debe enviarse al servidor.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AsociacionGrupoCuenta {
    pub grupo_id: String,
    pub cuenta_id: String,
}

impl AsociacionGrupoCuenta {
    /// Crea una asociación validando que ambas monedas coincidan.
    pub fn nueva(
        grupo_id: impl Into<String>,
        moneda_grupo: CodigoMoneda,
        cuenta: &Cuenta,
    ) -> Result<Self, ErrorDominio> {
        let grupo_id = grupo_id.into();

        if grupo_id.trim().is_empty() {
            return Err(ErrorDominio::new(
                "grupo_id",
                "El identificador del grupo no puede estar vacío.",
            ));
        }

        if cuenta.id.trim().is_empty() {
            return Err(ErrorDominio::new(
                "cuenta_id",
                "El identificador de la cuenta no puede estar vacío.",
            ));
        }

        cuenta.validar_moneda_grupo(moneda_grupo)?;

        Ok(Self {
            grupo_id: grupo_id.trim().to_string(),
            cuenta_id: cuenta.id.clone(),
        })
    }

    /// Crea una nueva asociación para el mismo grupo,
    /// utilizando otra cuenta personal.
    ///
    /// Los movimientos anteriores permanecen asociados
    /// a las cuentas donde fueron registrados.
    pub fn cambiar_cuenta(
        &self,
        moneda_grupo: CodigoMoneda,
        nueva_cuenta: &Cuenta,
    ) -> Result<Self, ErrorDominio> {
        Self::nueva(
            &self.grupo_id,
            moneda_grupo,
            nueva_cuenta,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dominio::cuenta::TipoCuenta;
    use crate::dominio::monto::Monto;

    fn cuenta(id: &str, moneda: CodigoMoneda) -> Cuenta {
        Cuenta::nueva(
            id,
            "Cuenta personal",
            TipoCuenta::Banco,
            moneda,
            Monto::cero(moneda),
        )
        .unwrap()
    }

    #[test]
    fn crea_asociacion_valida() {
        let cuenta = cuenta("cta-1", CodigoMoneda::USD);

        let asociacion = AsociacionGrupoCuenta::nueva(
            "grupo-1",
            CodigoMoneda::USD,
            &cuenta,
        )
        .unwrap();

        assert_eq!(asociacion.grupo_id, "grupo-1");
        assert_eq!(asociacion.cuenta_id, "cta-1");
    }

    #[test]
    fn rechaza_monedas_distintas() {
        let cuenta = cuenta("cta-1", CodigoMoneda::EUR);

        let error = AsociacionGrupoCuenta::nueva(
            "grupo-1",
            CodigoMoneda::USD,
            &cuenta,
        )
        .unwrap_err();

        assert_eq!(error.campo, "moneda");
    }

    #[test]
    fn rechaza_grupo_vacio() {
        let cuenta = cuenta("cta-1", CodigoMoneda::USD);

        let error = AsociacionGrupoCuenta::nueva(
            "   ",
            CodigoMoneda::USD,
            &cuenta,
        )
        .unwrap_err();

        assert_eq!(error.campo, "grupo_id");
    }

    #[test]
    fn permite_cambiar_cuenta() {
        let cuenta_1 = cuenta("cta-1", CodigoMoneda::USD);
        let cuenta_2 = cuenta("cta-2", CodigoMoneda::USD);

        let original = AsociacionGrupoCuenta::nueva(
            "grupo-1",
            CodigoMoneda::USD,
            &cuenta_1,
        )
        .unwrap();

        let nueva = original
            .cambiar_cuenta(CodigoMoneda::USD, &cuenta_2)
            .unwrap();

        assert_eq!(original.cuenta_id, "cta-1");
        assert_eq!(nueva.cuenta_id, "cta-2");
        assert_eq!(nueva.grupo_id, original.grupo_id);
    }

    #[test]
    fn rechaza_cambiar_a_cuenta_de_otra_moneda() {
        let cuenta_1 = cuenta("cta-1", CodigoMoneda::USD);
        let cuenta_2 = cuenta("cta-2", CodigoMoneda::VES);

        let asociacion = AsociacionGrupoCuenta::nueva(
            "grupo-1",
            CodigoMoneda::USD,
            &cuenta_1,
        )
        .unwrap();

        let error = asociacion
            .cambiar_cuenta(CodigoMoneda::USD, &cuenta_2)
            .unwrap_err();

        assert_eq!(error.campo, "moneda");
        assert_eq!(asociacion.cuenta_id, "cta-1");
    }

    #[test]
    fn serializa_y_deserializa_asociacion() {
        let cuenta = cuenta("cta-1", CodigoMoneda::USD);

        let asociacion = AsociacionGrupoCuenta::nueva(
            "grupo-1",
            CodigoMoneda::USD,
            &cuenta,
        )
        .unwrap();

        let json = serde_json::to_string(&asociacion).unwrap();

        let recuperada: AsociacionGrupoCuenta =
            serde_json::from_str(&json).unwrap();

        assert_eq!(recuperada, asociacion);
    }
}
