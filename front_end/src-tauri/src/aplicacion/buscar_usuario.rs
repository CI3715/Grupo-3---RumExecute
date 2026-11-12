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

    use crate::aplicacion::pruebas::social::{
        AlmacenFalso,
        ServicioSocialFalso,
        sesion_prueba,
    };

    #[test]
    fn busca_usuario_correctamente() {
        let servicio = ServicioSocialFalso::default();
        let almacen = AlmacenFalso {
            sesion: Some(sesion_prueba()),
        };

        let caso = BuscarUsuario::new(&servicio, &almacen);

        let resultado = caso.ejecutar("beto").unwrap();

        assert_eq!(resultado.nombre_usuario, "beto");
        assert_eq!(resultado.nombre_visible, "Beto");

        assert_eq!(
            *servicio.llamadas_buscar.borrow(),
            vec![(
                "token-prueba".to_string(),
                "beto".to_string(),
            )]
        );
    }

    #[test]
    fn requiere_sesion() {
        let servicio = ServicioSocialFalso::default();
        let almacen = AlmacenFalso {
            sesion: None,
        };

        let caso = BuscarUsuario::new(&servicio, &almacen);

        assert_eq!(
            caso.ejecutar("beto"),
            Err(ErrorServicioSocial::NoAutenticado)
        );

        assert!(servicio.llamadas_buscar.borrow().is_empty());
    }

    #[test]
    fn rechaza_nombre_invalido() {
        let servicio = ServicioSocialFalso::default();
        let almacen = AlmacenFalso {
            sesion: Some(sesion_prueba()),
        };

        let caso = BuscarUsuario::new(&servicio, &almacen);

        for nombre in ["ab", "Beto", "a-b", "ana ", ""] {
            assert!(matches!(
                caso.ejecutar(nombre),
                Err(ErrorServicioSocial::Validacion { .. })
            ));
        }

        assert!(servicio.llamadas_buscar.borrow().is_empty());
    }

    #[test]
    fn acepta_nombres_validos() {
        let servicio = ServicioSocialFalso::default();
        let almacen = AlmacenFalso {
            sesion: Some(sesion_prueba()),
        };

        let caso = BuscarUsuario::new(&servicio, &almacen);

        for nombre in ["ana", "beto_123", "usuario.test"] {
            assert!(caso.ejecutar(nombre).is_ok());
        }

        assert_eq!(
            servicio.llamadas_buscar.borrow().len(),
            3
        );
    }

    #[test]
    fn propaga_usuario_no_encontrado() {
        let mut servicio = ServicioSocialFalso::default();

        servicio.respuesta_buscar =
            Err(ErrorServicioSocial::NoEncontrado);

        let almacen = AlmacenFalso {
            sesion: Some(sesion_prueba()),
        };

        let caso = BuscarUsuario::new(&servicio, &almacen);

        assert_eq!(
            caso.ejecutar("beto"),
            Err(ErrorServicioSocial::NoEncontrado)
        );
    }

    #[test]
    fn propaga_error_de_conexion() {
        let mut servicio = ServicioSocialFalso::default();

        servicio.respuesta_buscar =
            Err(ErrorServicioSocial::ServicioNoDisponible);

        let almacen = AlmacenFalso {
            sesion: Some(sesion_prueba()),
        };

        let caso = BuscarUsuario::new(&servicio, &almacen);

        assert_eq!(
            caso.ejecutar("beto"),
            Err(ErrorServicioSocial::ServicioNoDisponible)
        );
    }

    #[test]
    fn propaga_sesion_expirada() {
        let mut servicio = ServicioSocialFalso::default();

        servicio.respuesta_buscar =
            Err(ErrorServicioSocial::SesionExpirada);

        let almacen = AlmacenFalso {
            sesion: Some(sesion_prueba()),
        };

        let caso = BuscarUsuario::new(&servicio, &almacen);

        assert_eq!(
            caso.ejecutar("beto"),
            Err(ErrorServicioSocial::SesionExpirada)
        );
    }

    #[test]
    fn envia_nombre_exacto_sin_modificarlo() {
        let servicio = ServicioSocialFalso::default();
        let almacen = AlmacenFalso {
            sesion: Some(sesion_prueba()),
        };

        let caso = BuscarUsuario::new(&servicio, &almacen);

        caso.ejecutar("beto_123").unwrap();

        let llamadas = servicio.llamadas_buscar.borrow();

        assert_eq!(llamadas.len(), 1);
        assert_eq!(llamadas[0].0, "token-prueba");
        assert_eq!(llamadas[0].1, "beto_123");
    }
}
