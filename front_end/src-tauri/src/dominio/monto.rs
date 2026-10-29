use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::ops::Add;
use std::str::FromStr;

use super::errores::ErrorDominio;
use super::moneda::CodigoMoneda;

/// Value Object `Monto` (INV-01, INV-02, INV-03).
///
/// Representa un valor monetario indivisible compuesto por una cantidad decimal exacta
/// y su código de moneda.
/// - INV-01: Par indivisible cantidad + moneda.
/// - INV-02: Subtotales por moneda (prohíbe operar monedas distintas).
/// - INV-03: Aritmética decimal exacta utilizando `Decimal`, prohibiendo `f32` y `f64`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Monto {
    pub cantidad: Decimal,
    pub moneda: CodigoMoneda,
}

impl Monto {
    /// Crea una nueva instancia de `Monto`.
    pub fn new(cantidad: Decimal, moneda: CodigoMoneda) -> Self {
        Self { cantidad, moneda }
    }

    /// Valida y construye un monto para saldo inicial.
    ///
    /// Reglas de negocio (INV-03):
    /// - No puede ser negativo (`cantidad < 0`).
    /// - No puede tener más de 2 decimales (ej. 10.555 USD).
    ///
    /// Si falla alguna validación, retorna `ErrorDominio` con `campo: "saldo_inicial"`.
    pub fn nuevo_saldo_inicial(
        cantidad: Decimal,
        moneda: CodigoMoneda,
        decimales_admitidos: u32,
    ) -> Result<Self, ErrorDominio> {
        if cantidad < Decimal::ZERO {
            return Err(ErrorDominio::saldo_invalido(
                "El saldo inicial no puede ser negativo.",
            ));
        }

        // Verifica que la cantidad no tenga más de 2 decimales significativos
        if cantidad.normalize().scale() > decimales_admitidos {
            return Err(ErrorDominio::saldo_invalido(format!(
                "El saldo inicial {} tiene más de {} decimales, lo cual no está permitido para la moneda {}.",
                cantidad, decimales_admitidos, moneda.as_str()
            )));
        }

        Ok(Self { cantidad, moneda })
    }

    /// Parsea un saldo inicial a partir de una cadena y un código de moneda.
    pub fn parse_saldo_inicial(
        cantidad_str: &str,
        moneda: CodigoMoneda,
        decimales_admitidos: u32
    ) -> Result<Self, ErrorDominio> {
        // Soporta separador decimal tanto punto '.' como coma ','
        let normalizado = cantidad_str.trim().replace(',', ".");

        let decimal = Decimal::from_str(&normalizado).map_err(|_| {
            ErrorDominio::saldo_invalido(format!(
                "El valor '{}' no es un número decimal válido.",
                cantidad_str
            ))
        })?;

        Self::nuevo_saldo_inicial(decimal, moneda, decimales_admitidos)
    }

    /// Suma dos montos garantizando el invariante INV-02 (no sumar monedas distintas).
    pub fn sumar(&self, otro: &Monto) -> Result<Monto, ErrorDominio> {
        if self.moneda != otro.moneda {
            return Err(ErrorDominio::new(
                "moneda",
                format!(
                    "No se pueden sumar montos de monedas distintas ({} y {}).",
                    self.moneda, otro.moneda
                ),
            ));
        }

        Ok(Monto {
            cantidad: self.cantidad + otro.cantidad,
            moneda: self.moneda,
        })
    }

    /// Valida y construye un monto para un movimiento (Gasto o Ingreso).
    ///
    /// Reglas de negocio (INV-03, RF-LIB-03):
    /// - Monto estrictamente mayor que cero (`cantidad > 0`).
    /// - No puede tener más decimales que los que admite la moneda de la cuenta (RF-CAM-01).
    ///
    /// Si falla alguna validación, retorna `ErrorDominio` con `campo: "monto"`.
    pub fn nuevo_movimiento(
        cantidad: Decimal,
        moneda: CodigoMoneda,
        decimales_admitidos: u32,
    ) -> Result<Self, ErrorDominio> {
        if cantidad <= Decimal::ZERO {
            return Err(ErrorDominio::monto_invalido(
                "El monto del movimiento debe ser estrictamente mayor que cero.",
            ));
        }

        if cantidad.normalize().scale() > decimales_admitidos {
            return Err(ErrorDominio::monto_invalido(format!(
                "El monto {} tiene más de {} decimales, lo cual no está permitido para la moneda {}.",
                cantidad, decimales_admitidos, moneda
            )));
        }

        Ok(Self { cantidad, moneda })
    }

    /// Parsea un monto de movimiento a partir de una cadena y un código de moneda.
    pub fn parse_movimiento(
        cantidad_str: &str,
        moneda: CodigoMoneda,
        decimales_admitidos: u32, 
    ) -> Result<Self, ErrorDominio> {
        let normalizado = cantidad_str.trim().replace(',', ".");

        let decimal = Decimal::from_str(&normalizado).map_err(|_| {
            ErrorDominio::monto_invalido(format!(
                "El valor '{}' no es un número decimal válido.",
                cantidad_str
            ))
        })?;

        Self::nuevo_movimiento(decimal, moneda, decimales_admitidos)
    }

    /// Resta dos montos garantizando el invariante INV-02 (no restar monedas distintas).
    pub fn restar(&self, otro: &Monto) -> Result<Monto, ErrorDominio> {
        if self.moneda != otro.moneda {
            return Err(ErrorDominio::new(
                "moneda",
                format!(
                    "No se pueden operar montos de monedas distintas ({} y {}).",
                    self.moneda, otro.moneda
                ),
            ));
        }

        Ok(Monto {
            cantidad: self.cantidad - otro.cantidad,
            moneda: self.moneda,
        })
    }

    /// Retorna un monto con valor cero para una moneda dada.
    pub fn cero(moneda: CodigoMoneda) -> Self {
        Self {
            cantidad: Decimal::ZERO,
            moneda,
        }
    }
}

/// Implementación del operador `+` para `Monto` respetando INV-02.
impl Add for Monto {
    type Output = Result<Monto, ErrorDominio>;

    fn add(self, rhs: Self) -> Self::Output {
        self.sumar(&rhs)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_inv01_par_indivisible() {
        let monto = Monto::new(Decimal::new(10000, 2), CodigoMoneda::USD);
        assert_eq!(monto.cantidad, Decimal::new(10000, 2));
        assert_eq!(monto.moneda, CodigoMoneda::USD);
    }

    #[test]
    fn test_inv02_prohibir_suma_monedas_distintas() {
        let monto_usd = Monto::new(Decimal::new(5000, 2), CodigoMoneda::USD);
        let monto_ves = Monto::new(Decimal::new(5000, 2), CodigoMoneda::VES);

        let resultado = monto_usd + monto_ves;
        assert!(resultado.is_err());

        let error = resultado.unwrap_err();
        assert_eq!(error.campo, "moneda");
        assert!(error.mensaje.contains("INV-02"));
    }

    #[test]
    fn test_inv02_suma_misma_moneda_correcta() {
        let m1 = Monto::new(Decimal::new(15050, 2), CodigoMoneda::USD); // 150.50
        let m2 = Monto::new(Decimal::new(4950, 2), CodigoMoneda::USD); // 49.50

        let suma = (m1 + m2).expect("Debería sumar exitosamente montos en USD");
        assert_eq!(suma.cantidad, Decimal::new(20000, 2)); // 200.00
        assert_eq!(suma.moneda, CodigoMoneda::USD);
    }

    #[test]
    fn test_inv03_aritmetica_decimal_exacta_sin_errores_de_coma_flotante() {
        // En punto flotante binario (f32/f64), 0.1 + 0.2 es 0.30000000000000004
        // Con rust_decimal, debe ser exactamente 0.3
        let d1 = Decimal::from_str("0.1").unwrap();
        let d2 = Decimal::from_str("0.2").unwrap();
        let m1 = Monto::new(d1, CodigoMoneda::USD);
        let m2 = Monto::new(d2, CodigoMoneda::USD);

        let resultado = (m1 + m2).unwrap();
        assert_eq!(resultado.cantidad, Decimal::from_str("0.3").unwrap());
    }

    #[test]
    fn test_inv03_rechazo_saldo_inicial_negativo() {
        let negativo = Decimal::from_str("-10.00").unwrap();
        let resultado = Monto::nuevo_saldo_inicial(negativo, CodigoMoneda::USD, 2);

        assert!(resultado.is_err());
        let error = resultado.unwrap_err();
        assert_eq!(error.campo, "saldo_inicial");
    }

    #[test]
    fn test_inv03_rechazo_saldo_inicial_con_mas_de_dos_decimales() {
        // Ejemplo de la especificación: 10,555 USD
        let saldo_tres_decimales = Decimal::from_str("10.555").unwrap();
        let resultado = Monto::nuevo_saldo_inicial(saldo_tres_decimales, CodigoMoneda::USD, 2);

        assert!(resultado.is_err());
        let error = resultado.unwrap_err();
        assert_eq!(error.campo, "saldo_inicial");
        assert!(error.mensaje.contains("más de 2 decimales"));
    }

    #[test]
    fn test_saldo_inicial_valido() {
        // Saldo cero es permitido por especificación
        let cero = Monto::nuevo_saldo_inicial(Decimal::ZERO, CodigoMoneda::VES, 2);
        assert!(cero.is_ok());

        // Saldo exacto con dos decimales
        let valido = Monto::parse_saldo_inicial("1250.50", CodigoMoneda::VES, 2);
        assert!(valido.is_ok());

        // Saldo con coma decimal
        let valido_coma = Monto::parse_saldo_inicial("300,00", CodigoMoneda::USD, 2);
        assert!(valido_coma.is_ok());
    }
}
