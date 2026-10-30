use crate::dominio::errores::ErrorDominio;
use crate::dominio::moneda::CodigoMoneda;
use crate::dominio::monto::Monto;
use crate::dominio::tasa_cambio::{HistorialTasas, TasaAplicable};

pub fn convertir_monto(
    monto: &Monto,
    moneda_objetivo: CodigoMoneda,
    fecha_consulta: &str,
    historial: &HistorialTasas,
) -> Result<Monto, ErrorDominio> {
    // Si las monedas son iguales, no hay conversión que hacer
    if monto.moneda == moneda_objetivo {
        return Ok(*monto);
    }

    // Busca la tasa vigente en el historial (RF-CAM-03, RF-CAM-04)
    let tasa_vigente =
        historial.obtener_tasa_vigente(monto.moneda, moneda_objetivo, fecha_consulta);

    match tasa_vigente {
        Some(TasaAplicable::Directa(tasa)) => {
            // De moneda A a moneda B. Monto en A * Tasa = Monto en B
            let mut nueva_cantidad = monto.cantidad * tasa.valor;
            nueva_cantidad = nueva_cantidad.round_dp(moneda_objetivo.decimales());

            Ok(Monto::new(nueva_cantidad, moneda_objetivo))
        }
        Some(TasaAplicable::Inversa(tasa)) => {
            // De moneda B a moneda A. Monto en B / Tasa = Monto en A
            let mut nueva_cantidad = monto.cantidad.checked_div(tasa.valor).ok_or_else(|| {
                ErrorDominio::tasa_invalida(
                    "Error matemático: división por cero al aplicar tasa inversa",
                )
            })?;
            // NUNCA debería dar este error porque ninguna tasa es igual a 0, pero se toma por si acaso

            nueva_cantidad = nueva_cantidad.round_dp(moneda_objetivo.decimales());

            Ok(Monto::new(nueva_cantidad, moneda_objetivo))
        }
        None => Err(ErrorDominio::tasa_no_existente(
            monto.moneda.as_str(),
            moneda_objetivo.as_str(),
            fecha_consulta,
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dominio::moneda::CodigoMoneda;
    use crate::dominio::monto::Monto;
    use crate::dominio::tasa_cambio::{HistorialTasas, TasaCambio};
    use rust_decimal::Decimal;
    use std::str::FromStr;

    const MOMENTO_ACTUAL: &str = "2026-10-30T12:00:00.000Z";

    #[test]
    fn test_convertir_misma_moneda_retorna_monto_identico() {
        let historial = HistorialTasas::nuevo();
        let monto = Monto::new(Decimal::from_str("100.00").unwrap(), CodigoMoneda::USD);

        let resultado = convertir_monto(&monto, CodigoMoneda::USD, "2026-10-28", &historial)
            .expect("Convertir a la misma moneda debe ser exitoso");

        assert_eq!(resultado.cantidad, Decimal::from_str("100.00").unwrap());
        assert_eq!(resultado.moneda, CodigoMoneda::USD);
    }

    #[test]
    fn test_convertir_monto_tasa_directa_exitosa() {
        let mut historial = HistorialTasas::nuevo();
        let tasa = TasaCambio::nueva(
            CodigoMoneda::USD,
            CodigoMoneda::VES,
            Decimal::from_str("890").unwrap(),
            "2026-10-28",
            "BCV",
            MOMENTO_ACTUAL,
        )
        .unwrap();
        historial.registrar(tasa);

        // Convertir 10 USD a VES (10 * 890 = 8,900.00 VES)
        let monto_usd = Monto::new(Decimal::from_str("10").unwrap(), CodigoMoneda::USD);
        let resultado = convertir_monto(&monto_usd, CodigoMoneda::VES, "2026-10-29", &historial)
            .expect("Debería convertir usando la tasa directa");

        assert_eq!(resultado.cantidad, Decimal::from_str("8900").unwrap());
        assert_eq!(resultado.moneda, CodigoMoneda::VES);
    }

    #[test]
    fn test_convertir_monto_tasa_inversa_exitosa() {
        let mut historial = HistorialTasas::nuevo();
        // Solo existe la tasa USD -> VES (890)
        let tasa = TasaCambio::nueva(
            CodigoMoneda::USD,
            CodigoMoneda::VES,
            Decimal::from_str("890").unwrap(),
            "2026-10-28",
            "BCV",
            MOMENTO_ACTUAL,
        )
        .unwrap();
        historial.registrar(tasa);

        // Convertir 365 VES a USD (365 / 890 = 0.41 USD)
        let monto_ves = Monto::new(Decimal::from_str("365").unwrap(), CodigoMoneda::VES);
        let resultado = convertir_monto(&monto_ves, CodigoMoneda::USD, "2026-10-29", &historial)
            .expect("Debería convertir usando la tasa inversa");

        assert_eq!(resultado.cantidad, Decimal::from_str("0.41").unwrap());
        assert_eq!(resultado.moneda, CodigoMoneda::USD);
    }

    #[test]
    fn test_convertir_monto_redondea_a_decimales_de_moneda_objetivo() {
        let mut historial = HistorialTasas::nuevo();
        // Tasa EUR/USD = 1.0845
        let tasa = TasaCambio::nueva(
            CodigoMoneda::EUR,
            CodigoMoneda::USD,
            Decimal::from_str("1.0845").unwrap(),
            "2026-10-28",
            "ECB",
            MOMENTO_ACTUAL,
        )
        .unwrap();
        historial.registrar(tasa);

        // 10.36 EUR * 1.0845 = 11.23542 USD. Como USD admite 2 decimales, se redondea a 11.24 USD.
        let monto_eur = Monto::new(Decimal::from_str("10.36").unwrap(), CodigoMoneda::EUR);
        let resultado = convertir_monto(&monto_eur, CodigoMoneda::USD, "2026-10-28", &historial)
            .expect("Debería redondear según los decimales de USD");

        assert_eq!(resultado.cantidad, Decimal::from_str("11.24").unwrap());
        assert_eq!(resultado.moneda, CodigoMoneda::USD);
    }

    #[test]
    fn test_convertir_monto_falla_si_falta_tasa_aplicable() {
        let historial = HistorialTasas::nuevo();
        let monto_usd = Monto::new(Decimal::from_str("50.00").unwrap(), CodigoMoneda::USD);

        // Intentar convertir USD a VES sin tasas registradas
        let err = convertir_monto(&monto_usd, CodigoMoneda::VES, "2026-10-28", &historial)
            .expect_err("Debería fallar al no existir tasa aplicable");

        assert_eq!(err.campo, "tasa_no_existente");
        assert!(err.mensaje.contains("USD"));
        assert!(err.mensaje.contains("VES"));
        assert!(err.mensaje.contains("2026-10-28"));
    }

    #[test]
    fn test_convertir_monto_falla_si_tasa_es_posterior_a_fecha_consulta() {
        let mut historial = HistorialTasas::nuevo();
        // Tasa registrada el 2026-10-28
        let tasa = TasaCambio::nueva(
            CodigoMoneda::USD,
            CodigoMoneda::VES,
            Decimal::from_str("36.5").unwrap(),
            "2026-10-28",
            "BCV",
            MOMENTO_ACTUAL,
        )
        .unwrap();
        historial.registrar(tasa);

        // Consultamos con fecha 2026-10-20 (anterior a la tasa disponible)
        let monto_usd = Monto::new(Decimal::from_str("10.00").unwrap(), CodigoMoneda::USD);
        let err = convertir_monto(&monto_usd, CodigoMoneda::VES, "2026-10-20", &historial)
            .expect_err(
                "Debería fallar porque la tasa disponible es futura respecto a la consulta",
            );

        assert_eq!(err.campo, "tasa_no_existente");
    }
}
