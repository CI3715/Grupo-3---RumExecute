use std::collections::HashMap;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use base64::engine::general_purpose::STANDARD as BASE64;
use base64::Engine;
use uuid::Uuid;

use crate::aplicacion::puertos::sincronizacion::{
    CredencialesGit, ErrorTransporteSincronizacion, LoteSincronizacion, TransporteSincronizacion,
};

const EXTENSION_LOTE: &str = "ccbatch";
const DIRECTORIO_LEIDOS: &str = ".leidos";

struct AutenticacionGit {
    usuario: String,
    token: String,
}

pub struct TransporteGit {
    remoto: String,
    checkout: PathBuf,
    autenticacion: Option<AutenticacionGit>,
    pendientes: HashMap<String, Vec<LoteSincronizacion>>,
}

impl TransporteGit {
    pub fn nuevo(
        remoto: impl Into<String>,
        checkout: impl Into<PathBuf>,
        credenciales: Option<CredencialesGit>,
    ) -> Self {
        Self {
            remoto: remoto.into(),
            checkout: checkout.into(),
            autenticacion: credenciales.map(|credenciales| AutenticacionGit {
                usuario: credenciales.usuario,
                token: credenciales.token,
            }),
            pendientes: HashMap::new(),
        }
    }

    fn validar_id(valor: &str) -> bool {
        !valor.is_empty()
            && valor
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-' || byte == b'_')
    }

    fn es_repositorio(&self) -> bool {
        self.checkout.join(".git").is_dir()
    }

    fn con_autenticacion(&self, comando: &mut Command) {
        comando.env("GIT_TERMINAL_PROMPT", "0");
        if let Some(autenticacion) = &self.autenticacion {
            if self.remoto.starts_with("https://") || self.remoto.starts_with("http://") {
                let credencial = format!("{}:{}", autenticacion.usuario, autenticacion.token);
                let header = format!("AUTHORIZATION: Basic {}", BASE64.encode(credencial));
                comando
                    .env("GIT_CONFIG_COUNT", "1")
                    .env("GIT_CONFIG_KEY_0", "http.extraheader")
                    .env("GIT_CONFIG_VALUE_0", header);
            }
        }
    }

    fn ejecutar_git(
        &self,
        directorio: &Path,
        argumentos: &[String],
    ) -> Result<Output, ErrorTransporteSincronizacion> {
        let mut comando = Command::new("git");
        comando.current_dir(directorio).args(argumentos);
        self.con_autenticacion(&mut comando);
        comando
            .output()
            .map_err(|_| ErrorTransporteSincronizacion::DestinoNoDisponible)
    }

    fn asegurar_checkout(&self) -> Result<(), ErrorTransporteSincronizacion> {
        if self.es_repositorio() {
            let salida = self.ejecutar_git(
                &self.checkout,
                &[
                    "remote".to_string(),
                    "set-url".to_string(),
                    "origin".to_string(),
                    self.remoto.clone(),
                ],
            )?;
            return self.validar_salida(salida, false);
        }

        if self.checkout.exists()
            && fs::read_dir(&self.checkout)
                .map_err(|_| ErrorTransporteSincronizacion::DestinoNoDisponible)?
                .next()
                .is_some()
        {
            return Err(ErrorTransporteSincronizacion::DestinoNoDisponible);
        }
        let padre = self
            .checkout
            .parent()
            .ok_or(ErrorTransporteSincronizacion::DestinoNoDisponible)?;
        fs::create_dir_all(padre)
            .map_err(|_| ErrorTransporteSincronizacion::DestinoNoDisponible)?;
        let salida = self.ejecutar_git(
            padre,
            &[
                "clone".to_string(),
                "--".to_string(),
                self.remoto.clone(),
                self.checkout.to_string_lossy().into_owned(),
            ],
        )?;
        self.validar_salida(salida, false)
    }

    fn actualizar_checkout(&self) -> Result<String, ErrorTransporteSincronizacion> {
        self.asegurar_checkout()?;
        let salida =
            self.ejecutar_git(&self.checkout, &["fetch".to_string(), "origin".to_string()])?;
        self.validar_salida(salida, false)?;
        let salida = self.ejecutar_git(
            &self.checkout,
            &["branch".to_string(), "--show-current".to_string()],
        )?;
        let rama_salida = salida.stdout.clone();
        self.validar_salida(salida, false)?;
        let rama = String::from_utf8_lossy(&rama_salida).trim().to_string();
        if rama.is_empty() {
            return Err(ErrorTransporteSincronizacion::DestinoNoDisponible);
        }

        let referencia = format!("origin/{rama}");
        let salida = self.ejecutar_git(
            &self.checkout,
            &["reset".to_string(), "--hard".to_string(), referencia],
        )?;
        self.validar_salida(salida, false)?;
        let salida =
            self.ejecutar_git(&self.checkout, &["clean".to_string(), "-fd".to_string()])?;
        self.validar_salida(salida, false)?;
        Ok(rama)
    }

    fn validar_salida(
        &self,
        salida: Output,
        push: bool,
    ) -> Result<(), ErrorTransporteSincronizacion> {
        if salida.status.success() {
            return Ok(());
        }
        let error = String::from_utf8_lossy(&salida.stderr).to_ascii_lowercase();
        if push
            && (error.contains("non-fast-forward")
                || error.contains("fetch first")
                || error.contains("[rejected]")
                || error.contains("failed to push some refs"))
        {
            return Err(ErrorTransporteSincronizacion::ConflictoRemoto);
        }
        if error.contains("authentication failed")
            || error.contains("invalid username or password")
            || error.contains("could not read username")
            || error.contains("access denied")
        {
            return Err(ErrorTransporteSincronizacion::CredencialesGitInvalidas);
        }
        if error.contains("could not resolve host")
            || error.contains("failed to connect")
            || error.contains("connection timed out")
            || error.contains("repository not found")
        {
            return Err(ErrorTransporteSincronizacion::DestinoNoDisponible);
        }
        Err(ErrorTransporteSincronizacion::OperacionFallida)
    }

    fn ruta_lote(&self, dispositivo_id: &str, lote_id: &str) -> PathBuf {
        self.checkout
            .join(dispositivo_id)
            .join(format!("{lote_id}.{EXTENSION_LOTE}"))
    }

    fn id_ya_leido(&self, dispositivo_id: &str, lote_id: &str) -> bool {
        self.checkout
            .join(dispositivo_id)
            .join(DIRECTORIO_LEIDOS)
            .join(lote_id)
            .is_file()
    }

    fn existe_lote(&self, lote_id: &str) -> Result<bool, ErrorTransporteSincronizacion> {
        for entrada in fs::read_dir(&self.checkout)
            .map_err(|_| ErrorTransporteSincronizacion::DestinoNoDisponible)?
        {
            let entrada =
                entrada.map_err(|_| ErrorTransporteSincronizacion::DestinoNoDisponible)?;
            if !entrada
                .file_type()
                .map_err(|_| ErrorTransporteSincronizacion::DestinoNoDisponible)?
                .is_dir()
            {
                continue;
            }
            if entrada
                .path()
                .join(format!("{lote_id}.{EXTENSION_LOTE}"))
                .is_file()
            {
                return Ok(true);
            }
        }
        Ok(false)
    }

    fn escribir_atomico(
        &self,
        ruta: &Path,
        contenido: &[u8],
    ) -> Result<(), ErrorTransporteSincronizacion> {
        let directorio = ruta
            .parent()
            .ok_or(ErrorTransporteSincronizacion::OperacionFallida)?;
        fs::create_dir_all(directorio)
            .map_err(|_| ErrorTransporteSincronizacion::DestinoNoDisponible)?;
        if ruta.exists() {
            return if fs::read(ruta)
                .map_err(|_| ErrorTransporteSincronizacion::DestinoNoDisponible)?
                == contenido
            {
                Ok(())
            } else {
                Err(ErrorTransporteSincronizacion::OperacionFallida)
            };
        }
        let temporal = directorio.join(format!(".tmp-{}", Uuid::new_v4()));
        let resultado = (|| {
            let mut archivo = OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&temporal)
                .map_err(|_| ErrorTransporteSincronizacion::DestinoNoDisponible)?;
            archivo
                .write_all(contenido)
                .and_then(|_| archivo.sync_all())
                .map_err(|_| ErrorTransporteSincronizacion::DestinoNoDisponible)?;
            match fs::rename(&temporal, ruta) {
                Ok(()) => Ok(()),
                Err(_) if ruta.exists() => {
                    if fs::read(ruta)
                        .map_err(|_| ErrorTransporteSincronizacion::DestinoNoDisponible)?
                        == contenido
                    {
                        Ok(())
                    } else {
                        Err(ErrorTransporteSincronizacion::OperacionFallida)
                    }
                }
                Err(_) => Err(ErrorTransporteSincronizacion::DestinoNoDisponible),
            }
        })();
        if resultado.is_err() {
            let _ = fs::remove_file(temporal);
        }
        resultado
    }
}

impl TransporteSincronizacion for TransporteGit {
    fn traer_lotes_no_leidos(
        &mut self,
        dispositivo_id: &str,
    ) -> Result<Vec<LoteSincronizacion>, ErrorTransporteSincronizacion> {
        if !Self::validar_id(dispositivo_id) {
            return Err(ErrorTransporteSincronizacion::OperacionFallida);
        }
        self.actualizar_checkout()?;
        let mut lotes = Vec::new();
        for dispositivo in fs::read_dir(&self.checkout)
            .map_err(|_| ErrorTransporteSincronizacion::DestinoNoDisponible)?
        {
            let dispositivo =
                dispositivo.map_err(|_| ErrorTransporteSincronizacion::DestinoNoDisponible)?;
            let origen = dispositivo.file_name().to_string_lossy().into_owned();
            if origen == dispositivo_id
                || !Self::validar_id(&origen)
                || !dispositivo
                    .file_type()
                    .map_err(|_| ErrorTransporteSincronizacion::DestinoNoDisponible)?
                    .is_dir()
            {
                continue;
            }
            for archivo in fs::read_dir(dispositivo.path())
                .map_err(|_| ErrorTransporteSincronizacion::DestinoNoDisponible)?
            {
                let archivo =
                    archivo.map_err(|_| ErrorTransporteSincronizacion::DestinoNoDisponible)?;
                let ruta = archivo.path();
                if ruta.extension().and_then(|ext| ext.to_str()) != Some(EXTENSION_LOTE) {
                    continue;
                }
                let lote_id = ruta
                    .file_stem()
                    .and_then(|id| id.to_str())
                    .ok_or(ErrorTransporteSincronizacion::OperacionFallida)?;
                if !Self::validar_id(lote_id) || self.id_ya_leido(dispositivo_id, lote_id) {
                    continue;
                }
                lotes.push(LoteSincronizacion {
                    id: lote_id.to_string(),
                    dispositivo_origen: origen.clone(),
                    contenido: fs::read(&ruta)
                        .map_err(|_| ErrorTransporteSincronizacion::DestinoNoDisponible)?,
                });
            }
        }
        lotes.sort_by(|a, b| (&a.dispositivo_origen, &a.id).cmp(&(&b.dispositivo_origen, &b.id)));
        Ok(lotes)
    }

    fn enviar_lote(
        &mut self,
        dispositivo_id: &str,
        lote: LoteSincronizacion,
    ) -> Result<(), ErrorTransporteSincronizacion> {
        if !Self::validar_id(dispositivo_id)
            || !Self::validar_id(&lote.id)
            || lote.dispositivo_origen != dispositivo_id
        {
            return Err(ErrorTransporteSincronizacion::OperacionFallida);
        }
        let pendientes = self
            .pendientes
            .entry(dispositivo_id.to_string())
            .or_default();
        if let Some(existente) = pendientes.iter().find(|actual| actual.id == lote.id) {
            return if existente == &lote {
                Ok(())
            } else {
                Err(ErrorTransporteSincronizacion::OperacionFallida)
            };
        }
        pendientes.push(lote);
        Ok(())
    }

    fn publicar_cambios(
        &mut self,
        dispositivo_id: &str,
    ) -> Result<(), ErrorTransporteSincronizacion> {
        if !Self::validar_id(dispositivo_id) {
            return Err(ErrorTransporteSincronizacion::OperacionFallida);
        }
        self.asegurar_checkout()?;
        let pendientes = self
            .pendientes
            .get(dispositivo_id)
            .cloned()
            .unwrap_or_default();
        for lote in &pendientes {
            self.escribir_atomico(&self.ruta_lote(dispositivo_id, &lote.id), &lote.contenido)?;
        }

        let rama = self.actualizar_rama_local()?;
        let salida =
            self.ejecutar_git(&self.checkout, &["add".to_string(), "--all".to_string()])?;
        self.validar_salida(salida, false)?;
        let cambios = self.ejecutar_git(
            &self.checkout,
            &[
                "diff".to_string(),
                "--cached".to_string(),
                "--quiet".to_string(),
            ],
        )?;
        if !cambios.status.success() {
            if cambios.status.code() != Some(1) {
                self.validar_salida(cambios, false)?;
            }
            let salida = self.ejecutar_git(
                &self.checkout,
                &[
                    "-c".to_string(),
                    "user.name=Cuentas Claras".to_string(),
                    "-c".to_string(),
                    "user.email=cuentas-claras@localhost".to_string(),
                    "commit".to_string(),
                    "-m".to_string(),
                    "Sincronizar copia personal".to_string(),
                ],
            )?;
            self.validar_salida(salida, false)?;
        }

        let salida = self.ejecutar_git(
            &self.checkout,
            &[
                "push".to_string(),
                "origin".to_string(),
                format!("HEAD:refs/heads/{rama}"),
            ],
        )?;
        self.validar_salida(salida, true)?;
        self.pendientes.remove(dispositivo_id);
        Ok(())
    }

    fn confirmar_lotes_leidos(
        &mut self,
        dispositivo_id: &str,
        lotes: &[String],
    ) -> Result<(), ErrorTransporteSincronizacion> {
        if !Self::validar_id(dispositivo_id) {
            return Err(ErrorTransporteSincronizacion::OperacionFallida);
        }
        for lote_id in lotes {
            if !Self::validar_id(lote_id) || !self.existe_lote(lote_id)? {
                return Err(ErrorTransporteSincronizacion::OperacionFallida);
            }
        }
        for lote_id in lotes {
            let marcador = self
                .checkout
                .join(dispositivo_id)
                .join(DIRECTORIO_LEIDOS)
                .join(lote_id);
            self.escribir_atomico(&marcador, b"leido")?;
        }
        Ok(())
    }

    fn descartar_cambios_no_publicados(
        &mut self,
        dispositivo_id: &str,
    ) -> Result<(), ErrorTransporteSincronizacion> {
        self.pendientes.remove(dispositivo_id);
        if !self.es_repositorio() {
            return Ok(());
        }
        let salida =
            self.ejecutar_git(&self.checkout, &["fetch".to_string(), "origin".to_string()])?;
        self.validar_salida(salida, false)?;
        let rama = self.actualizar_rama_local()?;
        let salida = self.ejecutar_git(
            &self.checkout,
            &[
                "reset".to_string(),
                "--hard".to_string(),
                format!("origin/{rama}"),
            ],
        )?;
        self.validar_salida(salida, false)?;
        let salida =
            self.ejecutar_git(&self.checkout, &["clean".to_string(), "-fd".to_string()])?;
        self.validar_salida(salida, false)
    }
}

impl TransporteGit {
    fn actualizar_rama_local(&self) -> Result<String, ErrorTransporteSincronizacion> {
        let salida = self.ejecutar_git(
            &self.checkout,
            &["branch".to_string(), "--show-current".to_string()],
        )?;
        let rama_salida = salida.stdout.clone();
        self.validar_salida(salida, false)?;
        let rama = String::from_utf8_lossy(&rama_salida).trim().to_string();
        if rama.is_empty() {
            Err(ErrorTransporteSincronizacion::DestinoNoDisponible)
        } else {
            Ok(rama)
        }
    }
}
