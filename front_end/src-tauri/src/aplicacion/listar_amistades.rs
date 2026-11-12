use crate::aplicacion::dto::amistades::ListadoAmistadesDto;
use crate::aplicacion::puertos::{
    AlmacenSesiones,
    ErrorSesion,
};
use crate::aplicacion::puertos::servicio_social::{
    ErrorServicioSocial,
    ServicioSocial,
};


pub struct ListarAmistades<'a> {
    servicio_social: &'a dyn ServicioSocial,
    almacen_sesiones: &'a dyn AlmacenSesiones,
}


impl<'a> ListarAmistades<'a> {
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
    ) -> Result<ListadoAmistadesDto, ErrorServicioSocial> {

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

        // Solicitar al backend las amistades del usuario.
        self.servicio_social.listar_amistades(&sesion.token)
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
    fn lista_amistades_correctamente() {
        let servicio = ServicioSocialFalso::default();
        let almacen = almacen_con_sesion();

        let caso = ListarAmistades::new(&servicio, &almacen);
        let resultado = caso.ejecutar().unwrap();

        assert_eq!(resultado.amigos.len(), 1);
        assert_eq!(resultado.amigos[0].amistad_id, "amistad-1");
        assert_eq!(resultado.amigos[0].nombre_usuario, "beto");
        assert_eq!(resultado.amigos[0].nombre_visible, "Beto");
    }

    #[test]
    fn lista_invitaciones_recibidas_y_enviadas() {
        let servicio = ServicioSocialFalso::default();
        let almacen = almacen_con_sesion();

        let caso = ListarAmistades::new(&servicio, &almacen);
        let resultado = caso.ejecutar().unwrap();

        assert_eq!(resultado.invitaciones_recibidas.len(), 1);
        assert_eq!(
            resultado.invitaciones_recibidas[0].nombre_usuario,
            "carla"
        );
        assert_eq!(
            resultado.invitaciones_recibidas[0].amistad_id,
            "amistad-2"
        );

        assert_eq!(resultado.invitaciones_enviadas.len(), 1);
        assert_eq!(
            resultado.invitaciones_enviadas[0].nombre_usuario,
            "daniel"
        );
        assert_eq!(
            resultado.invitaciones_enviadas[0].amistad_id,
            "amistad-3"
        );
    }

    #[test]
    fn envia_token_correctamente() {
        let servicio = ServicioSocialFalso::default();
        let almacen = almacen_con_sesion();

        let caso = ListarAmistades::new(&servicio, &almacen);
        caso.ejecutar().unwrap();

        assert_eq!(
            *servicio.llamadas_listar_amistades.borrow(),
            vec!["token-prueba".to_string()]
        );
    }

    #[test]
    fn permite_listado_vacio() {
        let mut servicio = ServicioSocialFalso::default();

        servicio.respuesta_listar_amistades =
            Ok(ListadoAmistadesDto {
                amigos: vec![],
                invitaciones_recibidas: vec![],
                invitaciones_enviadas: vec![],
            });

        let almacen = almacen_con_sesion();
        let caso = ListarAmistades::new(&servicio, &almacen);

        let resultado = caso.ejecutar().unwrap();

        assert!(resultado.amigos.is_empty());
        assert!(resultado.invitaciones_recibidas.is_empty());
        assert!(resultado.invitaciones_enviadas.is_empty());
    }

    #[test]
    fn requiere_sesion() {
        let servicio = ServicioSocialFalso::default();
        let almacen = AlmacenFalso { sesion: None };

        let caso = ListarAmistades::new(&servicio, &almacen);

        assert_eq!(
            caso.ejecutar(),
            Err(ErrorServicioSocial::NoAutenticado)
        );

        assert!(
            servicio.llamadas_listar_amistades
                .borrow()
                .is_empty()
        );
    }

    #[test]
    fn propaga_sesion_expirada() {
        let mut servicio = ServicioSocialFalso::default();

        servicio.respuesta_listar_amistades =
            Err(ErrorServicioSocial::SesionExpirada);

        let almacen = almacen_con_sesion();
        let caso = ListarAmistades::new(&servicio, &almacen);

        assert_eq!(
            caso.ejecutar(),
            Err(ErrorServicioSocial::SesionExpirada)
        );
    }

    #[test]
    fn propaga_error_de_conexion() {
        let mut servicio = ServicioSocialFalso::default();

        servicio.respuesta_listar_amistades =
            Err(ErrorServicioSocial::ServicioNoDisponible);

        let almacen = almacen_con_sesion();
        let caso = ListarAmistades::new(&servicio, &almacen);

        assert_eq!(
            caso.ejecutar(),
            Err(ErrorServicioSocial::ServicioNoDisponible)
        );
    }

    #[test]
    fn propaga_error_tecnico() {
        let mut servicio = ServicioSocialFalso::default();

        let error = ErrorServicioSocial::ErrorTecnico(
            "Error al procesar la respuesta".into()
        );

        servicio.respuesta_listar_amistades =
            Err(error.clone());

        let almacen = almacen_con_sesion();
        let caso = ListarAmistades::new(&servicio, &almacen);

        assert_eq!(caso.ejecutar(), Err(error));
    }
}
