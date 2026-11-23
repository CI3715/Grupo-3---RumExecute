use std::collections::{BTreeMap, HashSet};

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use super::errores::ErrorDominio;

/// Tipo de división de un gasto compartido.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TipoDivisionPendiente {
    Iguales,
    Porcentajes,
    Montos,
}

/// Gasto compartido preparado localmente para enviarse al servidor.
///
/// La clave de idempotencia se conserva durante todos los reintentos.
/// No contiene información de las cuentas personales.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GastoPendiente {
    pub idempotency_key: String,
    pub grupo_id: String,
    pub concepto: String,
    pub pagador_id: String,

    #[serde(with = "rust_decimal::serde::str")]
    pub monto: Decimal,

    pub tipo_division: TipoDivisionPendiente,
    pub participantes_ids: Vec<String>,

    // Valores exactos de la división. El adaptador HTTP
    // los convertirá a cadenas decimales.
    pub valores: BTreeMap<String, Decimal>,

    pub fecha_creacion: DateTime<Utc>,
}

impl GastoPendiente {
    #[allow(clippy::too_many_arguments)]
    pub fn nuevo(
        idempotency_key: impl Into<String>,
        grupo_id: impl Into<String>,
        concepto: impl Into<String>,
        pagador_id: impl Into<String>,
        monto: Decimal,
        tipo_division: TipoDivisionPendiente,
        participantes_ids: Vec<String>,
        valores: BTreeMap<String, Decimal>,
        fecha_creacion: DateTime<Utc>,
    ) -> Result<Self, ErrorDominio> {
        let gasto = Self {
            idempotency_key: idempotency_key.into(),
            grupo_id: grupo_id.into(),
            concepto: concepto.into().trim().to_string(),
            pagador_id: pagador_id.into(),
            monto,
            tipo_division,
            participantes_ids,
            valores,
            fecha_creacion,
        };

        gasto.validar()?;
        Ok(gasto)
    }

    /// Valida que el gasto sea consistente antes de guardarlo
    /// en la cola local o de intentar enviarlo.
    pub fn validar(&self) -> Result<(), ErrorDominio> {
        for (campo, valor) in [
            ("idempotency_key", self.idempotency_key.as_str()),
            ("grupo_id", self.grupo_id.as_str()),
            ("concepto", self.concepto.as_str()),
            ("pagador_id", self.pagador_id.as_str()),
        ] {
            if valor.trim().is_empty() {
                return Err(ErrorDominio::new(
                    campo,
                    format!("El campo '{campo}' no puede estar vacío."),
                ));
            }
        }

        if self.monto <= Decimal::ZERO
            || self.monto.normalize().scale() > 2
        {
            return Err(ErrorDominio::new(
                "monto",
                "El monto debe ser positivo y admitir como máximo dos decimales.",
            ));
        }

        if self.participantes_ids.is_empty() {
            return Err(ErrorDominio::new(
                "participantes_ids",
                "Debe haber al menos un participante.",
            ));
        }

        let mut participantes = HashSet::new();

        for id in &self.participantes_ids {
            if id.trim().is_empty()
                || !participantes.insert(id.as_str())
            {
                return Err(ErrorDominio::new(
                    "participantes_ids",
                    "Los participantes deben tener identificadores válidos y únicos.",
                ));
            }
        }

        match self.tipo_division {
            TipoDivisionPendiente::Iguales => {
                if !self.valores.is_empty() {
                    return Err(ErrorDominio::new(
                        "valores",
                        "La división en partes iguales no requiere valores.",
                    ));
                }
            }

            TipoDivisionPendiente::Porcentajes
            | TipoDivisionPendiente::Montos => {
                if self.valores.len() != participantes.len()
                    || !self.valores.keys().all(
                        |id| participantes.contains(id.as_str())
                    )
                {
                    return Err(ErrorDominio::new(
                        "valores",
                        "Debe indicarse un valor para cada participante.",
                    ));
                }

                if self.valores.values().any(|v| {
                    *v < Decimal::ZERO
                        || v.normalize().scale() > 2
                }) {
                    return Err(ErrorDominio::new(
                        "valores",
                        "Los valores deben ser no negativos y tener máximo dos decimales.",
                    ));
                }

                let suma: Decimal = self.valores.values().copied().sum();

                let total_esperado = match self.tipo_division {
                    TipoDivisionPendiente::Porcentajes => {
                        Decimal::new(100, 0)
                    }
                    TipoDivisionPendiente::Montos => self.monto,
                    TipoDivisionPendiente::Iguales => unreachable!(),
                };

                if suma != total_esperado {
                    return Err(ErrorDominio::new(
                        "valores",
                        "La suma de los valores no coincide con el total requerido.",
                    ));
                }
            }
        }

        Ok(())
    }

    /// Un reintento conserva la clave y todos los datos originales.
    pub fn preparar_reintento(&self) -> Self {
        self.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    fn fecha() -> DateTime<Utc> {
        Utc.with_ymd_and_hms(2026, 11, 22, 12, 0, 0)
            .single()
            .unwrap()
    }

    fn gasto_valido() -> GastoPendiente {
        GastoPendiente::nuevo(
            "clave-123",
            "grupo-1",
            "Hotel",
            "usuario-1",
            Decimal::new(10000, 2),
            TipoDivisionPendiente::Iguales,
            vec![
                "usuario-1".into(),
                "usuario-2".into(),
            ],
            BTreeMap::new(),
            fecha(),
        )
        .unwrap()
    }

    #[test]
    fn crea_gasto_pendiente_valido() {
        let gasto = gasto_valido();

        assert_eq!(gasto.idempotency_key, "clave-123");
        assert_eq!(gasto.grupo_id, "grupo-1");
        assert_eq!(gasto.monto, Decimal::new(10000, 2));
        assert_eq!(gasto.participantes_ids.len(), 2);
    }

    #[test]
    fn reintento_conserva_clave_y_datos() {
        let original = gasto_valido();
        let reintento = original.preparar_reintento();

        assert_eq!(
            original.idempotency_key,
            reintento.idempotency_key
        );

        assert_eq!(original, reintento);
    }

    #[test]
    fn rechaza_clave_vacia() {
        let mut gasto = gasto_valido();
        gasto.idempotency_key = "   ".into();

        let error = gasto.validar().unwrap_err();
        assert_eq!(error.campo, "idempotency_key");
    }

    #[test]
    fn rechaza_monto_invalido() {
        let mut gasto = gasto_valido();
        gasto.monto = Decimal::ZERO;

        assert_eq!(
            gasto.validar().unwrap_err().campo,
            "monto"
        );

        gasto.monto = Decimal::new(12345, 3);

        assert_eq!(
            gasto.validar().unwrap_err().campo,
            "monto"
        );
    }

    #[test]
    fn rechaza_participantes_repetidos() {
        let mut gasto = gasto_valido();

        gasto.participantes_ids = vec![
            "usuario-1".into(),
            "usuario-1".into(),
        ];

        assert_eq!(
            gasto.validar().unwrap_err().campo,
            "participantes_ids"
        );
    }

    #[test]
    fn acepta_porcentajes_que_suman_cien() {
        let mut valores = BTreeMap::new();
        valores.insert("usuario-1".into(), Decimal::new(60, 0));
        valores.insert("usuario-2".into(), Decimal::new(40, 0));

        let gasto = GastoPendiente::nuevo(
            "clave-456",
            "grupo-1",
            "Cena",
            "usuario-1",
            Decimal::new(5000, 2),
            TipoDivisionPendiente::Porcentajes,
            vec![
                "usuario-1".into(),
                "usuario-2".into(),
            ],
            valores,
            fecha(),
        );

        assert!(gasto.is_ok());
    }

    #[test]
    fn rechaza_porcentajes_que_no_suman_cien() {
        let mut gasto = gasto_valido();
        gasto.tipo_division = TipoDivisionPendiente::Porcentajes;

        gasto.valores.insert(
            "usuario-1".into(),
            Decimal::new(50, 0),
        );
        gasto.valores.insert(
            "usuario-2".into(),
            Decimal::new(30, 0),
        );

        assert_eq!(
            gasto.validar().unwrap_err().campo,
            "valores"
        );
    }

    #[test]
    fn acepta_montos_que_suman_total() {
        let mut gasto = gasto_valido();
        gasto.tipo_division = TipoDivisionPendiente::Montos;

        gasto.valores.insert(
            "usuario-1".into(),
            Decimal::new(6000, 2),
        );
        gasto.valores.insert(
            "usuario-2".into(),
            Decimal::new(4000, 2),
        );

        assert!(gasto.validar().is_ok());
    }

    #[test]
    fn serializa_y_deserializa_gasto() {
        let gasto = gasto_valido();

        let json = serde_json::to_string(&gasto).unwrap();

        assert!(json.contains("\"monto\":\"100.00\""));

        let recuperado: GastoPendiente =
            serde_json::from_str(&json).unwrap();

        recuperado.validar().unwrap();

        assert_eq!(recuperado, gasto);
    }

    #[test]
    fn rechaza_valores_con_participante_ajeno() {
        let mut gasto = gasto_valido();
        gasto.tipo_division = TipoDivisionPendiente::Montos;

        gasto.valores.insert(
            "usuario-1".into(),
            Decimal::new(6000, 2),
        );
        gasto.valores.insert(
            "usuario-3".into(),
            Decimal::new(4000, 2),
        );

        assert_eq!(
            gasto.validar().unwrap_err().campo,
            "valores"
        );
    }
}
