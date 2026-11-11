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


#[cfg(test)]
mod tests {
    use super::*;
    use crate::aplicacion::dto::UsuarioRegistradoDto;
    use std::cell::{Cell, RefCell};

    // -----------------------------------------
    // Dobles de prueba
    // -----------------------------------------

    struct ServicioFalso {
        resultado: Result<SesionDto, ErrorAutenticacion>,
        llamadas: Cell<u32>,
        entrada_recibida: RefCell<Option<IniciarSesionEntrada>>,
    }

    impl ServicioInicioSesion for ServicioFalso {
        fn iniciar_sesion(
            &self,
            entrada: &IniciarSesionEntrada,
        ) -> Result<SesionDto, ErrorAutenticacion> {
            self.llamadas.set(self.llamadas.get() + 1);

            self.entrada_recibida
                .replace(Some(entrada.clone()));

            self.resultado.clone()
        }
    }

    struct AlmacenFalso {
        resultado: Result<(), ErrorSesion>,
        llamadas: Cell<u32>,
        sesion_guardada: RefCell<Option<SesionDto>>,
    }

    impl AlmacenSesiones for AlmacenFalso {
        fn guardar_sesion(
            &self,
            sesion: &SesionDto,
        ) -> Result<(), ErrorSesion> {
            self.llamadas.set(self.llamadas.get() + 1);

            // Registramos qué intentó guardar el caso de uso.
            self.sesion_guardada
                .replace(Some(sesion.clone()));

            self.resultado.clone()
        }
    }

    // -----------------------------------------
    // Datos de prueba
    // -----------------------------------------

    fn entrada_valida() -> IniciarSesionEntrada {
        IniciarSesionEntrada {
            correo: "eduardo@example.com".to_string(),
            contrasena: "contrasena123".to_string(),
        }
    }

    fn sesion_valida() -> SesionDto {
        SesionDto {
            usuario: UsuarioRegistradoDto {
                id: "usuario-123".to_string(),
                nombre_usuario: "eduardo".to_string(),
                nombre_visible: "Eduardo".to_string(),
                correo: "eduardo@example.com".to_string(),
                fecha_alta: "2026-11-11T12:00:00Z".to_string(),
            },
            token: "token_de_prueba".to_string(),
            fecha_expiracion: "2026-11-11T13:00:00Z".to_string(),
        }
    }

    fn servicio_exitoso() -> ServicioFalso {
        ServicioFalso {
            resultado: Ok(sesion_valida()),
            llamadas: Cell::new(0),
            entrada_recibida: RefCell::new(None),
        }
    }

    fn almacen_exitoso() -> AlmacenFalso {
        AlmacenFalso {
            resultado: Ok(()),
            llamadas: Cell::new(0),
            sesion_guardada: RefCell::new(None),
        }
    }

    // -----------------------------------------
    // Pruebas
    // -----------------------------------------

    #[test]
    fn inicia_sesion_exitosamente() {
        let servicio = servicio_exitoso();
        let almacen = almacen_exitoso();
        let esperada = sesion_valida();

        let resultado = iniciar_sesion(
            &servicio,
            &almacen,
            entrada_valida(),
        );

        assert!(resultado.is_ok());
        assert!(resultado.unwrap() == esperada);

        assert_eq!(servicio.llamadas.get(), 1);
        assert_eq!(almacen.llamadas.get(), 1);
    }

    #[test]
    fn envia_credenciales_correctas_al_servicio() {
        let servicio = servicio_exitoso();
        let almacen = almacen_exitoso();
        let entrada = entrada_valida();

        iniciar_sesion(
            &servicio,
            &almacen,
            entrada.clone(),
        ).unwrap();

        assert!(
            *servicio.entrada_recibida.borrow()
                == Some(entrada)
        );
    }

    #[test]
    fn guarda_la_sesion_recibida_del_servidor() {
        let servicio = servicio_exitoso();
        let almacen = almacen_exitoso();
        let esperada = sesion_valida();

        iniciar_sesion(
            &servicio,
            &almacen,
            entrada_valida(),
        ).unwrap();

        assert!(
            *almacen.sesion_guardada.borrow()
                == Some(esperada)
        );
    }

    #[test]
    fn no_guarda_sesion_si_credenciales_son_invalidas() {
        let servicio = ServicioFalso {
            resultado: Err(
                ErrorAutenticacion::Validacion {
                    campo: "credenciales".to_string(),
                    mensaje: "Correo o contraseña incorrectos."
                        .to_string(),
                }
            ),
            llamadas: Cell::new(0),
            entrada_recibida: RefCell::new(None),
        };

        let almacen = almacen_exitoso();

        let resultado = iniciar_sesion(
            &servicio,
            &almacen,
            entrada_valida(),
        );

        assert!(matches!(
            resultado,
            Err(ErrorIniciarSesion::Autenticacion(_))
        ));

        assert_eq!(servicio.llamadas.get(), 1);
        assert_eq!(almacen.llamadas.get(), 0);
    }

    #[test]
    fn informa_error_si_no_puede_guardar_sesion() {
        let servicio = servicio_exitoso();

        let almacen = AlmacenFalso {
            resultado: Err(
                ErrorSesion::Almacenamiento(
                    "No se pudo guardar la sesión".to_string()
                )
            ),
            llamadas: Cell::new(0),
            sesion_guardada: RefCell::new(None),
        };

        let resultado = iniciar_sesion(
            &servicio,
            &almacen,
            entrada_valida(),
        );

        assert!(matches!(
            resultado,
            Err(ErrorIniciarSesion::Sesion(_))
        ));

        assert_eq!(servicio.llamadas.get(), 1);
        assert_eq!(almacen.llamadas.get(), 1);
    }
}