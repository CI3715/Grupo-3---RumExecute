use crate::aplicacion::dto::gastos::DetalleGrupoDto;

use crate::aplicacion::puertos::{
    AlmacenSesiones,
    ErrorSesion,
};

use crate::aplicacion::puertos::servicio_social::{
    ErrorServicioSocial,
    ServicioSocial,
};

pub struct ObtenerDetalleGrupo<'a> {
    servicio_social: &'a dyn ServicioSocial,
    almacen_sesiones: &'a dyn AlmacenSesiones,
}

impl<'a> ObtenerDetalleGrupo<'a> {
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
    ) -> Result<DetalleGrupoDto, ErrorServicioSocial> {
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

        // Consultar el detalle mediante el puerto social.
        self.servicio_social.obtener_detalle_grupo(
            &sesion.token,
            grupo_id,
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
    fn obtiene_datos_del_grupo_correctamente() {
        let servicio = ServicioSocialFalso::default();
        let almacen = almacen_con_sesion();

        let caso = ObtenerDetalleGrupo::new(&servicio, &almacen);
        let resultado = caso.ejecutar("grupo-1").unwrap();

        assert_eq!(resultado.id, "grupo-1");
        assert_eq!(resultado.nombre_grupo, "Viaje");
        assert_eq!(resultado.moneda, "USD");
        assert_eq!(resultado.descripcion, "Vacaciones");
        assert_eq!(resultado.administrador_id, "usuario-1");
    }

    #[test]
    fn devuelve_miembros_y_administrador() {
        let servicio = ServicioSocialFalso::default();
        let almacen = almacen_con_sesion();

        let caso = ObtenerDetalleGrupo::new(&servicio, &almacen);
        let resultado = caso.ejecutar("grupo-1").unwrap();

        assert_eq!(resultado.miembros.len(), 2);

        let ana = &resultado.miembros[0];
        assert_eq!(ana.usuario_id, "usuario-1");
        assert_eq!(ana.nombre_usuario, "ana");
        assert_eq!(ana.nombre_visible, "Ana");
        assert!(ana.es_administrador);

        let beto = &resultado.miembros[1];
        assert_eq!(beto.usuario_id, "usuario-2");
        assert_eq!(beto.nombre_usuario, "beto");
        assert!(!beto.es_administrador);
    }

    #[test]
    fn devuelve_balances_sin_modificarlos() {
        let servicio = ServicioSocialFalso::default();
        let almacen = almacen_con_sesion();

        let caso = ObtenerDetalleGrupo::new(&servicio, &almacen);
        let resultado = caso.ejecutar("grupo-1").unwrap();

        assert_eq!(resultado.miembros[0].balance, "15.00");
        assert_eq!(resultado.miembros[1].balance, "-15.00");
    }

    #[test]
    fn devuelve_gastos_y_cuotas_correctamente() {
        let servicio = ServicioSocialFalso::default();
        let almacen = almacen_con_sesion();

        let caso = ObtenerDetalleGrupo::new(&servicio, &almacen);
        let resultado = caso.ejecutar("grupo-1").unwrap();

        assert_eq!(resultado.gastos.len(), 1);

        let gasto = &resultado.gastos[0];

        assert_eq!(gasto.id, "gasto-1");
        assert_eq!(gasto.grupo_id, "grupo-1");
        assert_eq!(gasto.concepto, "Cena");
        assert_eq!(gasto.pagador_id, "usuario-1");
        assert_eq!(gasto.monto, "30.00");
        assert_eq!(gasto.fecha, "2026-11-12T12:00:00Z");

        assert_eq!(gasto.cuotas.len(), 2);
        assert_eq!(gasto.cuotas[0].usuario_id, "usuario-1");
        assert_eq!(gasto.cuotas[0].monto, "15.00");
        assert_eq!(gasto.cuotas[1].usuario_id, "usuario-2");
        assert_eq!(gasto.cuotas[1].monto, "15.00");

        assert!(!gasto.cuotas[0].lleva_residuo);
        assert!(!gasto.cuotas[1].lleva_residuo);
    }

    #[test]
    fn envia_token_y_grupo_id_correctamente() {
        let servicio = ServicioSocialFalso::default();
        let almacen = almacen_con_sesion();

        let caso = ObtenerDetalleGrupo::new(&servicio, &almacen);

        caso.ejecutar("grupo-123").unwrap();

        assert_eq!(
            *servicio.llamadas_obtener_detalle_grupo.borrow(),
            vec![(
                "token-prueba".to_string(),
                "grupo-123".to_string(),
            )]
        );
    }

    #[test]
    fn permite_grupo_sin_gastos() {
        let mut servicio = ServicioSocialFalso::default();

        let mut detalle = servicio
            .respuesta_obtener_detalle_grupo
            .clone()
            .unwrap();

        detalle.gastos.clear();

        for miembro in &mut detalle.miembros {
            miembro.balance = "0.00".into();
        }

        servicio.respuesta_obtener_detalle_grupo = Ok(detalle);

        let almacen = almacen_con_sesion();
        let caso = ObtenerDetalleGrupo::new(&servicio, &almacen);

        let resultado = caso.ejecutar("grupo-1").unwrap();

        assert!(resultado.gastos.is_empty());
        assert_eq!(resultado.miembros.len(), 2);
        assert!(resultado.miembros.iter().all(|m| m.balance == "0.00"));
    }

    #[test]
    fn requiere_sesion() {
        let servicio = ServicioSocialFalso::default();
        let almacen = AlmacenFalso {
            sesion: None,
        };

        let caso = ObtenerDetalleGrupo::new(&servicio, &almacen);

        assert_eq!(
            caso.ejecutar("grupo-1"),
            Err(ErrorServicioSocial::NoAutenticado)
        );

        assert!(
            servicio.llamadas_obtener_detalle_grupo
                .borrow()
                .is_empty()
        );
    }

    #[test]
    fn rechaza_grupo_id_vacio() {
        let servicio = ServicioSocialFalso::default();
        let almacen = almacen_con_sesion();

        let caso = ObtenerDetalleGrupo::new(&servicio, &almacen);

        for grupo_id in ["", "   "] {
            assert!(matches!(
                caso.ejecutar(grupo_id),
                Err(ErrorServicioSocial::Validacion {
                    campo,
                    ..
                }) if campo == "grupo_id"
            ));
        }

        assert!(
            servicio.llamadas_obtener_detalle_grupo
                .borrow()
                .is_empty()
        );
    }

    #[test]
    fn propaga_grupo_no_encontrado() {
        let mut servicio = ServicioSocialFalso::default();

        servicio.respuesta_obtener_detalle_grupo =
            Err(ErrorServicioSocial::NoEncontrado);

        let almacen = almacen_con_sesion();
        let caso = ObtenerDetalleGrupo::new(&servicio, &almacen);

        assert_eq!(
            caso.ejecutar("grupo-1"),
            Err(ErrorServicioSocial::NoEncontrado)
        );
    }

    #[test]
    fn propaga_error_de_permiso() {
        let mut servicio = ServicioSocialFalso::default();

        servicio.respuesta_obtener_detalle_grupo =
            Err(ErrorServicioSocial::SinPermiso);

        let almacen = almacen_con_sesion();
        let caso = ObtenerDetalleGrupo::new(&servicio, &almacen);

        assert_eq!(
            caso.ejecutar("grupo-1"),
            Err(ErrorServicioSocial::SinPermiso)
        );
    }

    #[test]
    fn propaga_error_de_conexion() {
        let mut servicio = ServicioSocialFalso::default();

        servicio.respuesta_obtener_detalle_grupo =
            Err(ErrorServicioSocial::ServicioNoDisponible);

        let almacen = almacen_con_sesion();
        let caso = ObtenerDetalleGrupo::new(&servicio, &almacen);

        assert_eq!(
            caso.ejecutar("grupo-1"),
            Err(ErrorServicioSocial::ServicioNoDisponible)
        );
    }

    #[test]
    fn propaga_sesion_expirada() {
        let mut servicio = ServicioSocialFalso::default();

        servicio.respuesta_obtener_detalle_grupo =
            Err(ErrorServicioSocial::SesionExpirada);

        let almacen = almacen_con_sesion();
        let caso = ObtenerDetalleGrupo::new(&servicio, &almacen);

        assert_eq!(
            caso.ejecutar("grupo-1"),
            Err(ErrorServicioSocial::SesionExpirada)
        );
    }

    #[test]
    fn propaga_error_tecnico() {
        let mut servicio = ServicioSocialFalso::default();

        let error = ErrorServicioSocial::ErrorTecnico(
            "No se pudo procesar el detalle del grupo.".into()
        );

        servicio.respuesta_obtener_detalle_grupo =
            Err(error.clone());

        let almacen = almacen_con_sesion();
        let caso = ObtenerDetalleGrupo::new(&servicio, &almacen);

        assert_eq!(caso.ejecutar("grupo-1"), Err(error));
    }
}

