use std::collections::BTreeMap;

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

    // Decimal expresado como cadena.
    pub monto: String,

    pub tipo_division: TipoDivision,

    pub participantes_ids: Vec<String>,

    // usuario_id -> porcentaje o monto fijo.
    // None cuando la división es IGUALES.
    pub valores: Option<BTreeMap<String, String>>,
}


// Cuota calculada por el backend.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CuotaCompartidaDto {
    #[serde(alias = "usuario_id")]
    pub usuario_id: String,

    #[serde(alias = "nombre_usuario")]
    pub nombre_usuario: String,

    pub monto: String,

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

    pub monto: String,

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

    // Decimal expresado como cadena.
    // Puede ser positivo, negativo o cero.
    pub balance: String,
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
    use serde_json::{from_value, json, to_value};

    #[test]
    fn serializar_gasto_igualitario() {
        let entrada = RegistrarGastoCompartidoEntrada {
            concepto: "Cena".to_string(),
            pagador_id: "usuario-1".to_string(),
            monto: "10.00".to_string(),
            tipo_division: TipoDivision::Iguales,
            participantes_ids: vec![
                "usuario-1".to_string(),
                "usuario-2".to_string(),
            ],
            valores: None,
        };

        let json = to_value(entrada).unwrap();

        assert_eq!(json["concepto"], "Cena");
        assert_eq!(json["pagadorId"], "usuario-1");
        assert_eq!(json["monto"], "10.00");
        assert_eq!(json["tipoDivision"], "IGUALES");
        assert_eq!(json["participantesIds"][0], "usuario-1");
        assert!(json["valores"].is_null());
    }

    #[test]
    fn serializar_gasto_por_porcentajes() {
        let valores = BTreeMap::from([
            ("usuario-1".to_string(), "60".to_string()),
            ("usuario-2".to_string(), "40".to_string()),
        ]);

        let entrada = RegistrarGastoCompartidoEntrada {
            concepto: "Taxi".to_string(),
            pagador_id: "usuario-1".to_string(),
            monto: "100.00".to_string(),
            tipo_division: TipoDivision::Porcentajes,
            participantes_ids: vec![
                "usuario-1".to_string(),
                "usuario-2".to_string(),
            ],
            valores: Some(valores),
        };

        let json = to_value(entrada).unwrap();

        assert_eq!(json["tipoDivision"], "PORCENTAJES");
        assert_eq!(json["valores"]["usuario-1"], "60");
        assert_eq!(json["valores"]["usuario-2"], "40");
    }

    #[test]
    fn serializar_gasto_por_montos() {
        let entrada = RegistrarGastoCompartidoEntrada {
            concepto: "Hotel".to_string(),
            pagador_id: "usuario-1".to_string(),
            monto: "40.00".to_string(),
            tipo_division: TipoDivision::Montos,
            participantes_ids: vec![
                "usuario-1".to_string(),
                "usuario-2".to_string(),
            ],
            valores: Some(BTreeMap::from([
                ("usuario-1".to_string(), "15.00".to_string()),
                ("usuario-2".to_string(), "25.00".to_string()),
            ])),
        };

        let json = to_value(entrada).unwrap();

        assert_eq!(json["tipoDivision"], "MONTOS");
        assert_eq!(json["valores"]["usuario-1"], "15.00");
        assert_eq!(json["valores"]["usuario-2"], "25.00");
    }

    #[test]
    fn deserializar_gasto_registrado() {
        let json = json!({
            "id": "gasto-1",
            "grupo_id": "grupo-1",
            "concepto": "Cena",
            "pagador_id": "usuario-1",
            "monto": "10.00",
            "fecha": "2026-11-12T12:00:00+00:00",
            "cuotas": [
                {
                    "usuario_id": "usuario-1",
                    "nombre_usuario": "ana",
                    "monto": "5.00",
                    "lleva_residuo": false
                },
                {
                    "usuario_id": "usuario-2",
                    "nombre_usuario": "beto",
                    "monto": "5.00",
                    "lleva_residuo": false
                }
            ]
        });

        let gasto: GastoCompartidoDto =
            from_value(json).unwrap();

        assert_eq!(gasto.id, "gasto-1");
        assert_eq!(gasto.pagador_id, "usuario-1");
        assert_eq!(gasto.monto, "10.00");
        assert_eq!(gasto.cuotas.len(), 2);
        assert_eq!(gasto.cuotas[0].nombre_usuario, "ana");
    }

    #[test]
    fn deserializar_detalle_grupo() {
        let json = json!({
            "id": "grupo-1",
            "nombre_grupo": "Viaje",
            "moneda": "USD",
            "descripcion": "Vacaciones",
            "administrador_id": "usuario-1",
            "miembros": [
                {
                    "usuario_id": "usuario-1",
                    "nombre_usuario": "ana",
                    "nombre_visible": "Ana",
                    "es_administrador": true,
                    "balance": "5.00"
                },
                {
                    "usuario_id": "usuario-2",
                    "nombre_usuario": "beto",
                    "nombre_visible": "Beto",
                    "es_administrador": false,
                    "balance": "-5.00"
                }
            ],
            "gastos": []
        });

        let detalle: DetalleGrupoDto =
            from_value(json).unwrap();

        assert_eq!(detalle.id, "grupo-1");
        assert_eq!(detalle.nombre_grupo, "Viaje");
        assert_eq!(detalle.miembros.len(), 2);
        assert!(detalle.miembros[0].es_administrador);
        assert_eq!(detalle.miembros[0].balance, "5.00");
        assert_eq!(detalle.miembros[1].balance, "-5.00");
        assert!(detalle.gastos.is_empty());
    }

    #[test]
    fn deserializar_grupo_sin_gastos() {
        let json = json!({
            "id": "grupo-1",
            "nombre_grupo": "Viaje",
            "moneda": "USD",
            "descripcion": "",
            "administrador_id": "usuario-1",
            "miembros": [
                {
                    "usuario_id": "usuario-1",
                    "nombre_usuario": "ana",
                    "nombre_visible": "Ana",
                    "es_administrador": true,
                    "balance": "0"
                }
            ],
            "gastos": []
        });

        let detalle: DetalleGrupoDto =
            from_value(json).unwrap();

        assert!(detalle.gastos.is_empty());
        assert_eq!(detalle.miembros[0].balance, "0");
    }

    #[test]
    fn no_expone_datos_privados() {
        let miembro = MiembroGrupoDto {
            usuario_id: "usuario-1".to_string(),
            nombre_usuario: "ana".to_string(),
            nombre_visible: "Ana".to_string(),
            es_administrador: true,
            balance: "0.00".to_string(),
        };

        let json = to_value(miembro).unwrap();

        assert!(json.get("correo").is_none());
        assert!(json.get("contrasena").is_none());
        assert!(json.get("token").is_none());
    }
}
