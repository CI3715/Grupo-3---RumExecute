use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConsultarPatrimonioEntrada {
    #[serde(default)]
    pub fecha_consulta: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct RegistrarTasaDto {
    pub origen: String,
    pub destino: String,
    pub valor: String,
    pub fecha_vigencia: String,
    pub fuente: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EliminarTasaDto {
    pub id_tasa: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CambiarMonedaBaseDto {
    pub nuevo_codigo: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct CrearMonedaDto {
    pub codigo: String,
    pub nombre: String,
    pub simbolo: String,
    pub decimales: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct ListarHistorialTasasEntrada {
    pub origen: String,
    pub destino: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ErrorValidacionDto {
    pub campo: String,
    pub mensaje: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct MonedaDto {
    pub codigo: String,
    pub nombre: String,
    pub simbolo: String,
    pub decimales: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct TasaCambioDto {
    pub id: String,
    pub origen: String,
    pub destino: String,
    pub valor: String,
    pub fecha_vigencia: String,
    pub fuente: String,
    pub eliminada: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct MonedaBaseDto {
    pub moneda_base: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ResultadoTasaDto {
    pub exito: bool,
    pub id_tasa: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ResultadoOperacionDto {
    pub exito: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ResultadoMonedaBaseDto {
    pub exito: bool,
    pub moneda_base: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ResultadoMonedaDto {
    pub exito: bool,
    pub codigo: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CuentaPatrimonioDto {
    pub id: String,
    pub nombre: String,
    pub tipo: String,
    pub saldo_original: String,
    pub moneda_original: String,
    pub tasa_aplicada: Option<String>,
    pub saldo_convertido: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SubtotalPatrimonioDto {
    pub moneda: String,
    pub total: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct TasaUsadaDto {
    pub moneda_origen: String,
    pub moneda_destino: String,
    pub valor: String,
    pub valor_registrado: String,
    pub fecha_vigencia: String,
    pub fuente: String,
    pub inversa: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct TasaFaltanteDto {
    pub moneda_origen: String,
    pub moneda_destino: String,
    pub fecha_requerida: String,
    pub mensaje: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct PatrimonioDto {
    pub fecha_consulta: String,
    pub moneda_base: String,
    pub cuentas: Vec<CuentaPatrimonioDto>,
    pub subtotales: Vec<SubtotalPatrimonioDto>,
    pub tasas_usadas: Vec<TasaUsadaDto>,
    pub total_consolidado: Option<String>,
    pub tasa_faltante: Option<TasaFaltanteDto>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CuentaPatrimonioProyeccion {
    pub id: String,
    pub nombre: String,
    pub tipo: String,
    pub saldo: Decimal,
    pub moneda: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TasaCambioProyeccion {
    pub id: String,
    pub origen: String,
    pub destino: String,
    pub valor: Decimal,
    pub fecha_vigencia: String,
    pub fuente: String,
    pub orden_registro: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MonedaBaseProyeccion {
    pub codigo: String,
    pub decimales: u32,
}
