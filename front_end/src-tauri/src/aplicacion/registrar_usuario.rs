use crate::aplicacion::dto::{
    RegistrarUsuarioEntrada,
    UsuarioRegistradoDto,
};

use crate::aplicacion::puertos::{
    ErrorAutenticacion,
    ServicioAutenticacion,
};

pub fn registrar_usuario(
    servicio: &dyn ServicioAutenticacion,
    entrada: RegistrarUsuarioEntrada,
) -> Result<UsuarioRegistradoDto, ErrorAutenticacion> {
    servicio.registrar_usuario(&entrada)
}


#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::Cell;

    struct ServicioFalso {
        fue_invocado: Cell<bool>,
        resultado: Result<UsuarioRegistradoDto, ErrorAutenticacion>,
    }

    impl ServicioAutenticacion for ServicioFalso {
        fn registrar_usuario(
            &self,
            _entrada: &RegistrarUsuarioEntrada,
        ) -> Result<UsuarioRegistradoDto, ErrorAutenticacion> {
            self.fue_invocado.set(true);
            self.resultado.clone()
        }
    }

    fn entrada_valida() -> RegistrarUsuarioEntrada {
        RegistrarUsuarioEntrada {
            nombre_usuario: "eduardo".to_string(),
            nombre_visible: "Eduardo".to_string(),
            correo: "eduardo@example.com".to_string(),
            contrasena: "contrasena123".to_string(),
        }
    }

    fn usuario_registrado() -> UsuarioRegistradoDto {
        UsuarioRegistradoDto {
            id: "usuario-123".to_string(),
            nombre_usuario: "eduardo".to_string(),
            nombre_visible: "Eduardo".to_string(),
            correo: "eduardo@example.com".to_string(),
            fecha_alta: "2026-11-11T12:00:00Z".to_string(),
        }
    }

    #[test]
    fn registra_usuario_exitosamente() {
        let esperado = usuario_registrado();

        let servicio = ServicioFalso {
            fue_invocado: Cell::new(false),
            resultado: Ok(esperado.clone()),
        };

        let resultado = registrar_usuario(
            &servicio,
            entrada_valida(),
        );

        assert_eq!(resultado.unwrap(), esperado);
        assert!(servicio.fue_invocado.get());
    }

    #[test]
    fn propaga_error_usuario_duplicado() {
        let servicio = ServicioFalso {
            fue_invocado: Cell::new(false),
            resultado: Err(
                ErrorAutenticacion::UsuarioYaExistente {
                    campo: "correo".to_string(),
                },
            ),
        };

        let resultado = registrar_usuario(
            &servicio,
            entrada_valida(),
        );

        assert_eq!(
            resultado.unwrap_err(),
            ErrorAutenticacion::UsuarioYaExistente {
                campo: "correo".to_string(),
            }
        );

        assert!(servicio.fue_invocado.get());
    }

    #[test]
    fn propaga_error_servicio_no_disponible() {
        let servicio = ServicioFalso {
            fue_invocado: Cell::new(false),
            resultado: Err(
                ErrorAutenticacion::ServicioNoDisponible,
            ),
        };

        let resultado = registrar_usuario(
            &servicio,
            entrada_valida(),
        );

        assert_eq!(
            resultado.unwrap_err(),
            ErrorAutenticacion::ServicioNoDisponible
        );
    }
}