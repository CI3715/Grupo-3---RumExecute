use crate::aplicacion::dto::grupos::{
    ResponderInvitacionGrupoEntrada,
    RespuestaInvitacionGrupoDto,
};

use crate::aplicacion::puertos::{
    AlmacenSesiones,
    ErrorSesion,
};

use crate::aplicacion::puertos::servicio_social::{
    ErrorServicioSocial,
    ServicioSocial,
};

pub struct ResponderInvitacionGrupo<'a> {
    servicio_social: &'a dyn ServicioSocial,
    almacen_sesiones: &'a dyn AlmacenSesiones,
}

impl<'a> ResponderInvitacionGrupo<'a> {
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
        grupo_id: &str,
        aceptar: bool,
    ) -> Result<RespuestaInvitacionGrupoDto, ErrorServicioSocial> {
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

        // Validar el identificador del grupo.
        if grupo_id.trim().is_empty() {
            return Err(ErrorServicioSocial::Validacion {
                campo: "grupo_id".to_string(),
                mensaje: "El identificador del grupo es obligatorio."
                    .to_string(),
            });
        }

        let entrada = ResponderInvitacionGrupoEntrada {
            aceptar,
        };

        self.servicio_social.responder_invitacion_grupo(
            &sesion.token,
            grupo_id,
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

    fn almacen_con_sesion() -> AlmacenFalso {
        AlmacenFalso {
            sesion: Some(sesion_prueba()),
        }
    }

    #[test]
    fn acepta_invitacion_correctamente() {
        let servicio = ServicioSocialFalso::default();
        let almacen = almacen_con_sesion();

        let caso = ResponderInvitacionGrupo::new(&servicio, &almacen);

        let resultado = caso.ejecutar("grupo-1", true).unwrap();

        assert_eq!(resultado.id, "invitacion-1");
        assert_eq!(resultado.grupo_id, "grupo-1");
        assert_eq!(resultado.estado, "ACEPTADA");
    }

    #[test]
    fn rechaza_invitacion_correctamente() {
        let mut servicio = ServicioSocialFalso::default();

        servicio.respuesta_responder_invitacion_grupo =
            Ok(RespuestaInvitacionGrupoDto {
                id: "invitacion-1".into(),
                grupo_id: "grupo-1".into(),
                estado: "RECHAZADA".into(),
            });

        let almacen = almacen_con_sesion();
        let caso = ResponderInvitacionGrupo::new(&servicio, &almacen);

        let resultado = caso.ejecutar("grupo-1", false).unwrap();

        assert_eq!(resultado.estado, "RECHAZADA");

        let llamadas = servicio
            .llamadas_responder_invitacion_grupo
            .borrow();

        assert_eq!(llamadas.len(), 1);
        assert!(!llamadas[0].2.aceptar);
    }

    #[test]
    fn envia_token_grupo_y_decision_correctos() {
        let servicio = ServicioSocialFalso::default();
        let almacen = almacen_con_sesion();

        let caso = ResponderInvitacionGrupo::new(&servicio, &almacen);

        caso.ejecutar("grupo-1", true).unwrap();

        let llamadas = servicio
            .llamadas_responder_invitacion_grupo
            .borrow();

        assert_eq!(llamadas.len(), 1);
        assert_eq!(llamadas[0].0, "token-prueba");
        assert_eq!(llamadas[0].1, "grupo-1");
        assert!(llamadas[0].2.aceptar);
    }

    #[test]
    fn requiere_sesion() {
        let servicio = ServicioSocialFalso::default();
        let almacen = AlmacenFalso {
            sesion: None,
        };

        let caso = ResponderInvitacionGrupo::new(&servicio, &almacen);

        assert_eq!(
            caso.ejecutar("grupo-1", true),
            Err(ErrorServicioSocial::NoAutenticado)
        );

        assert!(
            servicio.llamadas_responder_invitacion_grupo
                .borrow()
                .is_empty()
        );
    }

    #[test]
    fn rechaza_grupo_id_vacio() {
        let servicio = ServicioSocialFalso::default();
        let almacen = almacen_con_sesion();

        let caso = ResponderInvitacionGrupo::new(&servicio, &almacen);

        for grupo_id in ["", "   "] {
            assert!(matches!(
                caso.ejecutar(grupo_id, true),
                Err(ErrorServicioSocial::Validacion {
                    campo,
                    ..
                }) if campo == "grupo_id"
            ));
        }

        assert!(
            servicio.llamadas_responder_invitacion_grupo
                .borrow()
                .is_empty()
        );
    }

    #[test]
    fn conserva_grupo_id_exacto() {
        let servicio = ServicioSocialFalso::default();
        let almacen = almacen_con_sesion();

        let caso = ResponderInvitacionGrupo::new(&servicio, &almacen);

        caso.ejecutar("grupo-123", false).unwrap();

        let llamadas = servicio
            .llamadas_responder_invitacion_grupo
            .borrow();

        assert_eq!(llamadas.len(), 1);
        assert_eq!(llamadas[0].1, "grupo-123");
        assert!(!llamadas[0].2.aceptar);
    }

    #[test]
    fn propaga_invitacion_no_encontrada() {
        let mut servicio = ServicioSocialFalso::default();

        servicio.respuesta_responder_invitacion_grupo =
            Err(ErrorServicioSocial::NoEncontrado);

        let almacen = almacen_con_sesion();
        let caso = ResponderInvitacionGrupo::new(&servicio, &almacen);

        assert_eq!(
            caso.ejecutar("grupo-1", true),
            Err(ErrorServicioSocial::NoEncontrado)
        );
    }

    #[test]
    fn propaga_error_de_permiso() {
        let mut servicio = ServicioSocialFalso::default();

        servicio.respuesta_responder_invitacion_grupo =
            Err(ErrorServicioSocial::SinPermiso);

        let almacen = almacen_con_sesion();
        let caso = ResponderInvitacionGrupo::new(&servicio, &almacen);

        assert_eq!(
            caso.ejecutar("grupo-1", true),
            Err(ErrorServicioSocial::SinPermiso)
        );
    }

    #[test]
    fn propaga_conflicto_por_invitacion_ya_respondida() {
        let mut servicio = ServicioSocialFalso::default();

        let conflicto = ErrorServicioSocial::Conflicto {
            campo: "invitacion".into(),
            mensaje: "La invitación ya fue respondida.".into(),
        };

        servicio.respuesta_responder_invitacion_grupo =
            Err(conflicto.clone());

        let almacen = almacen_con_sesion();
        let caso = ResponderInvitacionGrupo::new(&servicio, &almacen);

        assert_eq!(
            caso.ejecutar("grupo-1", true),
            Err(conflicto)
        );
    }

    #[test]
    fn propaga_error_de_conexion() {
        let mut servicio = ServicioSocialFalso::default();

        servicio.respuesta_responder_invitacion_grupo =
            Err(ErrorServicioSocial::ServicioNoDisponible);

        let almacen = almacen_con_sesion();
        let caso = ResponderInvitacionGrupo::new(&servicio, &almacen);

        assert_eq!(
            caso.ejecutar("grupo-1", true),
            Err(ErrorServicioSocial::ServicioNoDisponible)
        );
    }

    #[test]
    fn propaga_sesion_expirada() {
        let mut servicio = ServicioSocialFalso::default();

        servicio.respuesta_responder_invitacion_grupo =
            Err(ErrorServicioSocial::SesionExpirada);

        let almacen = almacen_con_sesion();
        let caso = ResponderInvitacionGrupo::new(&servicio, &almacen);

        assert_eq!(
            caso.ejecutar("grupo-1", true),
            Err(ErrorServicioSocial::SesionExpirada)
        );
    }
}
