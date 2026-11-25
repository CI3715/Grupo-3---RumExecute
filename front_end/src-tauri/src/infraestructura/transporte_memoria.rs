use std::collections::{HashMap, HashSet};

use crate::aplicacion::puertos::sincronizacion::{
    ErrorTransporteSincronizacion, LoteSincronizacion, TransporteSincronizacion,
};

#[derive(Default)]
pub struct TransporteMemoria {
    lotes_publicados: Vec<LoteSincronizacion>,
    lotes_pendientes: HashMap<String, Vec<LoteSincronizacion>>,
    lotes_leidos: HashMap<String, HashSet<String>>,
    conflictos_por_simular: usize,
}

impl TransporteMemoria {
    pub fn nuevo() -> Self {
        Self::default()
    }

    pub fn simular_conflictos_de_publicacion(&mut self, cantidad: usize) {
        self.conflictos_por_simular = cantidad;
    }

    pub fn lotes_publicados(&self) -> &[LoteSincronizacion] {
        &self.lotes_publicados
    }
}

impl TransporteSincronizacion for TransporteMemoria {
    fn traer_lotes_no_leidos(
        &mut self,
        dispositivo_id: &str,
    ) -> Result<Vec<LoteSincronizacion>, ErrorTransporteSincronizacion> {
        let leidos = self.lotes_leidos.entry(dispositivo_id.to_string()).or_default();
        Ok(self
            .lotes_publicados
            .iter()
            .filter(|lote| {
                lote.dispositivo_origen != dispositivo_id && !leidos.contains(&lote.id)
            })
            .cloned()
            .collect())
    }

    fn enviar_lote(
        &mut self,
        dispositivo_id: &str,
        lote: LoteSincronizacion,
    ) -> Result<(), ErrorTransporteSincronizacion> {
        if lote.dispositivo_origen != dispositivo_id {
            return Err(ErrorTransporteSincronizacion::OperacionFallida);
        }

        let lotes = self.lotes_pendientes.entry(dispositivo_id.to_string()).or_default();
        if let Some(existente) = lotes.iter().find(|existente| existente.id == lote.id) {
            return if existente == &lote {
                Ok(())
            } else {
                Err(ErrorTransporteSincronizacion::OperacionFallida)
            };
        }
        lotes.push(lote);
        Ok(())
    }

    fn publicar_cambios(
        &mut self,
        dispositivo_id: &str,
    ) -> Result<(), ErrorTransporteSincronizacion> {
        if self.conflictos_por_simular > 0 {
            self.conflictos_por_simular -= 1;
            return Err(ErrorTransporteSincronizacion::ConflictoRemoto);
        }

        if let Some(lotes) = self.lotes_pendientes.remove(dispositivo_id) {
            self.lotes_publicados.extend(lotes);
        }
        Ok(())
    }

    fn confirmar_lotes_leidos(
        &mut self,
        dispositivo_id: &str,
        lotes: &[String],
    ) -> Result<(), ErrorTransporteSincronizacion> {
        let disponibles = self
            .lotes_publicados
            .iter()
            .map(|lote| lote.id.as_str())
            .collect::<HashSet<_>>();
        if lotes.iter().any(|id| !disponibles.contains(id.as_str())) {
            return Err(ErrorTransporteSincronizacion::OperacionFallida);
        }

        self.lotes_leidos
            .entry(dispositivo_id.to_string())
            .or_default()
            .extend(lotes.iter().cloned());
        Ok(())
    }

    fn descartar_cambios_no_publicados(
        &mut self,
        dispositivo_id: &str,
    ) -> Result<(), ErrorTransporteSincronizacion> {
        self.lotes_pendientes.remove(dispositivo_id);
        Ok(())
    }
}
