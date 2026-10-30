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
            return Err(ErrorDominio::new(
                "valor",
                "El valor de la tasa de cambio debe ser mayor a cero.",
            ));
        }

        if valor.normalize().scale() > 8 {
            return Err(ErrorDominio::new(
                "valor",
                "La tasa de cambio no puede tener más de 8 decimales.",
            ));
        }

        validar_fecha_no_futura(fecha, momento_actual_utc)?;

        if fuente.trim().is_empty() {
            return Err(ErrorDominio::new(
                "fuente",
                "Debe especificar la fuente de la tasa de cambio.",
            ));
        }

        Ok(Self {
            par,
            valor,
            fecha: fecha.trim().to_string(),
            fuente: fuente.trim().to_string(),
        })
    }
}

pub enum TasaAplicable<'a> {
    Directa(&'a TasaCambio),
    Inversa(&'a TasaCambio),
}

// Colección de dominio para gestionar el historial de tasas de cambio.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct HistorialTasas {
    tasas: Vec<TasaCambio>,
}

impl HistorialTasas {
    pub fn nuevo() -> Self {
        Self { tasas: Vec::new() }
    }

    pub fn registrar(&mut self, tasa: TasaCambio) {
        self.tasas.push(tasa);
    }

    ///Busca la tasa aplicable para un par de monedas en una fecha dada.
    pub fn obtener_tasa_vigente<'a>(
        &'a self,
        origen: CodigoMoneda,
        destino: CodigoMoneda,
        fecha_consulta: &str,
    ) -> Option<TasaAplicable<'a>> {
        let mut mejor_tasa: Option<TasaAplicable> = None;
        let mut mejor_fecha = "";

        for tasa in &self.tasas {
            if tasa.fecha.as_str() <= fecha_consulta && tasa.fecha.as_str() >= mejor_fecha {
                if tasa.par.origen == origen && tasa.par.destino == destino {
                    mejor_fecha = tasa.fecha.as_str();
                    mejor_tasa = Some(TasaAplicable::Directa(tasa));
                } else if tasa.par.origen == destino && tasa.par.destino == origen {
                    mejor_fecha = tasa.fecha.as_str();
                    mejor_tasa = Some(TasaAplicable::Inversa(tasa));
                }
            }
        }

        mejor_tasa
    }

    pub fn eliminar_tasa(&mut self, par: ParMonedas, fecha: &str, fuente: &str) -> bool {
        let len_inicial = self.tasas.len();
        self.tasas
            .retain(|t| !(t.par == par && t.fecha == fecha && t.fuente == fuente));
        self.tasas.len() < len_inicial
    }

    /// Obtiene todo el historial de tasas para un par determinado (Regla 5 / RF-CAM-06).
    pub fn consultar_historial_por_par(&self, par: ParMonedas) -> Vec<TasaCambio> {
        self.tasas
            .iter()
            .filter(|t| t.par == par)
            .cloned()
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal::Decimal;
    use std::str::FromStr;

    // Constantes auxiliares para las pruebas
    const MOMENTO_ACTUAL: &str = "2026-10-30T12:00:00.000Z";

    #[test]
    fn test_par_monedas_exitoso() {
        let par = ParMonedas::nuevo(CodigoMoneda::USD, CodigoMoneda::VES)
            .expect("Debería crear un par válido");
        assert_eq!(par.origen, CodigoMoneda::USD);
        assert_eq!(par.destino, CodigoMoneda::VES);
    }

    #[test]
    fn test_par_monedas_rechaza_monedas_iguales() {
        let err = ParMonedas::nuevo(CodigoMoneda::USD, CodigoMoneda::USD)
            .expect_err("Debería rechazar origen y destino iguales");
        assert_eq!(err.campo, "par_monedas");
    }

    #[test]
    fn test_tasa_cambio_creacion_exitosa() {
        let valor = Decimal::from_str("36.5").unwrap();
        let tasa = TasaCambio::nueva(
            CodigoMoneda::USD,
            CodigoMoneda::VES,
            valor,
            "2026-10-28",
            "BCV",
            MOMENTO_ACTUAL,
        )
        .expect("Debería crear la tasa correctamente");

        assert_eq!(tasa.par.origen, CodigoMoneda::USD);
        assert_eq!(tasa.valor, valor);
        assert_eq!(tasa.fecha, "2026-10-28");
        assert_eq!(tasa.fuente, "BCV");
    }

    #[test]
    fn test_tasa_cambio_rechaza_valor_cero_o_negativo() {
        let valor_cero = Decimal::ZERO;
        let err_cero = TasaCambio::nueva(
            CodigoMoneda::USD,
            CodigoMoneda::VES,
            valor_cero,
            "2026-10-28",
            "BCV",
            MOMENTO_ACTUAL,
        )
        .unwrap_err();
        assert_eq!(err_cero.campo, "valor");

        let valor_neg = Decimal::from_str("-10.5").unwrap();
        let err_neg = TasaCambio::nueva(
            CodigoMoneda::USD,
            CodigoMoneda::VES,
            valor_neg,
            "2026-10-28",
            "BCV",
            MOMENTO_ACTUAL,
        )
        .unwrap_err();
        assert_eq!(err_neg.campo, "valor");
    }

    #[test]
    fn test_tasa_cambio_rechaza_exceso_de_decimales() {
        let valor_excesivo = Decimal::from_str("36.123456789").unwrap(); // 9 decimales
        let err = TasaCambio::nueva(
            CodigoMoneda::USD,
            CodigoMoneda::VES,
            valor_excesivo,
            "2026-10-28",
            "BCV",
            MOMENTO_ACTUAL,
        )
        .unwrap_err();
        assert!(err.mensaje.contains("8 decimales"));
    }

    #[test]
    fn test_tasa_cambio_rechaza_fuente_vacia() {
        let valor = Decimal::from_str("36.5").unwrap();
        let err = TasaCambio::nueva(
            CodigoMoneda::USD,
            CodigoMoneda::VES,
            valor,
            "2026-10-28",
            "   ",
            MOMENTO_ACTUAL,
        )
        .unwrap_err();
        assert!(err.mensaje.contains("fuente"));
    }

    #[test]
    fn test_tasa_cambio_rechaza_fecha_futura() {
        let valor = Decimal::from_str("36.5").unwrap();
        let err = TasaCambio::nueva(
            CodigoMoneda::USD,
            CodigoMoneda::VES,
            valor,
            "2026-10-31", // Posterior al MOMENTO_ACTUAL elegido (30 de octubre de 2026)
            "BCV",
            MOMENTO_ACTUAL,
        )
        .unwrap_err();
        assert_eq!(err.campo, "fecha");
    }

    #[test]
    fn test_historial_tasas_obtener_tasa_vigente_directa_mas_reciente() {
        let mut historial = HistorialTasas::nuevo();

        let tasa_vieja = TasaCambio::nueva(
            CodigoMoneda::USD,
            CodigoMoneda::VES,
            Decimal::from_str("30.0").unwrap(),
            "2026-10-25",
            "BCV",
            MOMENTO_ACTUAL,
        )
        .unwrap();
        let tasa_nueva = TasaCambio::nueva(
            CodigoMoneda::USD,
            CodigoMoneda::VES,
            Decimal::from_str("35.0").unwrap(),
            "2026-10-28",
            "BCV",
            MOMENTO_ACTUAL,
        )
        .unwrap();

        historial.registrar(tasa_vieja);
        historial.registrar(tasa_nueva);

        let encontrada = historial
            .obtener_tasa_vigente(CodigoMoneda::USD, CodigoMoneda::VES, "2026-10-29")
            .unwrap();

        match encontrada {
            TasaAplicable::Directa(t) => {
                assert_eq!(t.fecha, "2026-10-28");
                assert_eq!(t.valor, Decimal::from_str("35.0").unwrap());
            }
            _ => panic!("Debería ser una tasa directa"),
        }
    }

    #[test]
    fn test_historial_tasas_desempate_por_registro_mas_reciente_misma_fecha() {
        let mut historial = HistorialTasas::nuevo();

        // Se registran dos tasas el mismo día (ej. apertura y cierre)
        let tasa_manana = TasaCambio::nueva(
            CodigoMoneda::USD,
            CodigoMoneda::VES,
            Decimal::from_str("36.0").unwrap(),
            "2026-10-28",
            "BCV_Manana",
            MOMENTO_ACTUAL,
        )
        .unwrap();
        let tasa_tarde = TasaCambio::nueva(
            CodigoMoneda::USD,
            CodigoMoneda::VES,
            Decimal::from_str("36.5").unwrap(),
            "2026-10-28",
            "BCV_Tarde",
            MOMENTO_ACTUAL,
        )
        .unwrap();

        historial.registrar(tasa_manana);
        historial.registrar(tasa_tarde);

        let encontrada = historial
            .obtener_tasa_vigente(CodigoMoneda::USD, CodigoMoneda::VES, "2026-10-28")
            .unwrap();

        match encontrada {
            TasaAplicable::Directa(t) => {
                assert_eq!(
                    t.fuente, "BCV_Tarde",
                    "Debería tomar la última registrada en esa fecha"
                );
            }
            _ => panic!("Debería ser una tasa directa"),
        }
    }

    #[test]
    fn test_historial_tasas_obtener_tasa_inversa() {
        let mut historial = HistorialTasas::nuevo();

        // Registramos USD -> VES
        let tasa_base = TasaCambio::nueva(
            CodigoMoneda::USD,
            CodigoMoneda::VES,
            Decimal::from_str("36.5").unwrap(),
            "2026-10-28",
            "BCV",
            MOMENTO_ACTUAL,
        )
        .unwrap();
        historial.registrar(tasa_base);

        // Consultamos VES -> USD (Debe devolver la inversa automáticamente)
        let encontrada = historial
            .obtener_tasa_vigente(CodigoMoneda::VES, CodigoMoneda::USD, "2026-10-29")
            .unwrap();

        match encontrada {
            TasaAplicable::Inversa(t) => {
                assert_eq!(t.par.origen, CodigoMoneda::USD);
                assert_eq!(t.par.destino, CodigoMoneda::VES);
                assert_eq!(t.valor, Decimal::from_str("36.5").unwrap());
            }
            _ => panic!("Debería identificar que es una tasa inversa"),
        }
    }

    #[test]
    fn test_historial_tasas_eliminar_tasa_existente() {
        let mut historial = HistorialTasas::nuevo();
        let par = ParMonedas::nuevo(CodigoMoneda::USD, CodigoMoneda::VES).unwrap();

        let tasa = TasaCambio::nueva(
            CodigoMoneda::USD,
            CodigoMoneda::VES,
            Decimal::from_str("36.5").unwrap(),
            "2026-10-28",
            "ErrorHumano",
            MOMENTO_ACTUAL,
        )
        .unwrap();
        historial.registrar(tasa);

        assert_eq!(historial.tasas.len(), 1);

        let eliminada = historial.eliminar_tasa(par, "2026-10-28", "ErrorHumano");
        assert!(eliminada);
        assert_eq!(historial.tasas.len(), 0);
    }
}
