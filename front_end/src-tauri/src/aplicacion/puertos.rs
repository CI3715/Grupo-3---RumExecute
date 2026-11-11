use crate::dominio::evento::EventoLibro; // Importa el tipo EventoLibro del módulo dominio
use crate::aplicacion::dto::{
    RegistrarUsuarioEntrada,
    UsuarioRegistradoDto,
    IniciarSesionEntrada,
    SesionDto,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ErrorAutenticacion {
    Validacion {
        campo: String,
        mensaje: String,
    },
    UsuarioYaExistente {
        campo: String,
    },
    ServicioNoDisponible,
    ErrorTecnico(String),
    SesionInvalida,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ErrorSesion {
    Almacenamiento(String),
}

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

pub trait ServicioAutenticacion {
    fn registrar_usuario(
        &self,
        entrada: &RegistrarUsuarioEntrada,
    ) -> Result<UsuarioRegistradoDto, ErrorAutenticacion>;
}

pub trait ServicioInicioSesion {
    fn iniciar_sesion(
        &self,
        entrada: &IniciarSesionEntrada,
    ) -> Result<SesionDto, ErrorAutenticacion>;
}

pub trait AlmacenSesiones {
    fn guardar_sesion(
        &self,
        sesion: &SesionDto,
    ) -> Result<(), ErrorSesion>;

    fn obtener_sesion(
        &self,
    ) -> Result<Option<SesionDto>, ErrorSesion>;

    fn eliminar_sesion(
        &self,
    ) -> Result<(), ErrorSesion>;
}

pub trait ServicioPerfilPropio {
    fn obtener_perfil_propio(
        &self,
        token: &str,
    ) -> Result<UsuarioRegistradoDto, ErrorAutenticacion>;
}