use crate::aplicacion::dto::UsuarioRegistradoDto;

use crate::aplicacion::puertos::{
    AlmacenSesiones,
    ErrorAutenticacion,
    ErrorSesion,
    ServicioPerfilPropio,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EstadoSesion {
    Invitado,
    Autenticado(UsuarioRegistradoDto),
    NoVerificada,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ErrorRestaurarSesion {
    Almacenamiento(ErrorSesion),
    Autenticacion(ErrorAutenticacion),
}

impl From<ErrorSesion> for ErrorRestaurarSesion {
    fn from(error: ErrorSesion) -> Self {
        Self::Almacenamiento(error)
    }
}

pub fn restaurar_sesion(
    almacen: &dyn AlmacenSesiones,
    servicio: &dyn ServicioPerfilPropio,
) -> Result<EstadoSesion, ErrorRestaurarSesion> {

    // 1. Consultar la sesión guardada
    let sesion = match almacen.obtener_sesion()? {
        Some(sesion) => sesion,
        None => return Ok(EstadoSesion::Invitado),
    };

    // 2. Validar la sesión consultando el perfil
    match servicio.obtener_perfil_propio(&sesion.token) {

        // El servidor reconoce al usuario
        Ok(usuario) => {
            Ok(EstadoSesion::Autenticado(usuario))
        }

        // El servidor rechaza el token
        Err(ErrorAutenticacion::SesionInvalida) => {
            almacen.eliminar_sesion()?;
            Ok(EstadoSesion::Invitado)
        }

        // No se pudo contactar con el servidor
        Err(ErrorAutenticacion::ServicioNoDisponible) => {
            Ok(EstadoSesion::NoVerificada)
        }

        // Cualquier otro error se propaga
        Err(error) => {
            Err(ErrorRestaurarSesion::Autenticacion(error))
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::aplicacion::dto::SesionDto;
    use std::cell::{Cell, RefCell};

    // -----------------------------------------
    // Dobles de prueba
    // -----------------------------------------

    struct AlmacenFalso {
        sesion: RefCell<Option<SesionDto>>,
        error_lectura: bool,
        error_eliminacion: bool,
        llamadas_eliminar: Cell<u32>,
    }

    impl AlmacenSesiones for AlmacenFalso {
        fn guardar_sesion(
            &self,
            sesion: &SesionDto,
        ) -> Result<(), ErrorSesion> {
            self.sesion.replace(Some(sesion.clone()));
            Ok(())
        }

        fn obtener_sesion(
            &self,
        ) -> Result<Option<SesionDto>, ErrorSesion> {
            if self.error_lectura {
                return Err(ErrorSesion::Almacenamiento(
                    "Error al leer la sesión".to_string(),
                ));
            }

            Ok(self.sesion.borrow().clone())
        }

        fn eliminar_sesion(
            &self,
        ) -> Result<(), ErrorSesion> {
            self.llamadas_eliminar
                .set(self.llamadas_eliminar.get() + 1);

            if self.error_eliminacion {
                return Err(ErrorSesion::Almacenamiento(
                    "Error al eliminar la sesión".to_string(),
                ));
            }

            self.sesion.replace(None);
            Ok(())
        }
    }

    struct ServicioFalso {
        resultado: Result<
            UsuarioRegistradoDto,
            ErrorAutenticacion,
        >,
        llamadas: Cell<u32>,
    }

    impl ServicioPerfilPropio for ServicioFalso {
        fn obtener_perfil_propio(
            &self,
            token: &str,
        ) -> Result<UsuarioRegistradoDto, ErrorAutenticacion> {
            assert_eq!(token, "token_de_prueba");

            self.llamadas.set(self.llamadas.get() + 1);
            self.resultado.clone()
        }
    }

    // -----------------------------------------
    // Datos de prueba
    // -----------------------------------------

    fn usuario_valido() -> UsuarioRegistradoDto {
        UsuarioRegistradoDto {
            id: "usuario-123".to_string(),
            nombre_usuario: "eduardo".to_string(),
            nombre_visible: "Eduardo".to_string(),
            correo: "eduardo@example.com".to_string(),
            fecha_alta: "2026-11-11T12:00:00Z".to_string(),
        }
    }

    fn sesion_valida() -> SesionDto {
        SesionDto {
            usuario: usuario_valido(),
            token: "token_de_prueba".to_string(),
            fecha_expiracion: "2026-11-11T13:00:00Z".to_string(),
        }
    }

    fn almacen_con_sesion() -> AlmacenFalso {
        AlmacenFalso {
            sesion: RefCell::new(Some(sesion_valida())),
            error_lectura: false,
            error_eliminacion: false,
            llamadas_eliminar: Cell::new(0),
        }
    }

    fn servicio_exitoso() -> ServicioFalso {
        ServicioFalso {
            resultado: Ok(usuario_valido()),
            llamadas: Cell::new(0),
        }
    }

    // -----------------------------------------
    // Pruebas
    // -----------------------------------------

    #[test]
    fn restaura_sesion_valida() {
        let almacen = almacen_con_sesion();
        let servicio = servicio_exitoso();

        let resultado = restaurar_sesion(
            &almacen,
            &servicio,
        );

        assert_eq!(
            resultado.unwrap(),
            EstadoSesion::Autenticado(usuario_valido())
        );

        assert_eq!(servicio.llamadas.get(), 1);
        assert_eq!(almacen.llamadas_eliminar.get(), 0);
    }

    #[test]
    fn sin_sesion_continua_como_invitado() {
        let almacen = AlmacenFalso {
            sesion: RefCell::new(None),
            error_lectura: false,
            error_eliminacion: false,
            llamadas_eliminar: Cell::new(0),
        };

        let servicio = servicio_exitoso();

        let resultado = restaurar_sesion(
            &almacen,
            &servicio,
        );

        assert_eq!(resultado.unwrap(), EstadoSesion::Invitado);
        assert_eq!(servicio.llamadas.get(), 0);
    }

    #[test]
    fn elimina_sesion_invalida() {
        let almacen = almacen_con_sesion();

        let servicio = ServicioFalso {
            resultado: Err(
                ErrorAutenticacion::SesionInvalida
            ),
            llamadas: Cell::new(0),
        };

        let resultado = restaurar_sesion(
            &almacen,
            &servicio,
        );

        assert_eq!(resultado.unwrap(), EstadoSesion::Invitado);
        assert_eq!(almacen.llamadas_eliminar.get(), 1);
        assert!(almacen.obtener_sesion().unwrap().is_none());
    }

    #[test]
    fn conserva_sesion_si_servidor_no_disponible() {
        let almacen = almacen_con_sesion();

        let servicio = ServicioFalso {
            resultado: Err(
                ErrorAutenticacion::ServicioNoDisponible
            ),
            llamadas: Cell::new(0),
        };

        let resultado = restaurar_sesion(
            &almacen,
            &servicio,
        );

        assert_eq!(
            resultado.unwrap(),
            EstadoSesion::NoVerificada
        );

        assert_eq!(almacen.llamadas_eliminar.get(), 0);
        assert!(almacen.obtener_sesion().unwrap().is_some());
    }

    #[test]
    fn propaga_error_de_lectura() {
        let mut almacen = almacen_con_sesion();
        almacen.error_lectura = true;

        let servicio = servicio_exitoso();

        let resultado = restaurar_sesion(
            &almacen,
            &servicio,
        );

        assert!(matches!(
            resultado,
            Err(ErrorRestaurarSesion::Almacenamiento(_))
        ));

        assert_eq!(servicio.llamadas.get(), 0);
    }

    #[test]
    fn propaga_error_al_eliminar_sesion_invalida() {
        let mut almacen = almacen_con_sesion();
        almacen.error_eliminacion = true;

        let servicio = ServicioFalso {
            resultado: Err(
                ErrorAutenticacion::SesionInvalida
            ),
            llamadas: Cell::new(0),
        };

        let resultado = restaurar_sesion(
            &almacen,
            &servicio,
        );

        assert!(matches!(
            resultado,
            Err(ErrorRestaurarSesion::Almacenamiento(_))
        ));

        assert_eq!(almacen.llamadas_eliminar.get(), 1);
    }

    #[test]
    fn propaga_error_tecnico_del_servicio() {
        let almacen = almacen_con_sesion();

        let servicio = ServicioFalso {
            resultado: Err(
                ErrorAutenticacion::ErrorTecnico(
                    "Respuesta inesperada".to_string(),
                )
            ),
            llamadas: Cell::new(0),
        };

        let resultado = restaurar_sesion(
            &almacen,
            &servicio,
        );

        assert!(matches!(
            resultado,
            Err(ErrorRestaurarSesion::Autenticacion(_))
        ));

        assert_eq!(almacen.llamadas_eliminar.get(), 0);
        assert!(almacen.obtener_sesion().unwrap().is_some());
    }

    #[test]
    fn devuelve_perfil_actualizado_del_servidor() {
        let almacen = almacen_con_sesion();

        let mut usuario_actualizado = usuario_valido();
        usuario_actualizado.nombre_visible =
            "Eduardo Actualizado".to_string();

        let servicio = ServicioFalso {
            resultado: Ok(usuario_actualizado.clone()),
            llamadas: Cell::new(0),
        };

        let resultado = restaurar_sesion(
            &almacen,
            &servicio,
        );

        assert_eq!(
            resultado.unwrap(),
            EstadoSesion::Autenticado(usuario_actualizado)
        );
    }
}