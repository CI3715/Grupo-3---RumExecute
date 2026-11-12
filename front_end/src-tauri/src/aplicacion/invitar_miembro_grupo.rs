use crate::aplicacion::dto::grupos::{
    InvitarMiembroGrupoEntrada,
    InvitacionGrupoDto,
};

use crate::aplicacion::puertos::{
    AlmacenSesiones,
    ErrorSesion,
};

use crate::aplicacion::puertos::servicio_social::{
    ErrorServicioSocial,
    ServicioSocial,
};

pub struct InvitarMiembroGrupo<'a> {
    servicio_social: &'a dyn ServicioSocial,
    almacen_sesiones: &'a dyn AlmacenSesiones,
}

impl<'a> InvitarMiembroGrupo<'a> {
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
        nombre_usuario_invitado: &str,
    ) -> Result<InvitacionGrupoDto, ErrorServicioSocial> {
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

        // Validar el nombre de usuario del invitado.
        if nombre_usuario_invitado.len() < 3
            || nombre_usuario_invitado.len() > 30
            || !nombre_usuario_invitado.chars().all(|c| {
                c.is_ascii_lowercase()
                    || c.is_ascii_digit()
                    || c == '.'
                    || c == '_'
            })
        {
            return Err(ErrorServicioSocial::Validacion {
                campo: "nombre_usuario_invitado".to_string(),
                mensaje: "El nombre de usuario no tiene un formato válido."
                    .to_string(),
            });
        }

        let entrada = InvitarMiembroGrupoEntrada {
            nombre_usuario_invitado: nombre_usuario_invitado.to_string(),
        };

        self.servicio_social.invitar_miembro_grupo(
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
    fn invita_miembro_correctamente() {
        let servicio = ServicioSocialFalso::default();
        let almacen = almacen_con_sesion();

        let caso = InvitarMiembroGrupo::new(&servicio, &almacen);

        let resultado = caso.ejecutar("grupo-1", "beto").unwrap();

        assert_eq!(resultado.id, "invitacion-1");
        assert_eq!(resultado.grupo_id, "grupo-1");
        assert_eq!(resultado.invitado_id, "usuario-2");
        assert_eq!(resultado.invitador_id, "usuario-1");
        assert_eq!(resultado.estado, "PENDIENTE");
    }

    #[test]
    fn envia_token_grupo_y_usuario_correctos() {
        let servicio = ServicioSocialFalso::default();
        let almacen = almacen_con_sesion();

        let caso = InvitarMiembroGrupo::new(&servicio, &almacen);

        caso.ejecutar("grupo-1", "beto").unwrap();

        let llamadas = servicio.llamadas_invitar_miembro_grupo.borrow();

        assert_eq!(llamadas.len(), 1);
        assert_eq!(llamadas[0].0, "token-prueba");
        assert_eq!(llamadas[0].1, "grupo-1");
        assert_eq!(
            llamadas[0].2.nombre_usuario_invitado,
            "beto"
        );
    }

    #[test]
    fn requiere_sesion() {
        let servicio = ServicioSocialFalso::default();
        let almacen = AlmacenFalso {
            sesion: None,
        };

        let caso = InvitarMiembroGrupo::new(&servicio, &almacen);

        assert_eq!(
            caso.ejecutar("grupo-1", "beto"),
            Err(ErrorServicioSocial::NoAutenticado)
        );

        assert!(
            servicio.llamadas_invitar_miembro_grupo
                .borrow()
                .is_empty()
        );
    }

    #[test]
    fn rechaza_grupo_id_vacio() {
        let servicio = ServicioSocialFalso::default();
        let almacen = almacen_con_sesion();

        let caso = InvitarMiembroGrupo::new(&servicio, &almacen);

        for id in ["", "   "] {
            assert!(matches!(
                caso.ejecutar(id, "beto"),
                Err(ErrorServicioSocial::Validacion {
                    campo,
                    ..
                }) if campo == "grupo_id"
            ));
        }

        assert!(
            servicio.llamadas_invitar_miembro_grupo
                .borrow()
                .is_empty()
        );
    }

    #[test]
    fn rechaza_nombre_usuario_invalido() {
        let servicio = ServicioSocialFalso::default();
        let almacen = almacen_con_sesion();

        let caso = InvitarMiembroGrupo::new(&servicio, &almacen);

        for nombre in ["", "ab", "Beto", "a-b", "beto "] {
            assert!(matches!(
                caso.ejecutar("grupo-1", nombre),
                Err(ErrorServicioSocial::Validacion {
                    campo,
                    ..
                }) if campo == "nombre_usuario_invitado"
            ));
        }

        assert!(
            servicio.llamadas_invitar_miembro_grupo
                .borrow()
                .is_empty()
        );
    }

    #[test]
    fn conserva_nombre_usuario_exacto() {
        let servicio = ServicioSocialFalso::default();
        let almacen = almacen_con_sesion();

        let caso = InvitarMiembroGrupo::new(&servicio, &almacen);

        caso.ejecutar("grupo-1", "beto_123").unwrap();

        let llamadas = servicio.llamadas_invitar_miembro_grupo.borrow();

        assert_eq!(llamadas.len(), 1);
        assert_eq!(
            llamadas[0].2.nombre_usuario_invitado,
            "beto_123"
        );
    }

    #[test]
    fn propaga_usuario_o_grupo_no_encontrado() {
        let mut servicio = ServicioSocialFalso::default();

        servicio.respuesta_invitar_miembro_grupo =
            Err(ErrorServicioSocial::NoEncontrado);

        let almacen = almacen_con_sesion();
        let caso = InvitarMiembroGrupo::new(&servicio, &almacen);

        assert_eq!(
            caso.ejecutar("grupo-1", "beto"),
            Err(ErrorServicioSocial::NoEncontrado)
        );
    }

    #[test]
    fn propaga_error_de_permiso() {
        let mut servicio = ServicioSocialFalso::default();

        servicio.respuesta_invitar_miembro_grupo =
            Err(ErrorServicioSocial::SinPermiso);

        let almacen = almacen_con_sesion();
        let caso = InvitarMiembroGrupo::new(&servicio, &almacen);

        assert_eq!(
            caso.ejecutar("grupo-1", "beto"),
            Err(ErrorServicioSocial::SinPermiso)
        );
    }

    #[test]
    fn propaga_conflicto_por_invitacion_duplicada() {
        let mut servicio = ServicioSocialFalso::default();

        let conflicto = ErrorServicioSocial::Conflicto {
            campo: "invitado".into(),
            mensaje: "Ya existe una invitación pendiente.".into(),
        };

        servicio.respuesta_invitar_miembro_grupo =
            Err(conflicto.clone());

        let almacen = almacen_con_sesion();
        let caso = InvitarMiembroGrupo::new(&servicio, &almacen);

        assert_eq!(
            caso.ejecutar("grupo-1", "beto"),
            Err(conflicto)
        );
    }

    #[test]
    fn propaga_error_de_conexion() {
        let mut servicio = ServicioSocialFalso::default();

        servicio.respuesta_invitar_miembro_grupo =
            Err(ErrorServicioSocial::ServicioNoDisponible);

        let almacen = almacen_con_sesion();
        let caso = InvitarMiembroGrupo::new(&servicio, &almacen);

        assert_eq!(
            caso.ejecutar("grupo-1", "beto"),
            Err(ErrorServicioSocial::ServicioNoDisponible)
        );
    }

    #[test]
    fn propaga_sesion_expirada() {
        let mut servicio = ServicioSocialFalso::default();

        servicio.respuesta_invitar_miembro_grupo =
            Err(ErrorServicioSocial::SesionExpirada);

        let almacen = almacen_con_sesion();
        let caso = InvitarMiembroGrupo::new(&servicio, &almacen);

        assert_eq!(
            caso.ejecutar("grupo-1", "beto"),
            Err(ErrorServicioSocial::SesionExpirada)
        );
    }
}
