use crate::aplicacion::dto::amistades::{
    EnviarInvitacionAmistadEntrada,
    InvitacionAmistadDto,
};

use crate::aplicacion::puertos::{
    AlmacenSesiones,
    ErrorSesion,
};

use crate::aplicacion::puertos::servicio_social::{
    ErrorServicioSocial,
    ServicioSocial,
};


pub struct EnviarInvitacionAmistad<'a> {
    servicio_social: &'a dyn ServicioSocial,
    almacen_sesiones: &'a dyn AlmacenSesiones,
}


impl<'a> EnviarInvitacionAmistad<'a> {
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
        nombre_usuario_receptor: &str,
    ) -> Result<InvitacionAmistadDto, ErrorServicioSocial> {

        // Obtener sesión local.
        let sesion = self
            .almacen_sesiones
            .obtener_sesion()
            .map_err(|error| match error {
                ErrorSesion::Almacenamiento(mensaje) => {
                    ErrorServicioSocial::ErrorTecnico(mensaje)
                }
            })?
            .ok_or(ErrorServicioSocial::NoAutenticado)?;

        // Validar formato del nombre de usuario.
        if nombre_usuario_receptor.len() < 3
            || nombre_usuario_receptor.len() > 30
            || !nombre_usuario_receptor.chars().all(|c| {
                c.is_ascii_lowercase()
                    || c.is_ascii_digit()
                    || c == '.'
                    || c == '_'
            })
        {
            return Err(ErrorServicioSocial::Validacion {
                campo: "nombre_usuario_receptor".to_string(),
                mensaje: "El nombre de usuario no tiene un formato válido."
                    .to_string(),
            });
        }

        let entrada = EnviarInvitacionAmistadEntrada {
            nombre_usuario_receptor: nombre_usuario_receptor.to_string(),
        };

        self.servicio_social.enviar_invitacion_amistad(
            &sesion.token,
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
    fn envia_invitacion_correctamente() {
        let servicio = ServicioSocialFalso::default();
        let almacen = AlmacenFalso {
            sesion: Some(sesion_prueba()),
        };

        let caso = EnviarInvitacionAmistad::new(
            &servicio,
            &almacen,
        );

        let resultado = caso.ejecutar("beto").unwrap();

        assert_eq!(resultado.id, "amistad-1");
        assert_eq!(resultado.nombre_usuario_solicitante, "ana");
        assert_eq!(resultado.nombre_usuario_receptor, "beto");
        assert_eq!(resultado.estado, "PENDIENTE");
    }

    #[test]
    fn envia_token_y_destinatario_correctos() {
        let servicio = ServicioSocialFalso::default();
        let almacen = AlmacenFalso {
            sesion: Some(sesion_prueba()),
        };

        let caso = EnviarInvitacionAmistad::new(
            &servicio,
            &almacen,
        );

        caso.ejecutar("beto").unwrap();

        let llamadas = servicio.llamadas_enviar_invitacion.borrow();

        assert_eq!(llamadas.len(), 1);
        assert_eq!(llamadas[0].0, "token-prueba");
        assert_eq!(
            llamadas[0].1.nombre_usuario_receptor,
            "beto"
        );
    }

    #[test]
    fn requiere_sesion() {
        let servicio = ServicioSocialFalso::default();
        let almacen = AlmacenFalso {
            sesion: None,
        };

        let caso = EnviarInvitacionAmistad::new(
            &servicio,
            &almacen,
        );

        assert_eq!(
            caso.ejecutar("beto"),
            Err(ErrorServicioSocial::NoAutenticado)
        );

        assert!(
            servicio.llamadas_enviar_invitacion
                .borrow()
                .is_empty()
        );
    }

    #[test]
    fn rechaza_nombre_invalido() {
        let servicio = ServicioSocialFalso::default();
        let almacen = AlmacenFalso {
            sesion: Some(sesion_prueba()),
        };

        let caso = EnviarInvitacionAmistad::new(
            &servicio,
            &almacen,
        );

        for nombre in ["", "ab", "Beto", "a-b", "beto "] {
            assert!(matches!(
                caso.ejecutar(nombre),
                Err(ErrorServicioSocial::Validacion { .. })
            ));
        }

        assert!(
            servicio.llamadas_enviar_invitacion
                .borrow()
                .is_empty()
        );
    }

    #[test]
    fn propaga_usuario_no_encontrado() {
        let mut servicio = ServicioSocialFalso::default();

        servicio.respuesta_enviar_invitacion =
            Err(ErrorServicioSocial::NoEncontrado);

        let almacen = AlmacenFalso {
            sesion: Some(sesion_prueba()),
        };

        let caso = EnviarInvitacionAmistad::new(
            &servicio,
            &almacen,
        );

        assert_eq!(
            caso.ejecutar("beto"),
            Err(ErrorServicioSocial::NoEncontrado)
        );
    }

    #[test]
    fn propaga_conflicto_por_invitacion_duplicada() {
        let mut servicio = ServicioSocialFalso::default();

        let conflicto = ErrorServicioSocial::Conflicto {
            campo: "receptor".into(),
            mensaje: "Ya existe una invitación pendiente.".into(),
        };

        servicio.respuesta_enviar_invitacion =
            Err(conflicto.clone());

        let almacen = AlmacenFalso {
            sesion: Some(sesion_prueba()),
        };

        let caso = EnviarInvitacionAmistad::new(
            &servicio,
            &almacen,
        );

        assert_eq!(
            caso.ejecutar("beto"),
            Err(conflicto)
        );
    }

    #[test]
    fn propaga_error_de_conexion() {
        let mut servicio = ServicioSocialFalso::default();

        servicio.respuesta_enviar_invitacion =
            Err(ErrorServicioSocial::ServicioNoDisponible);

        let almacen = AlmacenFalso {
            sesion: Some(sesion_prueba()),
        };

        let caso = EnviarInvitacionAmistad::new(
            &servicio,
            &almacen,
        );

        assert_eq!(
            caso.ejecutar("beto"),
            Err(ErrorServicioSocial::ServicioNoDisponible)
        );
    }

    #[test]
    fn propaga_sesion_expirada() {
        let mut servicio = ServicioSocialFalso::default();

        servicio.respuesta_enviar_invitacion =
            Err(ErrorServicioSocial::SesionExpirada);

        let almacen = AlmacenFalso {
            sesion: Some(sesion_prueba()),
        };

        let caso = EnviarInvitacionAmistad::new(
            &servicio,
            &almacen,
        );

        assert_eq!(
            caso.ejecutar("beto"),
            Err(ErrorServicioSocial::SesionExpirada)
        );
    }
}
