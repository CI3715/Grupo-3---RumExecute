use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::fmt;

use super::errores::ErrorDominio;
use super::moneda::CodigoMoneda;
use super::movimiento::validar_fecha_no_futura; // Conveniente también para esta capa del dominio

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ParMonedas {
    pub origen: CodigoMoneda,
    pub destino: CodigoMoneda,
}

impl ParMonedas {
    pub fn nuevo(origen: CodigoMoneda, destino: CodigoMoneda) -> Result<Self, ErrorDominio> {
        if origen == destino {
            return Err(ErrorDominio::monedas_iguales(&origen.to_string()));
        }
        Ok(Self { origen, destino })
    }
}

impl fmt::Display for ParMonedas {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}/{}", self.origen.as_str(), self.destino.as_str())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TasaCambio {
    pub par: ParMonedas,
    pub valor: Decimal,
    pub fecha: String,
    pub fuente: String,
}

impl TasaCambio {
    pub fn nueva(
        origen: CodigoMoneda,
        destino: CodigoMoneda,
        valor: Decimal,
        fecha: &str,
        fuente: &str,
        momento_actual_utc: &str,
    ) -> Result<Self, ErrorDominio> {
        let par = ParMonedas::nuevo(origen, destino)?;

        if valor <= Decimal::ZERO {
            return Err(ErrorDominio::tasa_invalida("El valor de la tasa de cambio debe ser mayor a cero."));
        }

        if valor.normalize().scale() > 8 {
            return Err(ErrorDominio::tasa_invalida("La tasa de cambio no puede tener más de 8 decimales."));
        }

        validar_fecha_no_futura(fecha, momento_actual_utc)?;

        if fuente.trim().is_empty() {
            return Err(ErrorDominio::tasa_invalida("Debe especificar la fuente de la tasa de cambio."));
        }

        Ok(Self { 
            par,
            valor,
            fecha: fecha.trim().to_string(),
            fuente: fuente.trim().to_string(),
        })
    }
}