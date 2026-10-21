use crate::dominio::evento::EventoLibro; // Importa el tipo EventoLibro del módulo dominio

#[derive(Debug)]
pub struct ErrorRepositorio(pub String);

pub trait RepositorioLibro {
    // Agrega un evento al libro
    fn guardar(&mut self, evento: EventoLibro) -> Result<(), ErrorRepositorio>;
    // Devuelve todos los eventos, en orden
    fn listar_eventos(&self) -> Result<Vec<EventoLibro>, ErrorRepositorio>;
}

pub trait Reloj {
    fn ahora(&self) -> String; // Devuelve un string con la fecha y hora actual (revisar que tipo le pone Jean)
}

pub trait GeneradorId {
    fn nuevo_id(&mut self) -> String; // Genera un id unico
}
