use crate::aplicacion::dto::UsuarioRegistradoDto;

use crate::aplicacion::puertos::{
    AlmacenSesiones,
    ErrorAutenticacion,
    ErrorSesion,
    ServicioPerfilPropio,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ErrorObtenerPerfilPropio {
    SesionNoIniciada,
    Almacenamiento(ErrorSesion),
    Autenticacion(ErrorAutenticacion),
}

impl From<ErrorSesion> for ErrorObtenerPerfilPropio {
    fn from(error: ErrorSesion) -> Self {
        Self::Almacenamiento(error)
    }
}

impl From<ErrorAutenticacion> for ErrorObtenerPerfilPropio {
    fn from(error: ErrorAutenticacion) -> Self {
        Self::Autenticacion(error)
    }
}

pub fn obtener_perfil_propio(
    almacen: &dyn AlmacenSesiones,
    servicio: &dyn ServicioPerfilPropio,
) -> Result<UsuarioRegistradoDto, ErrorObtenerPerfilPropio> {

    // 1. Recuperar la sesión local
    let sesion = almacen
        .obtener_sesion()?
        .ok_or(ErrorObtenerPerfilPropio::SesionNoIniciada)?;

    // 2. Consultar el perfil usando el token
    let usuario = servicio.obtener_perfil_propio(
        &sesion.token
    )?;

    // 3. Devolver los datos del usuario
    Ok(usuario)
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::aplicacion::dto::SesionDto;
    use std::cell::Cell;

    // -----------------------------------------
    // Dobles de prueba
    // -----------------------------------------

    struct AlmacenFalso {
        resultado: Result<Option<SesionDto>, ErrorSesion>,
    }

    impl AlmacenSesiones for AlmacenFalso {
        fn guardar_sesion(
            &self,
            _sesion: &SesionDto,
        ) -> Result<(), ErrorSesion> {
            Ok(())
        }

        fn obtener_sesion(
            &self,
        ) -> Result<Option<SesionDto>, ErrorSesion> {
            self.resultado.clone()
        }
    }

    struct ServicioFalso {
        resultado: Result<UsuarioRegistradoDto, ErrorAutenticacion>,
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
    fn obtiene_perfil_exitosamente() {
        let almacen = AlmacenFalso {
            resultado: Ok(Some(sesion_valida())),
        };

        let servicio = servicio_exitoso();

        let resultado = obtener_perfil_propio(
            &almacen,
            &servicio,
        );

        assert_eq!(resultado.unwrap(), usuario_valido());
        assert_eq!(servicio.llamadas.get(), 1);
    }

    #[test]
    fn rechaza_consulta_sin_sesion() {
        let almacen = AlmacenFalso {
            resultado: Ok(None),
        };

        let servicio = servicio_exitoso();

        let resultado = obtener_perfil_propio(
            &almacen,
            &servicio,
        );

        assert_eq!(
            resultado.unwrap_err(),
            ErrorObtenerPerfilPropio::SesionNoIniciada
        );

        assert_eq!(servicio.llamadas.get(), 0);
    }

    #[test]
    fn propaga_error_del_almacenamiento() {
        let almacen = AlmacenFalso {
            resultado: Err(
                ErrorSesion::Almacenamiento(
                    "Error al leer la sesión".to_string()
                )
            ),
        };

        let servicio = servicio_exitoso();

        let resultado = obtener_perfil_propio(
            &almacen,
            &servicio,
        );

        assert!(matches!(
            resultado,
            Err(ErrorObtenerPerfilPropio::Almacenamiento(_))
        ));

        assert_eq!(servicio.llamadas.get(), 0);
    }

    #[test]
    fn propaga_error_de_autenticacion() {
        let almacen = AlmacenFalso {
            resultado: Ok(Some(sesion_valida())),
        };

        let servicio = ServicioFalso {
            resultado: Err(
                ErrorAutenticacion::Validacion {
                    campo: "token".to_string(),
                    mensaje: "Token inválido".to_string(),
                }
            ),
            llamadas: Cell::new(0),
        };

        let resultado = obtener_perfil_propio(
            &almacen,
            &servicio,
        );

        assert!(matches!(
            resultado,
            Err(ErrorObtenerPerfilPropio::Autenticacion(_))
        ));

        assert_eq!(servicio.llamadas.get(), 1);
    }

    #[test]
    fn devuelve_datos_actualizados_del_servidor() {
        let almacen = AlmacenFalso {
            resultado: Ok(Some(sesion_valida())),
        };

        let mut usuario_actualizado = usuario_valido();
        usuario_actualizado.nombre_visible =
            "Eduardo Actualizado".to_string();

        let servicio = ServicioFalso {
            resultado: Ok(usuario_actualizado.clone()),
            llamadas: Cell::new(0),
        };

        let resultado = obtener_perfil_propio(
            &almacen,
            &servicio,
        );

        assert_eq!(
            resultado.unwrap().nombre_visible,
            "Eduardo Actualizado"
        );
    }
}