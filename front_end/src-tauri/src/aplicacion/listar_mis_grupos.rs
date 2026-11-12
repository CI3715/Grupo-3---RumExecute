use crate::aplicacion::dto::grupos::ListadoGruposDto;

use crate::aplicacion::puertos::{
    AlmacenSesiones,
    ErrorSesion,
};

use crate::aplicacion::puertos::servicio_social::{
    ErrorServicioSocial,
    ServicioSocial,
};


pub struct ListarMisGrupos<'a> {
    servicio_social: &'a dyn ServicioSocial,
    almacen_sesiones: &'a dyn AlmacenSesiones,
}


impl<'a> ListarMisGrupos<'a> {
    pub fn new(
        servicio_social: &'a dyn ServicioSocial,
        almacen_sesiones: &'a dyn AlmacenSesiones,
    ) -> Self {
        Self {
            servicio_social,
            almacen_sesiones,
        }
    }

    pub fn ejecutar(&self) -> Result<ListadoGruposDto, ErrorServicioSocial> {
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

        // Consultar grupos e invitaciones del usuario.
        self.servicio_social.listar_mis_grupos(&sesion.token)
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
    fn lista_grupos_correctamente() {
        let servicio = ServicioSocialFalso::default();
        let almacen = almacen_con_sesion();

        let caso = ListarMisGrupos::new(&servicio, &almacen);
        let resultado = caso.ejecutar().unwrap();

        assert_eq!(resultado.grupos.len(), 1);
        assert_eq!(resultado.grupos[0].id, "grupo-1");
        assert_eq!(resultado.grupos[0].nombre_grupo, "Viaje");
        assert_eq!(resultado.grupos[0].moneda, "USD");
        assert_eq!(resultado.grupos[0].descripcion, "Vacaciones");
        assert_eq!(
            resultado.grupos[0].administrador_id,
            "usuario-1"
        );
    }

    #[test]
    fn lista_invitaciones_recibidas() {
        let servicio = ServicioSocialFalso::default();
        let almacen = almacen_con_sesion();

        let caso = ListarMisGrupos::new(&servicio, &almacen);
        let resultado = caso.ejecutar().unwrap();

        assert_eq!(resultado.invitaciones_recibidas.len(), 1);

        let invitacion = &resultado.invitaciones_recibidas[0];

        assert_eq!(invitacion.id, "invitacion-1");
        assert_eq!(invitacion.grupo_id, "grupo-2");
        assert_eq!(invitacion.invitador_id, "usuario-2");
        assert_eq!(invitacion.estado, "PENDIENTE");
    }

    #[test]
    fn envia_token_correctamente() {
        let servicio = ServicioSocialFalso::default();
        let almacen = almacen_con_sesion();

        let caso = ListarMisGrupos::new(&servicio, &almacen);
        caso.ejecutar().unwrap();

        assert_eq!(
            *servicio.llamadas_listar_mis_grupos.borrow(),
            vec!["token-prueba".to_string()]
        );
    }

    #[test]
    fn permite_listado_vacio() {
        let mut servicio = ServicioSocialFalso::default();

        servicio.respuesta_listar_mis_grupos =
            Ok(ListadoGruposDto {
                grupos: vec![],
                invitaciones_recibidas: vec![],
            });

        let almacen = almacen_con_sesion();
        let caso = ListarMisGrupos::new(&servicio, &almacen);

        let resultado = caso.ejecutar().unwrap();

        assert!(resultado.grupos.is_empty());
        assert!(resultado.invitaciones_recibidas.is_empty());
    }

    #[test]
    fn puede_tener_invitaciones_sin_ser_miembro() {
        let mut servicio = ServicioSocialFalso::default();

        let mut respuesta = servicio
            .respuesta_listar_mis_grupos
            .clone()
            .unwrap();

        respuesta.grupos.clear();

        servicio.respuesta_listar_mis_grupos = Ok(respuesta);

        let almacen = almacen_con_sesion();
        let caso = ListarMisGrupos::new(&servicio, &almacen);

        let resultado = caso.ejecutar().unwrap();

        assert!(resultado.grupos.is_empty());
        assert_eq!(resultado.invitaciones_recibidas.len(), 1);
    }

    #[test]
    fn requiere_sesion() {
        let servicio = ServicioSocialFalso::default();
        let almacen = AlmacenFalso {
            sesion: None,
        };

        let caso = ListarMisGrupos::new(&servicio, &almacen);

        assert_eq!(
            caso.ejecutar(),
            Err(ErrorServicioSocial::NoAutenticado)
        );

        assert!(
            servicio.llamadas_listar_mis_grupos
                .borrow()
                .is_empty()
        );
    }

    #[test]
    fn propaga_sesion_expirada() {
        let mut servicio = ServicioSocialFalso::default();

        servicio.respuesta_listar_mis_grupos =
            Err(ErrorServicioSocial::SesionExpirada);

        let almacen = almacen_con_sesion();
        let caso = ListarMisGrupos::new(&servicio, &almacen);

        assert_eq!(
            caso.ejecutar(),
            Err(ErrorServicioSocial::SesionExpirada)
        );
    }

    #[test]
    fn propaga_error_de_conexion() {
        let mut servicio = ServicioSocialFalso::default();

        servicio.respuesta_listar_mis_grupos =
            Err(ErrorServicioSocial::ServicioNoDisponible);

        let almacen = almacen_con_sesion();
        let caso = ListarMisGrupos::new(&servicio, &almacen);

        assert_eq!(
            caso.ejecutar(),
            Err(ErrorServicioSocial::ServicioNoDisponible)
        );
    }

    #[test]
    fn propaga_error_tecnico() {
        let mut servicio = ServicioSocialFalso::default();

        let error = ErrorServicioSocial::ErrorTecnico(
            "No se pudo procesar el listado".into()
        );

        servicio.respuesta_listar_mis_grupos =
            Err(error.clone());

        let almacen = almacen_con_sesion();
        let caso = ListarMisGrupos::new(&servicio, &almacen);

        assert_eq!(caso.ejecutar(), Err(error));
    }
}
