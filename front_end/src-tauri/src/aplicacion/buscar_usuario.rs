use crate::aplicacion::dto::usuario_publico::UsuarioPublicoDto;
use crate::aplicacion::puertos::{
    AlmacenSesiones,
    ErrorSesion,
};
use crate::aplicacion::puertos::servicio_social::{
    ErrorServicioSocial,
    ServicioSocial,
};


pub struct BuscarUsuario<'a> {
    servicio_social: &'a dyn ServicioSocial,
    almacen_sesiones: &'a dyn AlmacenSesiones,
}


impl<'a> BuscarUsuario<'a> {
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
        nombre_usuario: &str,
    ) -> Result<UsuarioPublicoDto, ErrorServicioSocial> {
        // Recuperar la sesión del usuario autenticado.
        let sesion = self
            .almacen_sesiones
            .obtener_sesion()
            .map_err(|error| match error {
                ErrorSesion::Almacenamiento(mensaje) => {
                    ErrorServicioSocial::ErrorTecnico(mensaje)
                }
            })?
            .ok_or(ErrorServicioSocial::NoAutenticado)?;

        // Validar el nombre de usuario antes de consultar el backend.
        if nombre_usuario.len() < 3
            || nombre_usuario.len() > 30
            || !nombre_usuario
                .chars()
                .all(|c| c.is_ascii_lowercase()
                    || c.is_ascii_digit()
                    || c == '.'
                    || c == '_')
        {
            return Err(ErrorServicioSocial::Validacion {
                campo: "nombre_usuario".to_string(),
                mensaje: "El nombre de usuario debe tener entre 3 y 30 caracteres y contener solo minúsculas, dígitos, puntos o guiones bajos.".to_string(),
            });
        }

        // Buscar mediante el puerto, sin conocer detalles HTTP.
        self.servicio_social.buscar_usuario(
            &sesion.token,
            nombre_usuario,
        )
    }
}



#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;

    use crate::aplicacion::dto::{
        SesionDto,
        UsuarioRegistradoDto,
    };
    use crate::aplicacion::dto::amistades::*;
    use crate::aplicacion::dto::grupos::*;
    use crate::aplicacion::dto::gastos::*;

    struct ServicioSocialFalso {
        llamadas: RefCell<Vec<(String, String)>>,
        respuesta: Result<UsuarioPublicoDto, ErrorServicioSocial>,
    }

    impl ServicioSocial for ServicioSocialFalso {
        fn buscar_usuario(
            &self,
            token: &str,
            nombre_usuario: &str,
        ) -> Result<UsuarioPublicoDto, ErrorServicioSocial> {
            self.llamadas.borrow_mut().push((
                token.to_string(),
                nombre_usuario.to_string(),
            ));
            self.respuesta.clone()
        }

        fn enviar_invitacion_amistad(
            &self, _: &str, _: &EnviarInvitacionAmistadEntrada,
        ) -> Result<InvitacionAmistadDto, ErrorServicioSocial> {
            unimplemented!()
        }

        fn responder_invitacion_amistad(
            &self, _: &str, _: &str,
            _: &ResponderInvitacionAmistadEntrada,
        ) -> Result<RespuestaAmistadDto, ErrorServicioSocial> {
            unimplemented!()
        }

        fn listar_amistades(
            &self, _: &str,
        ) -> Result<ListadoAmistadesDto, ErrorServicioSocial> {
            unimplemented!()
        }

        fn eliminar_amistad(
            &self, _: &str, _: &str,
        ) -> Result<(), ErrorServicioSocial> {
            unimplemented!()
        }

        fn crear_grupo(
            &self, _: &str, _: &CrearGrupoEntrada,
        ) -> Result<GrupoCreadoDto, ErrorServicioSocial> {
            unimplemented!()
        }

        fn listar_mis_grupos(
            &self, _: &str,
        ) -> Result<ListadoGruposDto, ErrorServicioSocial> {
            unimplemented!()
        }

        fn invitar_miembro_grupo(
            &self, _: &str, _: &str, _: &InvitarMiembroGrupoEntrada,
        ) -> Result<InvitacionGrupoDto, ErrorServicioSocial> {
            unimplemented!()
        }

        fn responder_invitacion_grupo(
            &self, _: &str, _: &str,
            _: &ResponderInvitacionGrupoEntrada,
        ) -> Result<RespuestaInvitacionGrupoDto, ErrorServicioSocial> {
            unimplemented!()
        }

        fn registrar_gasto_compartido(
            &self, _: &str, _: &str,
            _: &RegistrarGastoCompartidoEntrada,
        ) -> Result<GastoCompartidoDto, ErrorServicioSocial> {
            unimplemented!()
        }

        fn obtener_detalle_grupo(
            &self, _: &str, _: &str,
        ) -> Result<DetalleGrupoDto, ErrorServicioSocial> {
            unimplemented!()
        }
    }

    struct AlmacenFalso {
        sesion: Option<SesionDto>,
    }

    impl AlmacenSesiones for AlmacenFalso {
        fn guardar_sesion(
            &self, _: &SesionDto,
        ) -> Result<(), ErrorSesion> {
            Ok(())
        }

        fn obtener_sesion(
            &self,
        ) -> Result<Option<SesionDto>, ErrorSesion> {
            Ok(self.sesion.clone())
        }

        fn eliminar_sesion(&self) -> Result<(), ErrorSesion> {
            Ok(())
        }
    }

    fn sesion() -> SesionDto {
        SesionDto {
            usuario: UsuarioRegistradoDto {
                id: "usuario-1".into(),
                nombre_usuario: "ana".into(),
                nombre_visible: "Ana".into(),
                correo: "ana@example.com".into(),
                fecha_alta: "2026-11-12T12:00:00Z".into(),
            },
            token: "token-prueba".into(),
            fecha_expiracion: "2026-12-12T12:00:00Z".into(),
        }
    }

    fn servicio() -> ServicioSocialFalso {
        ServicioSocialFalso {
            llamadas: RefCell::new(vec![]),
            respuesta: Ok(UsuarioPublicoDto {
                nombre_usuario: "beto".into(),
                nombre_visible: "Beto".into(),
            }),
        }
    }

    #[test]
    fn busca_usuario_correctamente() {
        let servicio = servicio();
        let almacen = AlmacenFalso {
            sesion: Some(sesion()),
        };

        let caso = BuscarUsuario::new(&servicio, &almacen);

        let resultado = caso.ejecutar("beto").unwrap();

        assert_eq!(resultado.nombre_usuario, "beto");
        assert_eq!(resultado.nombre_visible, "Beto");
        assert_eq!(
            *servicio.llamadas.borrow(),
            vec![("token-prueba".into(), "beto".into())]
        );
    }

    #[test]
    fn requiere_sesion() {
        let servicio = servicio();
        let almacen = AlmacenFalso { sesion: None };
        let caso = BuscarUsuario::new(&servicio, &almacen);

        assert_eq!(
            caso.ejecutar("beto"),
            Err(ErrorServicioSocial::NoAutenticado)
        );
        assert!(servicio.llamadas.borrow().is_empty());
    }

    #[test]
    fn rechaza_nombre_invalido() {
        let servicio = servicio();
        let almacen = AlmacenFalso {
            sesion: Some(sesion()),
        };
        let caso = BuscarUsuario::new(&servicio, &almacen);

        for nombre in ["ab", "Beto", "a-b", "ana ", ""] {
            assert!(matches!(
                caso.ejecutar(nombre),
                Err(ErrorServicioSocial::Validacion { .. })
            ));
        }

        assert!(servicio.llamadas.borrow().is_empty());
    }

    #[test]
    fn propaga_usuario_no_encontrado() {
        let mut servicio = servicio();
        servicio.respuesta = Err(ErrorServicioSocial::NoEncontrado);

        let almacen = AlmacenFalso {
            sesion: Some(sesion()),
        };
        let caso = BuscarUsuario::new(&servicio, &almacen);

        assert_eq!(
            caso.ejecutar("beto"),
            Err(ErrorServicioSocial::NoEncontrado)
        );
    }

    #[test]
    fn propaga_error_de_conexion() {
        let mut servicio = servicio();
        servicio.respuesta =
            Err(ErrorServicioSocial::ServicioNoDisponible);

        let almacen = AlmacenFalso {
            sesion: Some(sesion()),
        };
        let caso = BuscarUsuario::new(&servicio, &almacen);

        assert_eq!(
            caso.ejecutar("beto"),
            Err(ErrorServicioSocial::ServicioNoDisponible)
        );
    }
}
