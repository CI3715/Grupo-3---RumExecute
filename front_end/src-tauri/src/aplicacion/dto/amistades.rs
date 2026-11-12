use serde::{Deserialize, Serialize};


// POST /amistades
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EnviarInvitacionAmistadEntrada {
    pub nombre_usuario_receptor: String,
}


// Respuesta de POST /amistades
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InvitacionAmistadDto {
    pub id: String,

    #[serde(alias = "nombre_usuario_solicitante")]
    pub nombre_usuario_solicitante: String,

    #[serde(alias = "nombre_usuario_receptor")]
    pub nombre_usuario_receptor: String,

    pub estado: String,
}


// POST /amistades/{id}/respuesta
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ResponderInvitacionAmistadEntrada {
    pub aceptar: bool,
}


// Respuesta de POST /amistades/{id}/respuesta
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RespuestaAmistadDto {
    pub id: String,
    pub estado: String,
}


// Amistad activa en GET /amistades
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AmigoDto {
    #[serde(alias = "amistad_id")]
    pub amistad_id: String,

    #[serde(alias = "nombre_usuario")]
    pub nombre_usuario: String,

    #[serde(alias = "nombre_visible")]
    pub nombre_visible: String,
}


// Invitación pendiente en GET /amistades
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InvitacionAmistadResumenDto {
    #[serde(alias = "amistad_id")]
    pub amistad_id: String,

    #[serde(alias = "nombre_usuario")]
    pub nombre_usuario: String,

    #[serde(alias = "nombre_visible")]
    pub nombre_visible: String,
}


// Respuesta de GET /amistades
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListadoAmistadesDto {
    pub amigos: Vec<AmigoDto>,

    #[serde(alias = "invitaciones_recibidas")]
    pub invitaciones_recibidas: Vec<InvitacionAmistadResumenDto>,

    #[serde(alias = "invitaciones_enviadas")]
    pub invitaciones_enviadas: Vec<InvitacionAmistadResumenDto>,
}



#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{json, from_value, to_value};

    #[test]
    fn serializar_solicitud_amistad() {
        let entrada = EnviarInvitacionAmistadEntrada {
            nombre_usuario_receptor: "beto".to_string(),
        };

        let json = to_value(entrada).unwrap();

        assert_eq!(json["nombreUsuarioReceptor"], "beto");
    }

    #[test]
    fn deserializar_invitacion_creada() {
        let json = json!({
            "id": "amistad-1",
            "nombre_usuario_solicitante": "ana",
            "nombre_usuario_receptor": "beto",
            "estado": "PENDIENTE"
        });

        let invitacion: InvitacionAmistadDto =
            from_value(json).unwrap();

        assert_eq!(invitacion.id, "amistad-1");
        assert_eq!(invitacion.nombre_usuario_receptor, "beto");
        assert_eq!(invitacion.estado, "PENDIENTE");
    }

    #[test]
    fn serializar_respuesta_amistad() {
        let entrada = ResponderInvitacionAmistadEntrada {
            aceptar: true,
        };

        let json = to_value(entrada).unwrap();

        assert_eq!(json["aceptar"], true);
    }

    #[test]
    fn deserializar_respuesta_amistad() {
        let json = json!({
            "id": "amistad-1",
            "estado": "ACTIVA"
        });

        let respuesta: RespuestaAmistadDto =
            from_value(json).unwrap();

        assert_eq!(respuesta.id, "amistad-1");
        assert_eq!(respuesta.estado, "ACTIVA");
    }

    #[test]
    fn deserializar_listado_amistades() {
        let json = json!({
            "amigos": [
                {
                    "amistad_id": "amistad-1",
                    "nombre_usuario": "beto",
                    "nombre_visible": "Beto"
                }
            ],
            "invitaciones_recibidas": [
                {
                    "amistad_id": "amistad-2",
                    "nombre_usuario": "carla",
                    "nombre_visible": "Carla"
                }
            ],
            "invitaciones_enviadas": []
        });

        let resultado: ListadoAmistadesDto =
            from_value(json).unwrap();

        assert_eq!(resultado.amigos.len(), 1);
        assert_eq!(resultado.amigos[0].nombre_usuario, "beto");

        assert_eq!(resultado.invitaciones_recibidas.len(), 1);
        assert_eq!(
            resultado.invitaciones_recibidas[0].nombre_usuario,
            "carla"
        );

        assert!(resultado.invitaciones_enviadas.is_empty());
    }

    #[test]
    fn listado_vacio() {
        let json = json!({
            "amigos": [],
            "invitaciones_recibidas": [],
            "invitaciones_enviadas": []
        });

        let resultado: ListadoAmistadesDto =
            from_value(json).unwrap();

        assert!(resultado.amigos.is_empty());
        assert!(resultado.invitaciones_recibidas.is_empty());
        assert!(resultado.invitaciones_enviadas.is_empty());
    }

    #[test]
    fn no_expone_datos_privados() {
        let amigo = AmigoDto {
            amistad_id: "amistad-1".to_string(),
            nombre_usuario: "beto".to_string(),
            nombre_visible: "Beto".to_string(),
        };

        let json = to_value(amigo).unwrap();

        assert!(json.get("correo").is_none());
        assert!(json.get("contrasena").is_none());
        assert!(json.get("token").is_none());
    }
}
