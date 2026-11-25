use serde::{Deserialize, Serialize};
use super::errores::ErrorDominio;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct DispositivoId(String);

impl DispositivoId {
    pub fn nuevo(valor: impl Into<String>) -> Result<Self, ErrorDominio> {
        let limpio = valor.into().trim().to_string();
        if limpio.is_empty() {
            return Err(ErrorDominio::dispositivo_id_invalido(
                "El identificador del dispositivo no puede estar vacío.",
            ));
        }
        Ok(Self(limpio))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<DispositivoId> for String {
    fn from(id: DispositivoId) -> Self {
        id.0
    }
}