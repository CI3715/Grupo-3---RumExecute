use crate::aplicacion::puertos::{
    AlmacenSesiones,
    ErrorSesion,
};

pub fn cerrar_sesion(
    almacen: &dyn AlmacenSesiones,
) -> Result<(), ErrorSesion> {
    almacen.eliminar_sesion()
}


#[cfg(test)]
mod tests {
    use super::*;

    use crate::aplicacion::dto::{
        SesionDto,
        UsuarioRegistradoDto,
    };

    use std::cell::{Cell, RefCell};

    // -----------------------------------------
    // Doble de prueba
    // -----------------------------------------

    struct AlmacenFalso {
        sesion: RefCell<Option<SesionDto>>,
        llamadas_eliminar: Cell<u32>,
        fallar_al_eliminar: bool,
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
            Ok(self.sesion.borrow().clone())
        }

        fn eliminar_sesion(
            &self,
        ) -> Result<(), ErrorSesion> {
            self.llamadas_eliminar
                .set(self.llamadas_eliminar.get() + 1);

            if self.fallar_al_eliminar {
                return Err(
                    ErrorSesion::Almacenamiento(
                        "No se pudo eliminar la sesión".to_string()
                    )
                );
            }

            self.sesion.replace(None);
            Ok(())
        }
    }

    // -----------------------------------------
    // Datos de prueba
    // -----------------------------------------

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

    fn almacen_con_sesion() -> AlmacenFalso {
        AlmacenFalso {
            sesion: RefCell::new(Some(sesion_valida())),
            llamadas_eliminar: Cell::new(0),
            fallar_al_eliminar: false,
        }
    }

    // -----------------------------------------
    // Pruebas
    // -----------------------------------------

    #[test]
    fn cierra_sesion_exitosamente() {
        let almacen = almacen_con_sesion();

        let resultado = cerrar_sesion(&almacen);

        assert!(resultado.is_ok());
        assert_eq!(almacen.llamadas_eliminar.get(), 1);
    }

    #[test]
    fn elimina_la_sesion_almacenada() {
        let almacen = almacen_con_sesion();

        assert!(almacen.obtener_sesion().unwrap().is_some());

        cerrar_sesion(&almacen).unwrap();

        assert!(almacen.obtener_sesion().unwrap().is_none());
    }

    #[test]
    fn cerrar_sesion_sin_sesion_es_exitoso() {
        let almacen = AlmacenFalso {
            sesion: RefCell::new(None),
            llamadas_eliminar: Cell::new(0),
            fallar_al_eliminar: false,
        };

        let resultado = cerrar_sesion(&almacen);

        assert!(resultado.is_ok());
        assert!(almacen.obtener_sesion().unwrap().is_none());
    }

    #[test]
    fn cerrar_sesion_es_idempotente() {
        let almacen = almacen_con_sesion();

        let primero = cerrar_sesion(&almacen);
        let segundo = cerrar_sesion(&almacen);

        assert!(primero.is_ok());
        assert!(segundo.is_ok());

        assert_eq!(almacen.llamadas_eliminar.get(), 2);
        assert!(almacen.obtener_sesion().unwrap().is_none());
    }

    #[test]
    fn propaga_error_de_almacenamiento() {
        let almacen = AlmacenFalso {
            sesion: RefCell::new(Some(sesion_valida())),
            llamadas_eliminar: Cell::new(0),
            fallar_al_eliminar: true,
        };

        let resultado = cerrar_sesion(&almacen);

        assert!(matches!(
            resultado,
            Err(ErrorSesion::Almacenamiento(_))
        ));

        assert_eq!(almacen.llamadas_eliminar.get(), 1);

        // El doble simula que la operación fallida
        // no eliminó la sesión.
        assert!(almacen.obtener_sesion().unwrap().is_some());
    }
}