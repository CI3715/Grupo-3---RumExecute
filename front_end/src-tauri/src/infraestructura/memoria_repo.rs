use crate::dominio::evento::EventoLibro;
use crate::aplicacion::puertos::{ErrorRepositorio, RepositorioLibro};

// Se crea el struct
pub struct RepositorioLibroEnMemoria {
    eventos : Vec<EventoLibro>,
}

// Funcion para inicializar la lista de EventosLibro (Vec)
impl RepositorioLibroEnMemoria {
    pub fn nuevo () -> RepositorioLibroEnMemoria{
        RepositorioLibroEnMemoria { eventos: Vec::new() }
    }

} 

impl RepositorioLibro for RepositorioLibroEnMemoria {
    fn guardar(&mut self, evento: EventoLibro) -> Result<(), ErrorRepositorio> {
        self.eventos.push(evento);
        Ok(())
    }
    
    fn listar_eventos(&self) -> Result<Vec<EventoLibro>, ErrorRepositorio> {
        Ok(self.eventos.clone())
    }
}