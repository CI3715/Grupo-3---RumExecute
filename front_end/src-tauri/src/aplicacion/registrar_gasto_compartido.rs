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

        // Validar monto como texto decimal positivo.
        // No usamos f32 ni f64 para evitar pérdida de precisión.
        if !monto_valido(&entrada.monto) {
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
        let participantes_unicos: std::collections::HashSet<&str> =
            entrada.participantes_ids.iter()
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

// Validación textual de dinero sin convertirlo a float.
// Acepta, por ejemplo: "10", "10.5", "10.50".
fn monto_valido(monto: &str) -> bool {
    let partes: Vec<&str> = monto.split('.').collect();

    if partes.is_empty() || partes.len() > 2 {
        return false;
    }

    let entero = partes[0];

    if entero.is_empty() || !entero.bytes().all(|b| b.is_ascii_digit()) {
        return false;
    }

    if partes.len() == 2 {
        let decimales = partes[1];

        if decimales.is_empty()
            || decimales.len() > 2
            || !decimales.bytes().all(|b| b.is_ascii_digit())
        {
            return false;
        }
    }

    // Debe tener al menos un dígito distinto de cero.
    monto.bytes().any(|b| b.is_ascii_digit() && b != b'0')
}




#[cfg(test)]
mod tests {
    use super::*;

    use std::collections::BTreeMap;

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

    fn entrada_iguales() -> RegistrarGastoCompartidoEntrada {
        RegistrarGastoCompartidoEntrada {
            concepto: "Cena".into(),
            pagador_id: "usuario-1".into(),
            monto: "30.00".into(),
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
        assert_eq!(resultado.monto, "30.00");
        assert_eq!(resultado.cuotas.len(), 2);
        assert_eq!(resultado.cuotas[0].monto, "15.00");
        assert_eq!(resultado.cuotas[1].monto, "15.00");
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
        assert_eq!(entrada.monto, "30.00");
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
            ("usuario-1".into(), "60".into()),
            ("usuario-2".into(), "40".into()),
        ]));

        caso.ejecutar("grupo-1", entrada).unwrap();

        let llamadas = servicio.llamadas_registrar_gasto.borrow();
        let enviado = &llamadas[0].2;

        assert_eq!(enviado.tipo_division, TipoDivision::Porcentajes);
        assert_eq!(
            enviado.valores.as_ref().unwrap()["usuario-1"],
            "60"
        );
        assert_eq!(
            enviado.valores.as_ref().unwrap()["usuario-2"],
            "40"
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
            ("usuario-1".into(), "10.00".into()),
            ("usuario-2".into(), "20.00".into()),
        ]));

        caso.ejecutar("grupo-1", entrada).unwrap();

        let llamadas = servicio.llamadas_registrar_gasto.borrow();

        assert_eq!(
            llamadas[0].2.tipo_division,
            TipoDivision::Montos
        );
        assert_eq!(
            llamadas[0].2.valores.as_ref().unwrap()["usuario-2"],
            "20.00"
        );
    }

    #[test]
    fn requiere_sesion() {
        let servicio = ServicioSocialFalso::default();
        let almacen = AlmacenFalso { sesion: None };

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
    fn rechaza_montos_invalidos() {
        let servicio = ServicioSocialFalso::default();
        let almacen = almacen_con_sesion();

        let caso = RegistrarGastoCompartido::new(
            &servicio, &almacen
        );

        for monto in [
            "", "0", "0.00", "-5.00",
            "abc", "10.999", "1,50", "10.",
        ] {
            let mut entrada = entrada_iguales();
            entrada.monto = monto.into();

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
        for monto in ["1", "0.01", "10.5", "100.00"] {
            assert!(monto_valido(monto));
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
