use crate::aplicacion::puertos::{
    AlmacenSesiones,
    ErrorSesion,
};
use crate::aplicacion::puertos::servicio_social::{
    ErrorServicioSocial,
    ServicioSocial,
};


pub struct EliminarAmistad<'a> {
    servicio_social: &'a dyn ServicioSocial,
    almacen_sesiones: &'a dyn AlmacenSesiones,
}


impl<'a> EliminarAmistad<'a> {
    pub fn new(
        servicio_social: &'a dyn ServicioSocial,
        almacen_sesiones: &'a dyn AlmacenSesiones,
    ) -> Self {
        Self {
            servicio_social,
            almacen_sesiones,
        }
    }

    pub fn ejecutar(
        &self,
        amistad_id: &str,
    ) -> Result<(), ErrorServicioSocial> {

        // Recuperar la sesión local.
        let sesion = self
            .almacen_sesiones
            .obtener_sesion()
            .map_err(|error| match error {
                ErrorSesion::Almacenamiento(mensaje) => {
                    ErrorServicioSocial::ErrorTecnico(mensaje)
                }
            })?
            .ok_or(ErrorServicioSocial::NoAutenticado)?;

        // Validar el identificador.
        if amistad_id.trim().is_empty() {
            return Err(ErrorServicioSocial::Validacion {
                campo: "amistad_id".to_string(),
                mensaje: "El identificador de la amistad es obligatorio."
                    .to_string(),
            });
        }

        // Solicitar la eliminación al backend.
        self.servicio_social.eliminar_amistad(
            &sesion.token,
            amistad_id,
        )
    }
}




#[cfg(test)]
mod tests {
    use super::*;

    use crate::aplicacion::pruebas::social::{
        AlmacenFalso,
        ServicioSocialFalso,
        sesion_prueba,
    };

    fn almacen_con_sesion() -> AlmacenFalso {
        AlmacenFalso {
            sesion: Some(sesion_prueba()),
        }
    }

    #[test]
    fn elimina_amistad_correctamente() {
        let servicio = ServicioSocialFalso::default();
        let almacen = almacen_con_sesion();

        let caso = EliminarAmistad::new(&servicio, &almacen);

        let resultado = caso.ejecutar("amistad-1");

        assert_eq!(resultado, Ok(()));

        let llamadas = servicio.llamadas_eliminar_amistad.borrow();
        assert_eq!(llamadas.len(), 1);
    }

    #[test]
    fn envia_token_e_id_correctamente() {
        let servicio = ServicioSocialFalso::default();
        let almacen = almacen_con_sesion();

        let caso = EliminarAmistad::new(&servicio, &almacen);

        caso.ejecutar("amistad-123").unwrap();

        assert_eq!(
            *servicio.llamadas_eliminar_amistad.borrow(),
            vec![(
                "token-prueba".to_string(),
                "amistad-123".to_string(),
            )]
        );
    }

    #[test]
    fn requiere_sesion() {
        let servicio = ServicioSocialFalso::default();
        let almacen = AlmacenFalso { sesion: None };

        let caso = EliminarAmistad::new(&servicio, &almacen);

        assert_eq!(
            caso.ejecutar("amistad-1"),
            Err(ErrorServicioSocial::NoAutenticado)
        );

        assert!(
            servicio.llamadas_eliminar_amistad
                .borrow()
                .is_empty()
        );
    }

    #[test]
    fn rechaza_id_vacio() {
        let servicio = ServicioSocialFalso::default();
        let almacen = almacen_con_sesion();

        let caso = EliminarAmistad::new(&servicio, &almacen);

        for id in ["", "   "] {
            assert!(matches!(
                caso.ejecutar(id),
                Err(ErrorServicioSocial::Validacion {
                    campo,
                    ..
                }) if campo == "amistad_id"
            ));
        }

        assert!(
            servicio.llamadas_eliminar_amistad
                .borrow()
                .is_empty()
        );
    }

    #[test]
    fn propaga_amistad_no_encontrada() {
        let mut servicio = ServicioSocialFalso::default();
        servicio.respuesta_eliminar_amistad =
            Err(ErrorServicioSocial::NoEncontrado);

        let almacen = almacen_con_sesion();
        let caso = EliminarAmistad::new(&servicio, &almacen);

        assert_eq!(
            caso.ejecutar("amistad-1"),
            Err(ErrorServicioSocial::NoEncontrado)
        );
    }

    #[test]
    fn propaga_error_de_permiso() {
        let mut servicio = ServicioSocialFalso::default();
        servicio.respuesta_eliminar_amistad =
            Err(ErrorServicioSocial::SinPermiso);

        let almacen = almacen_con_sesion();
        let caso = EliminarAmistad::new(&servicio, &almacen);

        assert_eq!(
            caso.ejecutar("amistad-1"),
            Err(ErrorServicioSocial::SinPermiso)
        );
    }

    #[test]
    fn propaga_conflicto_por_amistad_no_activa() {
        let mut servicio = ServicioSocialFalso::default();

        let conflicto = ErrorServicioSocial::Conflicto {
            campo: "amistad".into(),
            mensaje: "Solo se pueden eliminar amistades activas.".into(),
        };

        servicio.respuesta_eliminar_amistad =
            Err(conflicto.clone());

        let almacen = almacen_con_sesion();
        let caso = EliminarAmistad::new(&servicio, &almacen);

        assert_eq!(
            caso.ejecutar("amistad-1"),
            Err(conflicto)
        );
    }

    #[test]
    fn propaga_error_de_conexion() {
        let mut servicio = ServicioSocialFalso::default();
        servicio.respuesta_eliminar_amistad =
            Err(ErrorServicioSocial::ServicioNoDisponible);

        let almacen = almacen_con_sesion();
        let caso = EliminarAmistad::new(&servicio, &almacen);

        assert_eq!(
            caso.ejecutar("amistad-1"),
            Err(ErrorServicioSocial::ServicioNoDisponible)
        );
    }

    #[test]
    fn propaga_sesion_expirada() {
        let mut servicio = ServicioSocialFalso::default();
        servicio.respuesta_eliminar_amistad =
            Err(ErrorServicioSocial::SesionExpirada);

        let almacen = almacen_con_sesion();
        let caso = EliminarAmistad::new(&servicio, &almacen);

        assert_eq!(
            caso.ejecutar("amistad-1"),
            Err(ErrorServicioSocial::SesionExpirada)
        );
    }
}
