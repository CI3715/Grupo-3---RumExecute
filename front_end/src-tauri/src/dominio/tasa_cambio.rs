use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::fmt;

use super::errores::ErrorDominio;
user super::moneda::CodigoMoneda;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ParMonedas {
    pub origen: CodigoMoneda,
    pub destino: CodigoMoneda,
}

pub struct TasaCambio {
    pub par_monedas: ParMonedas,
    pub valor: Decimal,
}