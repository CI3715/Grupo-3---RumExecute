use std::collections::BTreeMap;

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};


// Modalidades de división admitidas por el backend.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TipoDivision {
    Iguales,
    Porcentajes,
    Montos,
}


// POST /grupos/{id}/gastos
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RegistrarGastoCompartidoEntrada {
    pub concepto: String,

    pub pagador_id: String,

    pub monto: Decimal,

    pub tipo_division: TipoDivision,

    pub participantes_ids: Vec<String>,

    // usuario_id -> porcentaje o monto fijo.
    // None cuando la división es IGUALES.
    pub valores: Option<BTreeMap<String, Decimal>>,
}


// Cuota calculada por el backend.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CuotaCompartidaDto {
    #[serde(alias = "usuario_id")]
    pub usuario_id: String,

    #[serde(alias = "nombre_usuario")]
    pub nombre_usuario: String,

    pub monto: Decimal,

    #[serde(alias = "lleva_residuo")]
    pub lleva_residuo: bool,
}


// Respuesta de POST /grupos/{id}/gastos.
// También se utiliza para los gastos de GET /grupos/{id}.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GastoCompartidoDto {
    pub id: String,

    #[serde(alias = "grupo_id")]
    pub grupo_id: String,

    pub concepto: String,

    #[serde(alias = "pagador_id")]
    pub pagador_id: String,

    pub monto: Decimal,

    // Fecha ISO 8601, por ejemplo 2026-11-12T12:00:00+00:00.
    pub fecha: String,

    pub cuotas: Vec<CuotaCompartidaDto>,
}


// Miembro del grupo con su balance.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MiembroGrupoDto {
    #[serde(alias = "usuario_id")]
    pub usuario_id: String,

    #[serde(alias = "nombre_usuario")]
    pub nombre_usuario: String,

    #[serde(alias = "nombre_visible")]
    pub nombre_visible: String,

    #[serde(alias = "es_administrador")]
    pub es_administrador: bool,

    pub balance: Decimal,
}


// GET /grupos/{id}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DetalleGrupoDto {
    pub id: String,

    #[serde(alias = "nombre_grupo")]
    pub nombre_grupo: String,

    pub moneda: String,

    pub descripcion: String,

    #[serde(alias = "administrador_id")]
    pub administrador_id: String,

    pub miembros: Vec<MiembroGrupoDto>,

    pub gastos: Vec<GastoCompartidoDto>,
}




#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    fn decimal(valor: &str) -> Decimal {
        Decimal::from_str(valor).unwrap()
    }

    #[test]
    fn serializa_monto_como_string() {
        let entrada = RegistrarGastoCompartidoEntrada {
            concepto: "Cena".into(),
            pagador_id: "usuario-1".into(),
            monto: decimal("30.50"),
            tipo_division: TipoDivision::Iguales,
            participantes_ids: vec![
                "usuario-1".into(),
                "usuario-2".into(),
            ],
            valores: None,
        };

        let json = serde_json::to_value(&entrada).unwrap();

        assert_eq!(json["monto"], "30.50");
        assert_eq!(json["tipoDivision"], "IGUALES");
        assert_eq!(json["pagadorId"], "usuario-1");
        assert!(json["valores"].is_null());
    }

    #[test]
    fn serializa_porcentajes_como_strings() {
        let entrada = RegistrarGastoCompartidoEntrada {
            concepto: "Cena".into(),
            pagador_id: "usuario-1".into(),
            monto: decimal("100.00"),
            tipo_division: TipoDivision::Porcentajes,
            participantes_ids: vec![
                "usuario-1".into(),
                "usuario-2".into(),
            ],
            valores: Some(BTreeMap::from([
                ("usuario-1".into(), decimal("60.5")),
                ("usuario-2".into(), decimal("39.5")),
            ])),
        };

        let json = serde_json::to_value(&entrada).unwrap();

        assert_eq!(json["valores"]["usuario-1"], "60.5");
        assert_eq!(json["valores"]["usuario-2"], "39.5");
    }

    #[test]
    fn deserializa_monto_como_decimal() {
        let json = r#"{
            "id": "gasto-1",
            "grupo_id": "grupo-1",
            "concepto": "Cena",
            "pagador_id": "usuario-1",
            "monto": "30.50",
            "fecha": "2026-11-12T12:00:00Z",
            "cuotas": []
        }"#;

        let gasto: GastoCompartidoDto =
            serde_json::from_str(json).unwrap();

        assert_eq!(gasto.monto, decimal("30.50"));
        assert_eq!(gasto.grupo_id, "grupo-1");
    }

    #[test]
    fn deserializa_balance_negativo() {
        let json = r#"{
            "usuario_id": "usuario-2",
            "nombre_usuario": "beto",
            "nombre_visible": "Beto",
            "es_administrador": false,
            "balance": "-15.25"
        }"#;

        let miembro: MiembroGrupoDto =
            serde_json::from_str(json).unwrap();

        assert_eq!(miembro.balance, decimal("-15.25"));
    }

    #[test]
    fn mantiene_precision_decimal() {
        let a = decimal("0.1");
        let b = decimal("0.2");

        assert_eq!(a + b, decimal("0.3"));
    }
}
