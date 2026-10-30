use serde::{de, Deserialize, Deserializer, Serialize, Serializer};
use std::collections::HashMap;
use std::fmt;
use std::str::FromStr;

use super::errores::ErrorDominio;

// Guarda un String de máximo 5 de longitud, que corresponde a un código de Moneda.
// Literamente existe solo para no tener que modificar todo el resto del código (Gracias rust, te estoy odiando)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct StringCodigo {
    bytes: [u8; 5],
    len: u8,
}

impl StringCodigo {
    pub fn nuevo(s: &str) -> Result<Self, ErrorDominio> {
        let cod = s.trim().to_uppercase();
        let bytes_vec = cod.as_bytes();

        if bytes_vec.len() < 3
            || bytes_vec.len() > 5
            || !cod.chars().all(|c| c.is_ascii_uppercase())
        {
            return Err(ErrorDominio::codigo_moneda_invalido());
        }

        let mut bytes = [0u8; 5];
        bytes[..bytes_vec.len()].copy_from_slice(bytes_vec);
        Ok(Self {
            bytes,
            len: bytes_vec.len() as u8,
        })
    }

    pub fn as_str(&self) -> &str {
        std::str::from_utf8(&self.bytes[..self.len as usize]).unwrap_or("")
    }
}

/// Códigos de moneda permitidos en el catálogo de fábrica (RF-CAM-01).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CodigoMoneda {
    VES,
    USD,
    EUR,
    USDT,
    Personalizado(StringCodigo),
}

impl CodigoMoneda {
    pub fn as_str(&self) -> &str {
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
                let pers = StringCodigo::nuevo(&cod)?;
                Ok(CodigoMoneda::Personalizado(pers))
            }
        }
    }

    // TEMPORAL: Mientras se implementa la lógica de las tasas, para no romper el resto del proyecto
    pub fn decimales(&self) -> u32 {
        2
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

impl Serialize for CodigoMoneda {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for CodigoMoneda {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        CodigoMoneda::parse(&s).map_err(de::Error::custom)
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
        decimales: u32,
    ) -> Result<Self, ErrorDominio> {
        if codigo.trim() != codigo
            || !(3..=5).contains(&codigo.len())
            || !codigo.bytes().all(|caracter| caracter.is_ascii_uppercase())
        {
            return Err(ErrorDominio::codigo_moneda_invalido());
        }
        let cod_moneda = CodigoMoneda::parse(codigo)?;

        if nombre.trim().is_empty() || nombre.trim().chars().count() > 50 {
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

// Guarda todas las monedas, tanto de fábrica como personalizadas
#[derive(Debug, Clone, Default)]
pub struct CatalogoMonedas {
    monedas: HashMap<CodigoMoneda, Moneda>,
}

impl CatalogoMonedas {
    // Inicializa el catálogo cargando automáticamente las monedas de fábrica
    pub fn nuevo() -> Self {
        let mut catalogo = Self {
            monedas: HashMap::new(),
        };

        for moneda in Moneda::monedas_de_fabrica() {
            catalogo.monedas.insert(moneda.codigo, moneda);
        }

        catalogo
    }

    // Registra una moneda personalizada en el catálogo.
    // Esto se llama al procesar un evento `MonedaCreada`.
    pub fn registrar_moneda(&mut self, moneda: Moneda) {
        self.monedas.insert(moneda.codigo, moneda);
    }

    // Recupera la entidad `Moneda` completa por su código.
    pub fn obtener(&self, codigo: &CodigoMoneda) -> Option<&Moneda> {
        self.monedas.get(codigo)
    }

    // Obtiene de forma segura los decimales exactos de una moneda.
    pub fn decimales(&self, codigo: &CodigoMoneda) -> Result<u32, ErrorDominio> {
        self.obtener(codigo)
            .map(|m| m.decimales)
            .ok_or_else(|| ErrorDominio::moneda_no_encontrada(codigo.as_str()))
    }

    // Retorna una lista con todas las monedas (útil para enviar a la interfaz gráfica).
    pub fn listar_todas(&self) -> Vec<Moneda> {
        self.monedas.values().cloned().collect()
    }
}

pub fn catalogo_monedas() -> Vec<Moneda> {
    Moneda::monedas_de_fabrica()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_catalogo_contiene_las_4_monedas_base_con_2_decimales() {
        let catalogo = Moneda::monedas_de_fabrica();
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
    fn test_parse_codigos_de_fabrica() {
        assert_eq!(CodigoMoneda::parse("ves").unwrap(), CodigoMoneda::VES);
        assert_eq!(CodigoMoneda::parse(" USD ").unwrap(), CodigoMoneda::USD);
        assert_eq!(CodigoMoneda::parse("Eur").unwrap(), CodigoMoneda::EUR);
        assert_eq!(CodigoMoneda::parse("USDT").unwrap(), CodigoMoneda::USDT);
    }

    #[test]
    fn test_parse_codigo_personalizado_valido() {
        let cod = CodigoMoneda::parse("cad").unwrap();
        assert_eq!(cod.as_str(), "CAD");

        let cod_5_letras = CodigoMoneda::parse("ABCDE").unwrap();
        assert_eq!(cod_5_letras.as_str(), "ABCDE");
    }

    #[test]
    fn test_parse_codigo_personalizado_invalido() {
        assert!(CodigoMoneda::parse("US").is_err());
        assert!(CodigoMoneda::parse("USDTX1").is_err());
        assert!(CodigoMoneda::parse("US1").is_err());
        assert!(CodigoMoneda::parse("U S").is_err());
        assert!(CodigoMoneda::parse("USD$").is_err());
    }

    #[test]
    fn test_crear_moneda_propia_exitosa() {
        let moneda =
            Moneda::nueva_propia("CAD", "Dólar canadiense".to_string(), "$".to_string(), 5)
                .unwrap();

        assert_eq!(moneda.codigo.as_str(), "CAD");
        assert_eq!(moneda.nombre, "Dólar canadiense");
        assert_eq!(moneda.simbolo, "$");
        assert_eq!(moneda.decimales, 5);
    }

    #[test]
    fn test_rechazo_moneda_propia_invalida() {
        // Nombre vacío
        assert!(Moneda::nueva_propia("CAD", "".to_string(), "C$".to_string(), 2).is_err());

        // Nombre demasiado largo
        assert!(Moneda::nueva_propia("CAD", "C".repeat(51), "C$".to_string(), 2).is_err());

        // Símbolo vacío
        assert!(
            Moneda::nueva_propia("CAD", "Dólar canadiense".to_string(), "".to_string(), 2).is_err()
        );

        // Símbolo demasiado largo
        assert!(Moneda::nueva_propia(
            "CAD",
            "Dólar canadiense".to_string(),
            "123456".to_string(),
            2
        )
        .is_err());

        // Decimales mayores a 8
        assert!(
            Moneda::nueva_propia("CAD", "Dólar canadiense".to_string(), "$".to_string(), 9)
                .is_err()
        );
    }

    #[test]
    fn test_serializacion_moneda() {
        // Prueba serialización de una moneda del catálogo de fábrica
        let moneda_usd = Moneda::monedas_de_fabrica()
            .into_iter()
            .find(|m| m.codigo == CodigoMoneda::USD)
            .expect("USD debe existir en el catálogo de fábrica");

        let json_usd = serde_json::to_string(&moneda_usd).unwrap();
        assert!(json_usd.contains("\"codigo\":\"USD\""));
        assert!(json_usd.contains("\"nombre\":\"Dólar estadounidense\""));
        assert!(json_usd.contains("\"simbolo\":\"$\""));
        assert!(json_usd.contains("\"decimales\":2"));

        // Prueba serialización de una moneda personalizada
        let moneda_cad =
            Moneda::nueva_propia("CAD", "Dólar canadiense".to_string(), "$".to_string(), 2)
                .unwrap();

        let json_cad = serde_json::to_string(&moneda_cad).unwrap();
        assert!(json_cad.contains("\"codigo\":\"CAD\""));
        assert!(json_cad.contains("\"nombre\":\"Dólar canadiense\""));
        assert!(json_cad.contains("\"simbolo\":\"$\""));
        assert!(json_cad.contains("\"decimales\":2"));
    }

    #[test]
    fn test_catalogo_inicia_con_monedas_de_fabrica() {
        let catalogo = CatalogoMonedas::nuevo();

        assert!(catalogo.obtener(&CodigoMoneda::USD).is_some());
        assert!(catalogo.obtener(&CodigoMoneda::VES).is_some());
        assert!(catalogo.obtener(&CodigoMoneda::EUR).is_some());
        assert!(catalogo.obtener(&CodigoMoneda::USDT).is_some());

        assert_eq!(catalogo.decimales(&CodigoMoneda::USD).unwrap(), 2);
    }

    #[test]
    fn test_catalogo_registra_y_recupera_moneda_personalizada() {
        let mut catalogo = CatalogoMonedas::nuevo();

        let moneda_custom =
            Moneda::nueva_propia("BTC", "Bitcoin".to_string(), "₿".to_string(), 8).unwrap();

        let codigo = moneda_custom.codigo.clone();
        catalogo.registrar_moneda(moneda_custom);

        let recuperada = catalogo.obtener(&codigo).unwrap();
        assert_eq!(recuperada.nombre, "Bitcoin");
        assert_eq!(catalogo.decimales(&codigo).unwrap(), 8);
    }

    #[test]
    fn test_catalogo_falla_al_pedir_decimales_de_moneda_inexistente() {
        let catalogo = CatalogoMonedas::nuevo();

        // Creamos un código que no hemos registrado en el catálogo
        let cod_fantasma = CodigoMoneda::parse("LTC").unwrap();

        let err = catalogo.decimales(&cod_fantasma).unwrap_err();
        assert_eq!(err.campo, "moneda_no_encontrada");
    }
}
