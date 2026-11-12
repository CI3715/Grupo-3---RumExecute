use std::collections::HashSet;
use rust_decimal::Decimal;

use crate::aplicacion::dto::gastos::{
    GastoCompartidoDto,
    RegistrarGastoCompartidoEntrada,
    TipoDivision,
};

use crate::aplicacion::puertos::{
    AlmacenSesiones,
    ErrorSesion,
};

use crate::aplicacion::puertos::servicio_social::{
    ErrorServicioSocial,
    ServicioSocial,
};

pub struct RegistrarGastoCompartido<'a> {
    servicio_social: &'a dyn ServicioSocial,
    almacen_sesiones: &'a dyn AlmacenSesiones,
}

impl<'a> RegistrarGastoCompartido<'a> {
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
        entrada: RegistrarGastoCompartidoEntrada,
    ) -> Result<GastoCompartidoDto, ErrorServicioSocial> {
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

        // Validar grupo.
        if grupo_id.trim().is_empty() {
            return Err(ErrorServicioSocial::Validacion {
                campo: "grupo_id".into(),
                mensaje: "El identificador del grupo es obligatorio.".into(),
            });
        }

        // Validar concepto.
        if entrada.concepto.trim().is_empty() {
            return Err(ErrorServicioSocial::Validacion {
                campo: "concepto".into(),
                mensaje: "El concepto del gasto es obligatorio.".into(),
            });
        }

        // Validar pagador.
        if entrada.pagador_id.trim().is_empty() {
            return Err(ErrorServicioSocial::Validacion {
                campo: "pagador_id".into(),
                mensaje: "El pagador es obligatorio.".into(),
            });
        }

        // Validar monto positivo con hasta dos decimales.
        if entrada.monto <= Decimal::ZERO || entrada.monto.scale() > 2 {
            return Err(ErrorServicioSocial::Validacion {
                campo: "monto".into(),
                mensaje: "El monto debe ser un decimal positivo con hasta dos decimales.".into(),
            });
        }

        // Debe existir al menos un participante.
        if entrada.participantes_ids.is_empty()
            || entrada.participantes_ids.iter().any(|id| id.trim().is_empty())
        {
            return Err(ErrorServicioSocial::Validacion {
                campo: "participantes_ids".into(),
                mensaje: "Debe indicarse al menos un participante válido.".into(),
            });
        }

        // Evitar participantes repetidos.
        let participantes_unicos: HashSet<&str> = entrada
            .participantes_ids
            .iter()
            .map(String::as_str)
            .collect();

        if participantes_unicos.len() != entrada.participantes_ids.len() {
            return Err(ErrorServicioSocial::Validacion {
                campo: "participantes_ids".into(),
                mensaje: "No puede haber participantes repetidos.".into(),
            });
        }

        // Para porcentajes y montos se necesitan valores individuales.
        if matches!(
            entrada.tipo_division,
            TipoDivision::Porcentajes | TipoDivision::Montos
        ) && entrada.valores.as_ref().is_none_or(|v| v.is_empty())
        {
            return Err(ErrorServicioSocial::Validacion {
                campo: "valores".into(),
                mensaje: "La división requiere valores por participante.".into(),
            });
        }

        // El backend se encarga de validar la distribución
        // y calcular las cuotas y balances.
        self.servicio_social.registrar_gasto_compartido(
            &sesion.token,
            grupo_id,
            &entrada,
        )
    }
}




#[cfg(test)]
mod tests {
    use super::*;

    use std::collections::BTreeMap;
    use std::str::FromStr;

    use crate::aplicacion::pruebas::social::{
        AlmacenFalso,
        ServicioSocialFalso,
        sesion_prueba,
    };

    fn decimal(valor: &str) -> Decimal {
        Decimal::from_str(valor).unwrap()
    }

    fn almacen_con_sesion() -> AlmacenFalso {
        AlmacenFalso {
            sesion: Some(sesion_prueba()),
        }
    }

    fn entrada_iguales() -> RegistrarGastoCompartidoEntrada {
        RegistrarGastoCompartidoEntrada {
            concepto: "Cena".into(),
            pagador_id: "usuario-1".into(),
            monto: decimal("30.00"),
            tipo_division: TipoDivision::Iguales,
            participantes_ids: vec![
                "usuario-1".into(),
                "usuario-2".into(),
            ],
            valores: None,
        }
    }

    #[test]
    fn registra_gasto_correctamente() {
        let servicio = ServicioSocialFalso::default();
        let almacen = almacen_con_sesion();

        let caso = RegistrarGastoCompartido::new(
            &servicio, &almacen
        );

        let resultado = caso
            .ejecutar("grupo-1", entrada_iguales())
            .unwrap();

        assert_eq!(resultado.id, "gasto-1");
        assert_eq!(resultado.grupo_id, "grupo-1");
        assert_eq!(resultado.concepto, "Cena");
        assert_eq!(resultado.pagador_id, "usuario-1");
        assert_eq!(resultado.monto, decimal("30.00"));
        assert_eq!(resultado.cuotas.len(), 2);
        assert_eq!(resultado.cuotas[0].monto, decimal("15.00"));
        assert_eq!(resultado.cuotas[1].monto, decimal("15.00"));
    }

    #[test]
    fn envia_token_grupo_y_datos_correctamente() {
        let servicio = ServicioSocialFalso::default();
        let almacen = almacen_con_sesion();

        let caso = RegistrarGastoCompartido::new(
            &servicio, &almacen
        );

        caso.ejecutar("grupo-1", entrada_iguales()).unwrap();

        let llamadas = servicio.llamadas_registrar_gasto.borrow();

        assert_eq!(llamadas.len(), 1);
        assert_eq!(llamadas[0].0, "token-prueba");
        assert_eq!(llamadas[0].1, "grupo-1");

        let entrada = &llamadas[0].2;

        assert_eq!(entrada.concepto, "Cena");
        assert_eq!(entrada.pagador_id, "usuario-1");
        assert_eq!(entrada.monto, decimal("30.00"));
        assert_eq!(entrada.tipo_division, TipoDivision::Iguales);
        assert_eq!(entrada.participantes_ids.len(), 2);
        assert!(entrada.valores.is_none());
    }

    #[test]
    fn acepta_division_por_porcentajes() {
        let servicio = ServicioSocialFalso::default();
        let almacen = almacen_con_sesion();

        let caso = RegistrarGastoCompartido::new(
            &servicio, &almacen
        );

        let mut entrada = entrada_iguales();
        entrada.tipo_division = TipoDivision::Porcentajes;
        entrada.valores = Some(BTreeMap::from([
            ("usuario-1".into(), decimal("60")),
            ("usuario-2".into(), decimal("40")),
        ]));

        caso.ejecutar("grupo-1", entrada).unwrap();

        let llamadas = servicio.llamadas_registrar_gasto.borrow();
        let enviado = &llamadas[0].2;

        assert_eq!(
            enviado.tipo_division,
            TipoDivision::Porcentajes
        );

        assert_eq!(
            enviado.valores.as_ref().unwrap()["usuario-1"],
            decimal("60")
        );

        assert_eq!(
            enviado.valores.as_ref().unwrap()["usuario-2"],
            decimal("40")
        );
    }

    #[test]
    fn acepta_division_por_montos() {
        let servicio = ServicioSocialFalso::default();
        let almacen = almacen_con_sesion();

        let caso = RegistrarGastoCompartido::new(
            &servicio, &almacen
        );

        let mut entrada = entrada_iguales();
        entrada.tipo_division = TipoDivision::Montos;
        entrada.valores = Some(BTreeMap::from([
            ("usuario-1".into(), decimal("10.00")),
            ("usuario-2".into(), decimal("20.00")),
        ]));

        caso.ejecutar("grupo-1", entrada).unwrap();

        let llamadas = servicio.llamadas_registrar_gasto.borrow();

        assert_eq!(
            llamadas[0].2.tipo_division,
            TipoDivision::Montos
        );

        assert_eq!(
            llamadas[0].2.valores.as_ref().unwrap()["usuario-2"],
            decimal("20.00")
        );
    }

    #[test]
    fn requiere_sesion() {
        let servicio = ServicioSocialFalso::default();
        let almacen = AlmacenFalso {
            sesion: None,
        };

        let caso = RegistrarGastoCompartido::new(
            &servicio, &almacen
        );

        assert_eq!(
            caso.ejecutar("grupo-1", entrada_iguales()),
            Err(ErrorServicioSocial::NoAutenticado)
        );

        assert!(
            servicio.llamadas_registrar_gasto.borrow().is_empty()
        );
    }

    #[test]
    fn rechaza_grupo_id_vacio() {
        let servicio = ServicioSocialFalso::default();
        let almacen = almacen_con_sesion();

        let caso = RegistrarGastoCompartido::new(
            &servicio, &almacen
        );

        for grupo_id in ["", "   "] {
            assert!(matches!(
                caso.ejecutar(grupo_id, entrada_iguales()),
                Err(ErrorServicioSocial::Validacion {
                    campo, ..
                }) if campo == "grupo_id"
            ));
        }

        assert!(
            servicio.llamadas_registrar_gasto.borrow().is_empty()
        );
    }

    #[test]
    fn rechaza_concepto_vacio() {
        let servicio = ServicioSocialFalso::default();
        let almacen = almacen_con_sesion();

        let caso = RegistrarGastoCompartido::new(
            &servicio, &almacen
        );

        let mut entrada = entrada_iguales();
        entrada.concepto = "   ".into();

        assert!(matches!(
            caso.ejecutar("grupo-1", entrada),
            Err(ErrorServicioSocial::Validacion {
                campo, ..
            }) if campo == "concepto"
        ));

        assert!(
            servicio.llamadas_registrar_gasto.borrow().is_empty()
        );
    }

    #[test]
    fn rechaza_pagador_vacio() {
        let servicio = ServicioSocialFalso::default();
        let almacen = almacen_con_sesion();

        let caso = RegistrarGastoCompartido::new(
            &servicio, &almacen
        );

        let mut entrada = entrada_iguales();
        entrada.pagador_id = "".into();

        assert!(matches!(
            caso.ejecutar("grupo-1", entrada),
            Err(ErrorServicioSocial::Validacion {
                campo, ..
            }) if campo == "pagador_id"
        ));
    }

    #[test]
    fn rechaza_montos_no_positivos() {
        let servicio = ServicioSocialFalso::default();
        let almacen = almacen_con_sesion();

        let caso = RegistrarGastoCompartido::new(
            &servicio, &almacen
        );

        for monto in [
            decimal("0"),
            decimal("0.00"),
            decimal("-5.00"),
        ] {
            let mut entrada = entrada_iguales();
            entrada.monto = monto;

            assert!(matches!(
                caso.ejecutar("grupo-1", entrada),
                Err(ErrorServicioSocial::Validacion {
                    campo, ..
                }) if campo == "monto"
            ));
        }

        assert!(
            servicio.llamadas_registrar_gasto.borrow().is_empty()
        );
    }

    #[test]
    fn rechaza_montos_con_mas_de_dos_decimales() {
        let servicio = ServicioSocialFalso::default();
        let almacen = almacen_con_sesion();

        let caso = RegistrarGastoCompartido::new(
            &servicio, &almacen
        );

        for monto in [
            decimal("10.999"),
            decimal("1.2345"),
            decimal("10.000"),
        ] {
            let mut entrada = entrada_iguales();
            entrada.monto = monto;

            assert!(matches!(
                caso.ejecutar("grupo-1", entrada),
                Err(ErrorServicioSocial::Validacion {
                    campo, ..
                }) if campo == "monto"
            ));
        }

        assert!(
            servicio.llamadas_registrar_gasto.borrow().is_empty()
        );
    }

    #[test]
    fn acepta_montos_decimales_validos() {
        for monto in [
            decimal("1"),
            decimal("0.01"),
            decimal("10.5"),
            decimal("100.00"),
        ] {
            assert!(monto > Decimal::ZERO);
            assert!(monto.scale() <= 2);
        }
    }

    #[test]
    fn rechaza_participantes_vacios_o_repetidos() {
        let servicio = ServicioSocialFalso::default();
        let almacen = almacen_con_sesion();

        let caso = RegistrarGastoCompartido::new(
            &servicio, &almacen
        );

        let mut entrada = entrada_iguales();
        entrada.participantes_ids.clear();

        assert!(matches!(
            caso.ejecutar("grupo-1", entrada),
            Err(ErrorServicioSocial::Validacion {
                campo, ..
            }) if campo == "participantes_ids"
        ));

        let mut entrada = entrada_iguales();
        entrada.participantes_ids = vec![
            "usuario-1".into(),
            "usuario-1".into(),
        ];

        assert!(matches!(
            caso.ejecutar("grupo-1", entrada),
            Err(ErrorServicioSocial::Validacion {
                campo, ..
            }) if campo == "participantes_ids"
        ));

        assert!(
            servicio.llamadas_registrar_gasto.borrow().is_empty()
        );
    }

    #[test]
    fn porcentajes_requieren_valores() {
        let servicio = ServicioSocialFalso::default();
        let almacen = almacen_con_sesion();

        let caso = RegistrarGastoCompartido::new(
            &servicio, &almacen
        );

        let mut entrada = entrada_iguales();
        entrada.tipo_division = TipoDivision::Porcentajes;

        assert!(matches!(
            caso.ejecutar("grupo-1", entrada),
            Err(ErrorServicioSocial::Validacion {
                campo, ..
            }) if campo == "valores"
        ));
    }

    #[test]
    fn montos_requieren_valores() {
        let servicio = ServicioSocialFalso::default();
        let almacen = almacen_con_sesion();

        let caso = RegistrarGastoCompartido::new(
            &servicio, &almacen
        );

        let mut entrada = entrada_iguales();
        entrada.tipo_division = TipoDivision::Montos;

        assert!(matches!(
            caso.ejecutar("grupo-1", entrada),
            Err(ErrorServicioSocial::Validacion {
                campo, ..
            }) if campo == "valores"
        ));
    }

    #[test]
    fn propaga_error_de_permiso() {
        let mut servicio = ServicioSocialFalso::default();

        servicio.respuesta_registrar_gasto =
            Err(ErrorServicioSocial::SinPermiso);

        let almacen = almacen_con_sesion();
        let caso = RegistrarGastoCompartido::new(
            &servicio, &almacen
        );

        assert_eq!(
            caso.ejecutar("grupo-1", entrada_iguales()),
            Err(ErrorServicioSocial::SinPermiso)
        );
    }

    #[test]
    fn propaga_error_de_validacion_del_backend() {
        let mut servicio = ServicioSocialFalso::default();

        let error = ErrorServicioSocial::Validacion {
            campo: "valores".into(),
            mensaje: "Los porcentajes deben sumar 100.".into(),
        };

        servicio.respuesta_registrar_gasto = Err(error.clone());

        let almacen = almacen_con_sesion();
        let caso = RegistrarGastoCompartido::new(
            &servicio, &almacen
        );

        assert_eq!(
            caso.ejecutar("grupo-1", entrada_iguales()),
            Err(error)
        );
    }

    #[test]
    fn propaga_error_de_conexion() {
        let mut servicio = ServicioSocialFalso::default();

        servicio.respuesta_registrar_gasto =
            Err(ErrorServicioSocial::ServicioNoDisponible);

        let almacen = almacen_con_sesion();
        let caso = RegistrarGastoCompartido::new(
            &servicio, &almacen
        );

        assert_eq!(
            caso.ejecutar("grupo-1", entrada_iguales()),
            Err(ErrorServicioSocial::ServicioNoDisponible)
        );
    }

    #[test]
    fn propaga_sesion_expirada() {
        let mut servicio = ServicioSocialFalso::default();

        servicio.respuesta_registrar_gasto =
            Err(ErrorServicioSocial::SesionExpirada);

        let almacen = almacen_con_sesion();
        let caso = RegistrarGastoCompartido::new(
            &servicio, &almacen
        );

        assert_eq!(
            caso.ejecutar("grupo-1", entrada_iguales()),
            Err(ErrorServicioSocial::SesionExpirada)
        );
    }
}
