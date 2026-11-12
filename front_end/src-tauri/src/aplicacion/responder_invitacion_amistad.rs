use crate::aplicacion::dto::amistades::{
    ResponderInvitacionAmistadEntrada,
    RespuestaAmistadDto,
};
use crate::aplicacion::puertos::{
    AlmacenSesiones,
    ErrorSesion,
};
use crate::aplicacion::puertos::servicio_social::{
    ErrorServicioSocial,
    ServicioSocial,
};


pub struct ResponderInvitacionAmistad<'a> {
    servicio_social: &'a dyn ServicioSocial,
    almacen_sesiones: &'a dyn AlmacenSesiones,
}

impl<'a> ResponderInvitacionAmistad<'a> {
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
        aceptar: bool,
    ) -> Result<RespuestaAmistadDto, ErrorServicioSocial> {

        // Recuperar sesión local.
        let sesion = self
            .almacen_sesiones
            .obtener_sesion()
            .map_err(|error| match error {
                ErrorSesion::Almacenamiento(mensaje) => {
                    ErrorServicioSocial::ErrorTecnico(mensaje)
                }
            })?
            .ok_or(ErrorServicioSocial::NoAutenticado)?;

        // Validar que se haya proporcionado una invitación.
        if amistad_id.trim().is_empty() {
            return Err(ErrorServicioSocial::Validacion {
                campo: "amistad_id".to_string(),
                mensaje: "El identificador de la invitación es obligatorio."
                    .to_string(),
            });
        }

        let entrada = ResponderInvitacionAmistadEntrada {
            aceptar,
        };

        self.servicio_social.responder_invitacion_amistad(
            &sesion.token,
            amistad_id,
            &entrada,
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

    #[test]
    fn acepta_invitacion_correctamente() {
        let servicio = ServicioSocialFalso::default();
        let almacen = AlmacenFalso {
            sesion: Some(sesion_prueba()),
        };

        let caso = ResponderInvitacionAmistad::new(
            &servicio, &almacen
        );

        let resultado = caso.ejecutar("amistad-1", true).unwrap();

        assert_eq!(resultado.id, "amistad-1");
        assert_eq!(resultado.estado, "ACTIVA");
    }

    #[test]
    fn rechaza_invitacion_correctamente() {
        let mut servicio = ServicioSocialFalso::default();
        servicio.respuesta_responder_amistad =
            Ok(RespuestaAmistadDto {
                id: "amistad-1".into(),
                estado: "RECHAZADA".into(),
            });

        let almacen = AlmacenFalso {
            sesion: Some(sesion_prueba()),
        };

        let caso = ResponderInvitacionAmistad::new(
            &servicio, &almacen
        );

        let resultado = caso.ejecutar("amistad-1", false).unwrap();

        assert_eq!(resultado.estado, "RECHAZADA");

        let llamadas = servicio.llamadas_responder_amistad.borrow();
        assert_eq!(llamadas.len(), 1);
        assert!(!llamadas[0].2.aceptar);
    }

    #[test]
    fn envia_token_id_y_decision_correctos() {
        let servicio = ServicioSocialFalso::default();
        let almacen = AlmacenFalso {
            sesion: Some(sesion_prueba()),
        };

        let caso = ResponderInvitacionAmistad::new(
            &servicio, &almacen
        );

        caso.ejecutar("amistad-1", true).unwrap();

        let llamadas = servicio.llamadas_responder_amistad.borrow();

        assert_eq!(llamadas.len(), 1);
        assert_eq!(llamadas[0].0, "token-prueba");
        assert_eq!(llamadas[0].1, "amistad-1");
        assert!(llamadas[0].2.aceptar);
    }

    #[test]
    fn requiere_sesion() {
        let servicio = ServicioSocialFalso::default();
        let almacen = AlmacenFalso {
            sesion: None,
        };

        let caso = ResponderInvitacionAmistad::new(
            &servicio, &almacen
        );

        assert_eq!(
            caso.ejecutar("amistad-1", true),
            Err(ErrorServicioSocial::NoAutenticado)
        );

        assert!(
            servicio.llamadas_responder_amistad.borrow().is_empty()
        );
    }

    #[test]
    fn rechaza_id_vacio() {
        let servicio = ServicioSocialFalso::default();
        let almacen = AlmacenFalso {
            sesion: Some(sesion_prueba()),
        };

        let caso = ResponderInvitacionAmistad::new(
            &servicio, &almacen
        );

        for id in ["", "   "] {
            assert!(matches!(
                caso.ejecutar(id, true),
                Err(ErrorServicioSocial::Validacion { .. })
            ));
        }

        assert!(
            servicio.llamadas_responder_amistad.borrow().is_empty()
        );
    }

    #[test]
    fn propaga_invitacion_no_encontrada() {
        let mut servicio = ServicioSocialFalso::default();
        servicio.respuesta_responder_amistad =
            Err(ErrorServicioSocial::NoEncontrado);

        let almacen = AlmacenFalso {
            sesion: Some(sesion_prueba()),
        };

        let caso = ResponderInvitacionAmistad::new(
            &servicio, &almacen
        );

        assert_eq!(
            caso.ejecutar("amistad-1", true),
            Err(ErrorServicioSocial::NoEncontrado)
        );
    }

    #[test]
    fn propaga_error_de_permiso() {
        let mut servicio = ServicioSocialFalso::default();
        servicio.respuesta_responder_amistad =
            Err(ErrorServicioSocial::SinPermiso);

        let almacen = AlmacenFalso {
            sesion: Some(sesion_prueba()),
        };

        let caso = ResponderInvitacionAmistad::new(
            &servicio, &almacen
        );

        assert_eq!(
            caso.ejecutar("amistad-1", true),
            Err(ErrorServicioSocial::SinPermiso)
        );
    }

    #[test]
    fn propaga_conflicto_por_invitacion_ya_respondida() {
        let mut servicio = ServicioSocialFalso::default();

        let conflicto = ErrorServicioSocial::Conflicto {
            campo: "amistad".into(),
            mensaje: "La invitación ya fue respondida.".into(),
        };

        servicio.respuesta_responder_amistad =
            Err(conflicto.clone());

        let almacen = AlmacenFalso {
            sesion: Some(sesion_prueba()),
        };

        let caso = ResponderInvitacionAmistad::new(
            &servicio, &almacen
        );

        assert_eq!(
            caso.ejecutar("amistad-1", true),
            Err(conflicto)
        );
    }

    #[test]
    fn propaga_error_de_conexion() {
        let mut servicio = ServicioSocialFalso::default();
        servicio.respuesta_responder_amistad =
            Err(ErrorServicioSocial::ServicioNoDisponible);

        let almacen = AlmacenFalso {
            sesion: Some(sesion_prueba()),
        };

        let caso = ResponderInvitacionAmistad::new(
            &servicio, &almacen
        );

        assert_eq!(
            caso.ejecutar("amistad-1", true),
            Err(ErrorServicioSocial::ServicioNoDisponible)
        );
    }

    #[test]
    fn propaga_sesion_expirada() {
        let mut servicio = ServicioSocialFalso::default();
        servicio.respuesta_responder_amistad =
            Err(ErrorServicioSocial::SesionExpirada);

        let almacen = AlmacenFalso {
            sesion: Some(sesion_prueba()),
        };

        let caso = ResponderInvitacionAmistad::new(
            &servicio, &almacen
        );

        assert_eq!(
            caso.ejecutar("amistad-1", true),
            Err(ErrorServicioSocial::SesionExpirada)
        );
    }
}
