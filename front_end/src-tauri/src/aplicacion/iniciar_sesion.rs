use crate::aplicacion::dto::{
    IniciarSesionEntrada,
    SesionDto,
};

use crate::aplicacion::puertos::{
    AlmacenSesiones,
    ErrorAutenticacion,
    ErrorSesion,
    ServicioInicioSesion,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ErrorIniciarSesion {
    Autenticacion(ErrorAutenticacion),
    Sesion(ErrorSesion),
}

impl From<ErrorAutenticacion> for ErrorIniciarSesion {
    fn from(error: ErrorAutenticacion) -> Self {
        Self::Autenticacion(error)
    }
}

impl From<ErrorSesion> for ErrorIniciarSesion {
    fn from(error: ErrorSesion) -> Self {
        Self::Sesion(error)
    }
}

pub fn iniciar_sesion(
    servicio: &dyn ServicioInicioSesion,
    almacen: &dyn AlmacenSesiones,
    entrada: IniciarSesionEntrada,
) -> Result<SesionDto, ErrorIniciarSesion> {

    // 1. Solicitar autenticación al backend
    let sesion = servicio.iniciar_sesion(&entrada)?;

    // 2. Guardar la sesión obtenida
    almacen.guardar_sesion(&sesion)?;

    // 3. Devolver la sesión
    Ok(sesion)
}