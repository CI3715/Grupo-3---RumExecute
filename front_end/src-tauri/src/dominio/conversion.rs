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
        return Ok(monto.clone());
    }

    // Busca la tasa vigente en el historial (RF-CAM-03, RF-CAM-04)
    let tasa_vigente = historial.obtener_tasa_vigente(
        monto.moneda,
        moneda_objetivo,
        fecha_consulta
    );

    match tasa_vigente {
        Some(TasaAplicable::Directa(tasa)) => {
            // De moneda A a moneda B. Monto en A * Tasa = Monto en B
            let mut nueva_cantidad = monto.cantidad * tasa.valor;
            nueva_cantidad = nueva_cantidad.round_dp(moneda_objetivo.decimales());

            Ok(Monto::new(nueva_cantidad, moneda_objetivo))
        }
        Some(TasaAplicable::Inversa(tasa)) => {
            // De moneda B a moneda A. Monto en B / Tasa = Monto en A
            let mut nueva_cantidad = monto
                .cantidad
                .checked_div(tasa.valor)
                .ok_or_else(|| ErrorDominio::tasa_invalida("Error matemático: división por cero al aplicar tasa inversa"))?;
                // NUNCA debería dar este error porque ninguna tasa es igual a 0, pero se toma por si acaso

            nueva_cantidad = nueva_cantidad.round_dp(moneda_objetivo.decimales());

            Ok(Monto::new(nueva_cantidad, moneda_objetivo))
        }
        None => {
            Err(ErrorDominio::tasa_no_existente(monto.moneda.as_str(), moneda_objetivo.as_str(), fecha_consulta))
        }
    }
}