use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

use super::errores::ErrorDominio;
use super::moneda::CodigoMoneda;
use super::monto::Monto;

/// Tipos de cuenta soportados según RF-LIB-01.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TipoCuenta {
    Efectivo,
    Banco,
    BilleteraDigital,
}

impl TipoCuenta {
    pub fn as_str(&self) -> &'static str {
        match self {
            TipoCuenta::Efectivo => "efectivo",
            TipoCuenta::Banco => "banco",
            TipoCuenta::BilleteraDigital => "billetera_digital",
        }
    }
}

impl fmt::Display for TipoCuenta {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl FromStr for TipoCuenta {
    type Err = ErrorDominio;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_lowercase().as_str() {
            "efectivo" => Ok(TipoCuenta::Efectivo),
            "banco" => Ok(TipoCuenta::Banco),
            "billetera_digital" | "billeteradigital" => Ok(TipoCuenta::BilleteraDigital),
            _ => Err(ErrorDominio::tipo_invalido(format!(
                "El tipo de cuenta '{}' no es válido. Opciones permitidas: 'efectivo', 'banco', 'billetera_digital'.",
                s
            ))),
        }
    }
}

/// Entidad Cuenta (RF-LIB-01, INV-18).
///
/// Invariante INV-18: La moneda de una cuenta es inmutable.
/// No existe ningún método público para modificar la moneda una vez creada la cuenta.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Cuenta {
    pub id: String,
    pub nombre: String,
    pub tipo: TipoCuenta,
    pub moneda: CodigoMoneda,
    pub saldo_inicial: Monto,
}

impl Cuenta {
    /// Crea y valida una nueva entidad Cuenta.
    ///
    /// Validaciones:
    /// - Nombre no puede estar vacío ni contener solo espacios (indica campo: "nombre").
    /// - La moneda del saldo inicial debe coincidir con la moneda asignada a la cuenta (INV-01).
    pub fn nueva(
        id: impl Into<String>,
        nombre: impl Into<String>,
        tipo: TipoCuenta,
        moneda: CodigoMoneda,
        saldo_inicial: Monto,
    ) -> Result<Self, ErrorDominio> {
        let nombre = nombre.into();
        let trimmed_nombre = nombre.trim();

        if trimmed_nombre.is_empty() {
            return Err(ErrorDominio::nombre_invalido(
                "El nombre de la cuenta no puede estar vacío ni contener solo espacios en blanco.",
            ));
        }

        if saldo_inicial.moneda != moneda {
            return Err(ErrorDominio::new(
                "moneda",
                format!(
                    "La moneda del saldo inicial ({}) no coincide con la moneda de la cuenta ({}).",
                    saldo_inicial.moneda, moneda
                ),
            ));
        }

        Ok(Self {
            id: id.into(),
            nombre: trimmed_nombre.to_string(),
            tipo,
            moneda,
            saldo_inicial,
        })
    }

    /// Retorna el saldo actual de la cuenta. En esta entrega es igual al saldo inicial.
    pub fn saldo_actual(&self) -> Monto {
        self.saldo_inicial
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal::Decimal;

    #[test]
    fn test_creacion_cuenta_valida() {
        let saldo = Monto::nuevo_saldo_inicial(Decimal::new(10000, 2), CodigoMoneda::USD).unwrap();
        let cuenta = Cuenta::nueva(
            "cta-1",
            "Cartera Personal",
            TipoCuenta::Efectivo,
            CodigoMoneda::USD,
            saldo,
        );

        assert!(cuenta.is_ok());
        let c = cuenta.unwrap();
        assert_eq!(c.id, "cta-1");
        assert_eq!(c.nombre, "Cartera Personal");
        assert_eq!(c.tipo, TipoCuenta::Efectivo);
        assert_eq!(c.moneda, CodigoMoneda::USD);
        assert_eq!(c.saldo_inicial, saldo);
        assert_eq!(c.saldo_actual(), saldo);
    }

    #[test]
    fn test_rechazo_nombre_vacio_o_solo_espacios() {
        let saldo = Monto::cero(CodigoMoneda::VES);

        let vacio = Cuenta::nueva("1", "", TipoCuenta::Banco, CodigoMoneda::VES, saldo);
        assert!(vacio.is_err());
        assert_eq!(vacio.unwrap_err().campo, "nombre");

        let solo_espacios = Cuenta::nueva(
            "2",
            "   \t\n  ",
            TipoCuenta::Banco,
            CodigoMoneda::VES,
            saldo,
        );
        assert!(solo_espacios.is_err());
        assert_eq!(solo_espacios.unwrap_err().campo, "nombre");
    }

    #[test]
    fn test_inv18_moneda_inmutable() {
        // INV-18: La moneda es inmutable y no existen métodos de modificación de moneda en Cuenta.
        let saldo = Monto::cero(CodigoMoneda::EUR);
        let cuenta = Cuenta::nueva(
            "cta-eur",
            "Cuenta Banco Europa",
            TipoCuenta::Banco,
            CodigoMoneda::EUR,
            saldo,
        )
        .unwrap();

        assert_eq!(cuenta.moneda, CodigoMoneda::EUR);
        // La moneda no cambia con el saldo actual:
        assert_eq!(cuenta.saldo_actual().moneda, CodigoMoneda::EUR);
    }

    #[test]
    fn test_tipos_cuenta_serializacion_y_parse() {
        assert_eq!(
            TipoCuenta::from_str("efectivo").unwrap(),
            TipoCuenta::Efectivo
        );
        assert_eq!(TipoCuenta::from_str("banco").unwrap(), TipoCuenta::Banco);
        assert_eq!(
            TipoCuenta::from_str("billetera_digital").unwrap(),
            TipoCuenta::BilleteraDigital
        );

        let error = TipoCuenta::from_str("tarjeta_credito");
        assert!(error.is_err());
        assert_eq!(error.unwrap_err().campo, "tipo");

        let json = serde_json::to_string(&TipoCuenta::BilleteraDigital).unwrap();
        assert_eq!(json, "\"billetera_digital\"");
    }
}
