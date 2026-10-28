use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

use super::errores::ErrorDominio;

/// Códigos de moneda permitidos en el catálogo de fábrica (RF-CAM-01).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum CodigoMoneda {
    VES,
    USD,
    EUR,
    USDT,
    Personalizado(String),
}

impl CodigoMoneda {
    pub fn as_str(&self) -> &'static str {
        match self {
            CodigoMoneda::VES => "VES",
            CodigoMoneda::USD => "USD",
            CodigoMoneda::EUR => "EUR",
            CodigoMoneda::USDT => "USDT",
            CodigoMoneda::Personalizado(c) => c.as_str(),
        }
    }

    pub fn parse(codigo: &str) -> Result<Self, ErrorDominio> {
        let cod = codigo.trim().to_uppercase();
        match cod.as_str() {
            "VES" => Ok(CodigoMoneda::VES),
            "USD" => Ok(CodigoMoneda::USD),
            "EUR" => Ok(CodigoMoneda::EUR),
            "USDT" => Ok(CodigoMoneda::USDT),
            _ => {
                if cod.len() < 3 || cod.len() > 5 || !cod.chars().all(|c| c.is_ascii_uppercase()) {
                    return Err(ErrorDominio::codigo_moneda_invalido());
                }
                Ok(CodigoMoneda::Personalizado(cod))
            }
        }
    }
}

impl fmt::Display for CodigoMoneda {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl FromStr for CodigoMoneda {
    type Err = ErrorDominio;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        CodigoMoneda::parse(s)
    }
}

/// Representa una moneda del catálogo oficial (RF-CAM-01).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Moneda {
    pub codigo: CodigoMoneda,
    pub nombre: String,
    pub simbolo: String,
    pub decimales: u32,
}

impl Moneda {
    // Catálogo de Monedas de Fábrica (RF-CAM-01)
    pub fn monedas_de_fabrica() -> Vec<Moneda> {
        vec![
            Moneda {
                codigo: CodigoMoneda::VES,
                nombre: "Bolívar".to_string(),
                simbolo: "Bs.".to_string(),
                decimales: 2,
            },
            Moneda {
                codigo: CodigoMoneda::USD,
                nombre: "Dólar estadounidense".to_string(),
                simbolo: "$".to_string(),
                decimales: 2,
            },
            Moneda {
                codigo: CodigoMoneda::EUR,
                nombre: "Euro".to_string(),
                simbolo: "€".to_string(),
                decimales: 2,
            },
            Moneda {
                codigo: CodigoMoneda::USDT,
                nombre: "Tether (dólar digital)".to_string(),
                simbolo: "USDT".to_string(),
                decimales: 2,
            },
        ]
    }

    // Crea y valida una nueva moneda definida por el usuario
    pub fn nueva_propia(
        codigo: &str,
        nombre: String,
        simbolo: String,
        decimales: u8,
    ) -> Result<Self, ErrorDominio> {
        let cod_moneda = CodigoMoneda::parse(codigo)?;

        if nombre.trim().is_empty() || nombre.trim().len() > 50 {
            return Err(ErrorDominio::nombre_moneda_invalido());
        }

        if simbolo.trim().is_empty() || simbolo.trim().chars().count() > 5 {
            return Err(ErrorDominio::simbolo_moneda_invalido());
        }

        if decimales > 8 {
            return Err(ErrorDominio::decimales_moneda_invalido());
        }

        Ok(Moneda {
            codigo: cod_moneda,
            nombre: nombre.trim().to_string(),
            simbolo: simbolo.trim().to_string(),
            decimales,
        })
    }
}

/// Retorna el catálogo completo de monedas incluido de fábrica.
/// Según la especificación RF-CAM-01:
/// - VES: Bolívar, Bs., 2 decimales
/// - USD: Dólar estadounidense, $, 2 decimales
/// - EUR: Euro, €, 2 decimales
/// - USDT: Tether (dólar digital), USDT, 2 decimales
pub fn catalogo_monedas() -> Vec<Moneda> {
    vec![
        Moneda {
            codigo: CodigoMoneda::VES,
            nombre: "Bolívar".to_string(),
            simbolo: "Bs.".to_string(),
            decimales: 2,
        },
        Moneda {
            codigo: CodigoMoneda::USD,
            nombre: "Dólar estadounidense".to_string(),
            simbolo: "$".to_string(),
            decimales: 2,
        },
        Moneda {
            codigo: CodigoMoneda::EUR,
            nombre: "Euro".to_string(),
            simbolo: "€".to_string(),
            decimales: 2,
        },
        Moneda {
            codigo: CodigoMoneda::USDT,
            nombre: "Tether (dólar digital)".to_string(),
            simbolo: "USDT".to_string(),
            decimales: 2,
        },
    ]
}

/// Valida un código de moneda y devuelve su entidad `Moneda` correspondiente del catálogo.
/// Si la moneda no pertenece al catálogo, retorna un `ErrorDominio` con `campo: "moneda"`.
pub fn buscar_moneda(codigo: &str) -> Result<Moneda, ErrorDominio> {
    let codigo_enum = CodigoMoneda::from_str(codigo)?;
    catalogo_monedas()
        .into_iter()
        .find(|m| m.codigo == codigo_enum)
        .ok_or_else(|| ErrorDominio::moneda_invalida(codigo))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_catalogo_contiene_las_4_monedas_base_con_2_decimales() {
        let catalogo = catalogo_monedas();
        assert_eq!(catalogo.len(), 4);

        let ves = catalogo
            .iter()
            .find(|m| m.codigo == CodigoMoneda::VES)
            .unwrap();
        assert_eq!(ves.nombre, "Bolívar");
        assert_eq!(ves.simbolo, "Bs.");
        assert_eq!(ves.decimales, 2);

        let usd = catalogo
            .iter()
            .find(|m| m.codigo == CodigoMoneda::USD)
            .unwrap();
        assert_eq!(usd.nombre, "Dólar estadounidense");
        assert_eq!(usd.simbolo, "$");
        assert_eq!(usd.decimales, 2);

        let eur = catalogo
            .iter()
            .find(|m| m.codigo == CodigoMoneda::EUR)
            .unwrap();
        assert_eq!(eur.nombre, "Euro");
        assert_eq!(eur.simbolo, "€");
        assert_eq!(eur.decimales, 2);

        let usdt = catalogo
            .iter()
            .find(|m| m.codigo == CodigoMoneda::USDT)
            .unwrap();
        assert_eq!(usdt.nombre, "Tether (dólar digital)");
        assert_eq!(usdt.simbolo, "USDT");
        assert_eq!(usdt.decimales, 2);
    }

    #[test]
    fn test_busqueda_moneda_valida() {
        assert!(buscar_moneda("VES").is_ok());
        assert!(buscar_moneda("USD").is_ok());
        assert!(buscar_moneda("EUR").is_ok());
        assert!(buscar_moneda("USDT").is_ok());
    }

    #[test]
    fn test_rechazo_moneda_fuera_del_catalogo() {
        let resultado = buscar_moneda("BTC");
        assert!(resultado.is_err());
        let err = resultado.unwrap_err();
        assert_eq!(err.campo, "moneda");

        let resultado_vacio = buscar_moneda("");
        assert!(resultado_vacio.is_err());
        assert_eq!(resultado_vacio.unwrap_err().campo, "moneda");

        let resultado_otra = buscar_moneda("COP");
        assert!(resultado_otra.is_err());
        assert_eq!(resultado_otra.unwrap_err().campo, "moneda");
    }

    #[test]
    fn test_serializacion_moneda() {
        let moneda = buscar_moneda("USD").unwrap();
        let json = serde_json::to_string(&moneda).unwrap();
        assert!(json.contains("\"codigo\":\"USD\""));
        assert!(json.contains("\"nombre\":\"Dólar estadounidense\""));
        assert!(json.contains("\"simbolo\":\"$\""));
        assert!(json.contains("\"decimales\":2"));
    }
}
