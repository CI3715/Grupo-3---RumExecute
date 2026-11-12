use serde::{Deserialize, Serialize};


// POST /grupos
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CrearGrupoEntrada {
    pub nombre_grupo: String,
    pub moneda: String,
    pub descripcion: String,
}


// Respuesta de POST /grupos
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GrupoCreadoDto {
    pub id: String,

    #[serde(alias = "nombre_grupo")]
    pub nombre_grupo: String,

    pub moneda: String,

    #[serde(alias = "administrador_id")]
    pub administrador_id: String,

    pub descripcion: String,
}


// Grupo dentro de GET /grupos
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GrupoResumenDto {
    pub id: String,

    #[serde(alias = "nombre_grupo")]
    pub nombre_grupo: String,

    pub moneda: String,
    pub descripcion: String,

    #[serde(alias = "administrador_id")]
    pub administrador_id: String,
}


// Invitación dentro de GET /grupos
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InvitacionGrupoResumenDto {
    pub id: String,

    #[serde(alias = "grupo_id")]
    pub grupo_id: String,

    #[serde(alias = "invitador_id")]
    pub invitador_id: String,

    pub estado: String,
}


// Respuesta de GET /grupos
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListadoGruposDto {
    pub grupos: Vec<GrupoResumenDto>,

    #[serde(alias = "invitaciones_recibidas")]
    pub invitaciones_recibidas: Vec<InvitacionGrupoResumenDto>,
}


// POST /grupos/{id}/miembros
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InvitarMiembroGrupoEntrada {
    pub nombre_usuario_invitado: String,
}


// Respuesta de POST /grupos/{id}/miembros
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InvitacionGrupoDto {
    pub id: String,

    #[serde(alias = "grupo_id")]
    pub grupo_id: String,

    #[serde(alias = "invitado_id")]
    pub invitado_id: String,

    #[serde(alias = "invitador_id")]
    pub invitador_id: String,

    pub estado: String,
}


// POST /grupos/{id}/miembros/yo/respuesta
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResponderInvitacionGrupoEntrada {
    pub aceptar: bool,
}


// Respuesta de POST /grupos/{id}/miembros/yo/respuesta
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RespuestaInvitacionGrupoDto {
    pub id: String,

    #[serde(alias = "grupo_id")]
    pub grupo_id: String,

    pub estado: String,
}



#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{from_value, json, to_value};

    #[test]
    fn serializar_creacion_grupo() {
        let entrada = CrearGrupoEntrada {
            nombre_grupo: "Viaje".to_string(),
            moneda: "USD".to_string(),
            descripcion: "Vacaciones".to_string(),
        };

        let json = to_value(entrada).unwrap();

        assert_eq!(json["nombreGrupo"], "Viaje");
        assert_eq!(json["moneda"], "USD");
        assert_eq!(json["descripcion"], "Vacaciones");
    }

    #[test]
    fn deserializar_grupo_creado() {
        let json = json!({
            "id": "grupo-1",
            "nombre_grupo": "Viaje",
            "moneda": "USD",
            "administrador_id": "usuario-1",
            "descripcion": "Vacaciones"
        });

        let grupo: GrupoCreadoDto = from_value(json).unwrap();

        assert_eq!(grupo.id, "grupo-1");
        assert_eq!(grupo.nombre_grupo, "Viaje");
        assert_eq!(grupo.administrador_id, "usuario-1");
    }

    #[test]
    fn deserializar_listado_grupos() {
        let json = json!({
            "grupos": [
                {
                    "id": "grupo-1",
                    "nombre_grupo": "Viaje",
                    "moneda": "USD",
                    "descripcion": "Vacaciones",
                    "administrador_id": "usuario-1"
                }
            ],
            "invitaciones_recibidas": [
                {
                    "id": "invitacion-1",
                    "grupo_id": "grupo-2",
                    "invitador_id": "usuario-2",
                    "estado": "PENDIENTE"
                }
            ]
        });

        let listado: ListadoGruposDto = from_value(json).unwrap();

        assert_eq!(listado.grupos.len(), 1);
        assert_eq!(listado.grupos[0].nombre_grupo, "Viaje");

        assert_eq!(listado.invitaciones_recibidas.len(), 1);
        assert_eq!(
            listado.invitaciones_recibidas[0].estado,
            "PENDIENTE"
        );
    }

    #[test]
    fn deserializar_listado_vacio() {
        let json = json!({
            "grupos": [],
            "invitaciones_recibidas": []
        });

        let listado: ListadoGruposDto = from_value(json).unwrap();

        assert!(listado.grupos.is_empty());
        assert!(listado.invitaciones_recibidas.is_empty());
    }

    #[test]
    fn serializar_invitacion_miembro() {
        let entrada = InvitarMiembroGrupoEntrada {
            nombre_usuario_invitado: "beto".to_string(),
        };

        let json = to_value(entrada).unwrap();

        assert_eq!(json["nombreUsuarioInvitado"], "beto");
    }

    #[test]
    fn deserializar_invitacion_creada() {
        let json = json!({
            "id": "invitacion-1",
            "grupo_id": "grupo-1",
            "invitado_id": "usuario-2",
            "invitador_id": "usuario-1",
            "estado": "PENDIENTE"
        });

        let invitacion: InvitacionGrupoDto =
            from_value(json).unwrap();

        assert_eq!(invitacion.id, "invitacion-1");
        assert_eq!(invitacion.grupo_id, "grupo-1");
        assert_eq!(invitacion.invitado_id, "usuario-2");
        assert_eq!(invitacion.estado, "PENDIENTE");
    }

    #[test]
    fn serializar_respuesta_invitacion() {
        let entrada = ResponderInvitacionGrupoEntrada {
            aceptar: true,
        };

        let json = to_value(entrada).unwrap();

        assert_eq!(json["aceptar"], true);
    }

    #[test]
    fn deserializar_respuesta_invitacion() {
        let json = json!({
            "id": "invitacion-1",
            "grupo_id": "grupo-1",
            "estado": "ACEPTADA"
        });

        let respuesta: RespuestaInvitacionGrupoDto =
            from_value(json).unwrap();

        assert_eq!(respuesta.id, "invitacion-1");
        assert_eq!(respuesta.grupo_id, "grupo-1");
        assert_eq!(respuesta.estado, "ACEPTADA");
    }

    #[test]
    fn grupo_no_expone_datos_privados() {
        let grupo = GrupoResumenDto {
            id: "grupo-1".to_string(),
            nombre_grupo: "Viaje".to_string(),
            moneda: "USD".to_string(),
            descripcion: "Vacaciones".to_string(),
            administrador_id: "usuario-1".to_string(),
        };

        let json = to_value(grupo).unwrap();

        assert!(json.get("correo").is_none());
        assert!(json.get("contrasena").is_none());
        assert!(json.get("token").is_none());
    }
}
