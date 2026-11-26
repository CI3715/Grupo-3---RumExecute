use std::collections::HashMap;
use std::fs::{self, OpenOptions};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

use uuid::Uuid;

use crate::aplicacion::puertos::sincronizacion::{
    ErrorTransporteSincronizacion, LoteSincronizacion, TransporteSincronizacion,
};

const EXTENSION_LOTE: &str = "ccbatch";
const DIRECTORIO_LEIDOS: &str = ".leidos";

pub struct TransporteCarpeta {
    raiz: PathBuf,
    pendientes: HashMap<String, Vec<LoteSincronizacion>>,
}

impl TransporteCarpeta {
    pub fn nuevo(raiz: impl Into<PathBuf>) -> Self {
        Self {
            raiz: raiz.into(),
            pendientes: HashMap::new(),
        }
    }

    fn directorio_dispositivo(&self, dispositivo_id: &str) -> PathBuf {
        self.raiz.join(dispositivo_id)
    }

    fn validar_id(valor: &str) -> bool {
        !valor.is_empty()
            && valor
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-' || byte == b'_')
    }

    fn id_ya_leido(&self, dispositivo_id: &str, lote_id: &str) -> bool {
        self.directorio_dispositivo(dispositivo_id)
            .join(DIRECTORIO_LEIDOS)
            .join(lote_id)
            .is_file()
    }

    fn escribir_atomico(ruta: &Path, contenido: &[u8]) -> Result<(), ErrorTransporteSincronizacion> {
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

    fn existe_lote(&self, lote_id: &str) -> Result<bool, ErrorTransporteSincronizacion> {
        let directorios = fs::read_dir(&self.raiz)
            .map_err(|_| ErrorTransporteSincronizacion::DestinoNoDisponible)?;
        for directorio in directorios {
            let directorio = directorio
                .map_err(|_| ErrorTransporteSincronizacion::DestinoNoDisponible)?;
            if !directorio
                .file_type()
                .map_err(|_| ErrorTransporteSincronizacion::DestinoNoDisponible)?
                .is_dir()
            {
                continue;
            }
            if directorio
                .path()
                .join(format!("{lote_id}.{EXTENSION_LOTE}"))
                .is_file()
            {
                return Ok(true);
            }
        }
        Ok(false)
    }
}

impl TransporteSincronizacion for TransporteCarpeta {
    fn traer_lotes_no_leidos(
        &mut self,
        dispositivo_id: &str,
    ) -> Result<Vec<LoteSincronizacion>, ErrorTransporteSincronizacion> {
        if !Self::validar_id(dispositivo_id) {
            return Err(ErrorTransporteSincronizacion::OperacionFallida);
        }
        let mut lotes = Vec::new();
        let directorios = fs::read_dir(&self.raiz)
            .map_err(|_| ErrorTransporteSincronizacion::DestinoNoDisponible)?;
        for directorio in directorios {
            let directorio = directorio
                .map_err(|_| ErrorTransporteSincronizacion::DestinoNoDisponible)?;
            let origen = directorio.file_name().to_string_lossy().into_owned();
            if origen == dispositivo_id
                || !Self::validar_id(&origen)
                || !directorio
                    .file_type()
                    .map_err(|_| ErrorTransporteSincronizacion::DestinoNoDisponible)?
                    .is_dir()
            {
                continue;
            }

            for archivo in fs::read_dir(directorio.path())
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
                let mut contenido = Vec::new();
                fs::File::open(&ruta)
                    .and_then(|mut archivo| archivo.read_to_end(&mut contenido))
                    .map_err(|_| ErrorTransporteSincronizacion::DestinoNoDisponible)?;
                lotes.push(LoteSincronizacion {
                    id: lote_id.to_string(),
                    dispositivo_origen: origen.clone(),
                    contenido,
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
        let pendientes = self.pendientes.entry(dispositivo_id.to_string()).or_default();
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
        let pendientes = self.pendientes.remove(dispositivo_id).unwrap_or_default();
        for lote in pendientes {
            let ruta = self
                .directorio_dispositivo(dispositivo_id)
                .join(format!("{}.{}", lote.id, EXTENSION_LOTE));
            Self::escribir_atomico(&ruta, &lote.contenido)?;
        }
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
                .directorio_dispositivo(dispositivo_id)
                .join(DIRECTORIO_LEIDOS)
                .join(lote_id);
            Self::escribir_atomico(&marcador, b"leido")?;
        }
        Ok(())
    }

    fn descartar_cambios_no_publicados(
        &mut self,
        dispositivo_id: &str,
    ) -> Result<(), ErrorTransporteSincronizacion> {
        self.pendientes.remove(dispositivo_id);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn publica_lotes_atomicos_y_persiste_confirmacion_de_lectura() {
        let raiz = std::env::temp_dir().join(format!("cc_transporte_{}", Uuid::new_v4()));
        let mut emisor = TransporteCarpeta::nuevo(raiz.clone());
        let mut receptor = TransporteCarpeta::nuevo(raiz.clone());
        let lote = LoteSincronizacion {
            id: "lote-1".to_string(),
            dispositivo_origen: "dispositivo-a".to_string(),
            contenido: b"lote cifrado".to_vec(),
        };
        emisor.enviar_lote("dispositivo-a", lote).unwrap();
        emisor.publicar_cambios("dispositivo-a").unwrap();

        let recibidos = receptor.traer_lotes_no_leidos("dispositivo-b").unwrap();
        assert_eq!(recibidos.len(), 1);
        assert_eq!(recibidos[0].contenido, b"lote cifrado");
        receptor
            .confirmar_lotes_leidos("dispositivo-b", &["lote-1".to_string()])
            .unwrap();
        assert!(receptor
            .traer_lotes_no_leidos("dispositivo-b")
            .unwrap()
            .is_empty());

        fs::remove_dir_all(raiz).unwrap();
    }

    #[test]
    fn conflicto_de_id_con_contenido_distinto_es_rechazado() {
        let raiz = std::env::temp_dir().join(format!("cc_transporte_{}", Uuid::new_v4()));
        let mut transporte = TransporteCarpeta::nuevo(raiz.clone());
        let lote = LoteSincronizacion {
            id: "lote-1".to_string(),
            dispositivo_origen: "dispositivo-a".to_string(),
            contenido: b"uno".to_vec(),
        };
        transporte.enviar_lote("dispositivo-a", lote.clone()).unwrap();
        transporte.publicar_cambios("dispositivo-a").unwrap();
        let mut distinto = lote;
        distinto.contenido = b"dos".to_vec();
        transporte.enviar_lote("dispositivo-a", distinto).unwrap();
        assert_eq!(
            transporte.publicar_cambios("dispositivo-a"),
            Err(ErrorTransporteSincronizacion::OperacionFallida)
        );
        fs::remove_dir_all(raiz).unwrap();
    }
}
