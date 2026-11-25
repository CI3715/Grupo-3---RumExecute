use std::collections::HashSet;

use crate::aplicacion::puertos::{ErrorRepositorio, RepositorioLibro};
use crate::aplicacion::puertos::sincronizacion::RepositorioLibroSincronizable;
use crate::dominio::evento::EventoLibro;

// Se crea el struct
pub struct RepositorioLibroEnMemoria {
    eventos: Vec<EventoLibro>,
    ids_eventos_remotos: HashSet<String>,
    fecha_ultima_sincronizacion: Option<String>,
}

// Funcion para inicializar la lista de EventosLibro (Vec)
impl RepositorioLibroEnMemoria {
    pub fn nuevo() -> RepositorioLibroEnMemoria {
        RepositorioLibroEnMemoria {
            eventos: Vec::new(),
            ids_eventos_remotos: HashSet::new(),
            fecha_ultima_sincronizacion: None,
        }
    }

    pub fn fecha_ultima_sincronizacion(&self) -> Option<&str> {
        self.fecha_ultima_sincronizacion.as_deref()
    }
}

impl RepositorioLibro for RepositorioLibroEnMemoria {
    fn guardar(&mut self, evento: EventoLibro) -> Result<(), ErrorRepositorio> {
        if !self
            .eventos
            .iter()
            .any(|existente| existente.id() == evento.id())
        {
            self.eventos.push(evento);
        }
        Ok(())
    }

    fn listar_eventos(&self) -> Result<Vec<EventoLibro>, ErrorRepositorio> {
        Ok(self.eventos.clone())
    }
}

impl RepositorioLibroSincronizable for RepositorioLibroEnMemoria {
    fn fusionar_eventos_atomico(
        &mut self,
        entrantes: &[EventoLibro],
    ) -> Result<crate::dominio::evento::ResultadoFusion, ErrorRepositorio> {
        let resultado = EventoLibro::fusionar_con_reporte(&self.eventos, entrantes);
        self.eventos = resultado.eventos_resultantes.clone();
        Ok(resultado)
    }

    fn ids_eventos_remotos_conocidos(&self) -> Result<HashSet<String>, ErrorRepositorio> {
        Ok(self.ids_eventos_remotos.clone())
    }

    fn registrar_ids_eventos_remotos(
        &mut self,
        ids: &[String],
    ) -> Result<(), ErrorRepositorio> {
        self.ids_eventos_remotos.extend(ids.iter().cloned());
        Ok(())
    }

    fn registrar_sincronizacion_exitosa(
        &mut self,
        ids_eventos_remotos: &[String],
        fecha: &str,
    ) -> Result<(), ErrorRepositorio> {
        self.ids_eventos_remotos
            .extend(ids_eventos_remotos.iter().cloned());
        self.fecha_ultima_sincronizacion = Some(fecha.to_string());
        Ok(())
    }
}
