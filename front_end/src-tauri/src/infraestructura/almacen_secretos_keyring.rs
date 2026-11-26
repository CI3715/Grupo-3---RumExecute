use keyring::Entry;
use serde::{Deserialize, Serialize};

use crate::aplicacion::puertos::sincronizacion::{
    AlmacenSecretosSincronizacion, CredencialesGit, ErrorAlmacenSecretos,
};

const SERVICIO_KEYRING: &str = "cuentas_claras_sincronizacion";
const CLAVE_FRASE_PASO: &str = "frase_paso_sincronizacion";
const CLAVE_CREDENCIALES_GIT: &str = "credenciales_git";

#[derive(Serialize, Deserialize)]
struct CredencialesGitSerde {
    usuario: String,
    token: String,
}

/// Adaptador de `AlmacenSecretosSincronizacion` respaldado por el almacén de secretos del SO.
///
/// Utiliza el crate `keyring` (Windows Credential Manager / macOS Keychain / Linux Secret Service)
/// para almacenar la frase de paso y las credenciales de Git sin exponerlas en SQLite ni archivos.
#[derive(Debug, Clone)]
pub struct AlmacenSecretosKeyring {
    servicio: String,
}

impl Default for AlmacenSecretosKeyring {
    fn default() -> Self {
        Self::nuevo()
    }
}

impl AlmacenSecretosKeyring {
    pub fn nuevo() -> Self {
        Self {
            servicio: SERVICIO_KEYRING.to_string(),
        }
    }

    fn entrada(&self, clave: &str) -> Result<Entry, ErrorAlmacenSecretos> {
        Entry::new(&self.servicio, clave).map_err(|_| ErrorAlmacenSecretos::OperacionFallida)
    }
}

impl AlmacenSecretosSincronizacion for AlmacenSecretosKeyring {
    fn obtener_frase_paso(&self) -> Result<Option<String>, ErrorAlmacenSecretos> {
        let entrada = self.entrada(CLAVE_FRASE_PASO)?;
        match entrada.get_password() {
            Ok(valor) => Ok(Some(valor)),
            Err(keyring::Error::NoEntry) => Ok(None),
            Err(_) => Err(ErrorAlmacenSecretos::OperacionFallida),
        }
    }

    fn guardar_frase_paso(&mut self, frase_paso: &str) -> Result<(), ErrorAlmacenSecretos> {
        if frase_paso.is_empty() {
            return Err(ErrorAlmacenSecretos::OperacionFallida);
        }
        let entrada = self.entrada(CLAVE_FRASE_PASO)?;
        entrada
            .set_password(frase_paso)
            .map_err(|_| ErrorAlmacenSecretos::OperacionFallida)
    }

    fn obtener_credenciales_git(&self) -> Result<Option<CredencialesGit>, ErrorAlmacenSecretos> {
        let entrada = self.entrada(CLAVE_CREDENCIALES_GIT)?;
        let json = match entrada.get_password() {
            Ok(valor) => valor,
            Err(keyring::Error::NoEntry) => return Ok(None),
            Err(_) => return Err(ErrorAlmacenSecretos::OperacionFallida),
        };

        let datos: CredencialesGitSerde =
            serde_json::from_str(&json).map_err(|_| ErrorAlmacenSecretos::OperacionFallida)?;
        Ok(Some(CredencialesGit {
            usuario: datos.usuario,
            token: datos.token,
        }))
    }

    fn guardar_credenciales_git(
        &mut self,
        credenciales: &CredencialesGit,
    ) -> Result<(), ErrorAlmacenSecretos> {
        if credenciales.usuario.trim().is_empty() || credenciales.token.trim().is_empty() {
            return Err(ErrorAlmacenSecretos::OperacionFallida);
        }

        let payload = CredencialesGitSerde {
            usuario: credenciales.usuario.clone(),
            token: credenciales.token.clone(),
        };
        let json =
            serde_json::to_string(&payload).map_err(|_| ErrorAlmacenSecretos::OperacionFallida)?;

        let entrada = self.entrada(CLAVE_CREDENCIALES_GIT)?;
        entrada
            .set_password(&json)
            .map_err(|_| ErrorAlmacenSecretos::OperacionFallida)
    }

    fn eliminar_credenciales_git(&mut self) -> Result<(), ErrorAlmacenSecretos> {
        let entrada = self.entrada(CLAVE_CREDENCIALES_GIT)?;
        match entrada.delete_credential() {
            Ok(()) | Err(keyring::Error::NoEntry) => Ok(()),
            Err(_) => Err(ErrorAlmacenSecretos::OperacionFallida),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rechaza_frase_de_paso_vacia() {
        let mut almacen = AlmacenSecretosKeyring::nuevo();
        let err = almacen.guardar_frase_paso("").unwrap_err();
        assert_eq!(err, ErrorAlmacenSecretos::OperacionFallida);
    }

    #[test]
    fn rechaza_credenciales_git_vacias() {
        let mut almacen = AlmacenSecretosKeyring::nuevo();
        let err = almacen
            .guardar_credenciales_git(&CredencialesGit {
                usuario: "   ".to_string(),
                token: "ghp_123".to_string(),
            })
            .unwrap_err();
        assert_eq!(err, ErrorAlmacenSecretos::OperacionFallida);
    }
}
