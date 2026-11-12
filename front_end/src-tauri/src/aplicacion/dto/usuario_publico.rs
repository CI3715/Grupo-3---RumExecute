use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UsuarioPublicoDto {
    #[serde(alias = "nombre_usuario")]
    pub nombre_usuario: String,

    #[serde(alias = "nombre_visible")]
    pub nombre_visible: String,
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deserializar_usuario_publico_snake_case() {
        let json = r#"{
            "nombre_usuario": "ana",
            "nombre_visible": "Ana García"
        }"#;

        let usuario: UsuarioPublicoDto =
            serde_json::from_str(json).unwrap();

        assert_eq!(usuario.nombre_usuario, "ana");
        assert_eq!(usuario.nombre_visible, "Ana García");
    }

    #[test]
    fn deserializar_usuario_publico_camel_case() {
        let json = r#"{
            "nombreUsuario": "ana",
            "nombreVisible": "Ana García"
        }"#;

        let usuario: UsuarioPublicoDto =
            serde_json::from_str(json).unwrap();

        assert_eq!(usuario.nombre_usuario, "ana");
        assert_eq!(usuario.nombre_visible, "Ana García");
    }

    #[test]
    fn serializar_usuario_publico() {
        let usuario = UsuarioPublicoDto {
            nombre_usuario: "ana".to_string(),
            nombre_visible: "Ana García".to_string(),
        };

        let json = serde_json::to_value(usuario).unwrap();

        assert_eq!(json["nombreUsuario"], "ana");
        assert_eq!(json["nombreVisible"], "Ana García");
    }

    #[test]
    fn usuario_publico_no_incluye_datos_privados() {
        let usuario = UsuarioPublicoDto {
            nombre_usuario: "ana".to_string(),
            nombre_visible: "Ana García".to_string(),
        };

        let json = serde_json::to_value(usuario).unwrap();

        assert!(json.get("correo").is_none());
        assert!(json.get("contrasena").is_none());
        assert!(json.get("token").is_none());
    }
}
