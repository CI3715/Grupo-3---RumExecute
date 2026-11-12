use crate::aplicacion::dto::grupos::{
    CrearGrupoEntrada,
    GrupoCreadoDto,
};
use crate::aplicacion::puertos::{
    AlmacenSesiones,
    ErrorSesion,
};
use crate::aplicacion::puertos::servicio_social::{
    ErrorServicioSocial,
    ServicioSocial,
};


pub struct CrearGrupo<'a> {
    servicio_social: &'a dyn ServicioSocial,
    almacen_sesiones: &'a dyn AlmacenSesiones,
}


impl<'a> CrearGrupo<'a> {
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
        nombre_grupo: &str,
        moneda: &str,
        descripcion: &str,
    ) -> Result<GrupoCreadoDto, ErrorServicioSocial> {

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

        // Validar nombre del grupo.
        if nombre_grupo.trim().is_empty() {
            return Err(ErrorServicioSocial::Validacion {
                campo: "nombre_grupo".to_string(),
                mensaje: "El nombre del grupo es obligatorio."
                    .to_string(),
            });
        }

        // Validar moneda ISO de tres letras mayúsculas.
        if moneda.len() != 3
            || !moneda.bytes().all(|b| b.is_ascii_uppercase())
        {
            return Err(ErrorServicioSocial::Validacion {
                campo: "moneda".to_string(),
                mensaje: "La moneda debe tener tres letras mayúsculas."
                    .to_string(),
            });
        }

        let entrada = CrearGrupoEntrada {
            nombre_grupo: nombre_grupo.trim().to_string(),
            moneda: moneda.to_string(),
            descripcion: descripcion.to_string(),
        };

        self.servicio_social.crear_grupo(
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

    fn almacen_con_sesion() -> AlmacenFalso {
        AlmacenFalso {
            sesion: Some(sesion_prueba()),
        }
    }

    #[test]
    fn crea_grupo_correctamente() {
        let servicio = ServicioSocialFalso::default();
        let almacen = almacen_con_sesion();

        let caso = CrearGrupo::new(&servicio, &almacen);

        let resultado = caso.ejecutar(
            "Viaje",
            "USD",
            "Vacaciones",
        ).unwrap();

        assert_eq!(resultado.id, "grupo-1");
        assert_eq!(resultado.nombre_grupo, "Viaje");
        assert_eq!(resultado.moneda, "USD");
        assert_eq!(resultado.descripcion, "Vacaciones");
        assert_eq!(resultado.administrador_id, "usuario-1");
    }

    #[test]
    fn envia_token_y_datos_correctos() {
        let servicio = ServicioSocialFalso::default();
        let almacen = almacen_con_sesion();

        let caso = CrearGrupo::new(&servicio, &almacen);

        caso.ejecutar("Viaje", "USD", "Vacaciones").unwrap();

        let llamadas = servicio.llamadas_crear_grupo.borrow();

        assert_eq!(llamadas.len(), 1);
        assert_eq!(llamadas[0].0, "token-prueba");
        assert_eq!(llamadas[0].1.nombre_grupo, "Viaje");
        assert_eq!(llamadas[0].1.moneda, "USD");
        assert_eq!(llamadas[0].1.descripcion, "Vacaciones");
    }

    #[test]
    fn normaliza_espacios_del_nombre() {
        let servicio = ServicioSocialFalso::default();
        let almacen = almacen_con_sesion();

        let caso = CrearGrupo::new(&servicio, &almacen);

        caso.ejecutar(
            "  Viaje  ",
            "USD",
            "Vacaciones",
        ).unwrap();

        let llamadas = servicio.llamadas_crear_grupo.borrow();

        assert_eq!(llamadas[0].1.nombre_grupo, "Viaje");
    }

    #[test]
    fn permite_descripcion_vacia() {
        let servicio = ServicioSocialFalso::default();
        let almacen = almacen_con_sesion();

        let caso = CrearGrupo::new(&servicio, &almacen);

        caso.ejecutar("Viaje", "USD", "").unwrap();

        let llamadas = servicio.llamadas_crear_grupo.borrow();

        assert_eq!(llamadas[0].1.descripcion, "");
    }

    #[test]
    fn requiere_sesion() {
        let servicio = ServicioSocialFalso::default();
        let almacen = AlmacenFalso { sesion: None };

        let caso = CrearGrupo::new(&servicio, &almacen);

        assert_eq!(
            caso.ejecutar("Viaje", "USD", ""),
            Err(ErrorServicioSocial::NoAutenticado)
        );

        assert!(
            servicio.llamadas_crear_grupo.borrow().is_empty()
        );
    }

    #[test]
    fn rechaza_nombre_vacio() {
        let servicio = ServicioSocialFalso::default();
        let almacen = almacen_con_sesion();

        let caso = CrearGrupo::new(&servicio, &almacen);

        for nombre in ["", "   "] {
            assert!(matches!(
                caso.ejecutar(nombre, "USD", ""),
                Err(ErrorServicioSocial::Validacion {
                    campo,
                    ..
                }) if campo == "nombre_grupo"
            ));
        }

        assert!(
            servicio.llamadas_crear_grupo.borrow().is_empty()
        );
    }

    #[test]
    fn rechaza_moneda_invalida() {
        let servicio = ServicioSocialFalso::default();
        let almacen = almacen_con_sesion();

        let caso = CrearGrupo::new(&servicio, &almacen);

        for moneda in ["usd", "US", "USDT", "U1D", ""] {
            assert!(matches!(
                caso.ejecutar("Viaje", moneda, ""),
                Err(ErrorServicioSocial::Validacion {
                    campo,
                    ..
                }) if campo == "moneda"
            ));
        }

        assert!(
            servicio.llamadas_crear_grupo.borrow().is_empty()
        );
    }

    #[test]
    fn propaga_conflicto_del_backend() {
        let mut servicio = ServicioSocialFalso::default();

        let conflicto = ErrorServicioSocial::Conflicto {
            campo: "grupo".into(),
            mensaje: "No se pudo crear el grupo.".into(),
        };

        servicio.respuesta_crear_grupo = Err(conflicto.clone());

        let almacen = almacen_con_sesion();
        let caso = CrearGrupo::new(&servicio, &almacen);

        assert_eq!(
            caso.ejecutar("Viaje", "USD", ""),
            Err(conflicto)
        );
    }

    #[test]
    fn propaga_error_de_conexion() {
        let mut servicio = ServicioSocialFalso::default();
        servicio.respuesta_crear_grupo =
            Err(ErrorServicioSocial::ServicioNoDisponible);

        let almacen = almacen_con_sesion();
        let caso = CrearGrupo::new(&servicio, &almacen);

        assert_eq!(
            caso.ejecutar("Viaje", "USD", ""),
            Err(ErrorServicioSocial::ServicioNoDisponible)
        );
    }

    #[test]
    fn propaga_sesion_expirada() {
        let mut servicio = ServicioSocialFalso::default();
        servicio.respuesta_crear_grupo =
            Err(ErrorServicioSocial::SesionExpirada);

        let almacen = almacen_con_sesion();
        let caso = CrearGrupo::new(&servicio, &almacen);

        assert_eq!(
            caso.ejecutar("Viaje", "USD", ""),
            Err(ErrorServicioSocial::SesionExpirada)
        );
    }
}
