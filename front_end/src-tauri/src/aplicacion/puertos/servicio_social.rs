use crate::aplicacion::dto::usuario_publico::UsuarioPublicoDto;

use crate::aplicacion::dto::amistades::{
    EnviarInvitacionAmistadEntrada,
    InvitacionAmistadDto,
    ResponderInvitacionAmistadEntrada,
    RespuestaAmistadDto,
    ListadoAmistadesDto,
};

use crate::aplicacion::dto::grupos::{
    CrearGrupoEntrada,
    GrupoCreadoDto,
    ListadoGruposDto,
    InvitarMiembroGrupoEntrada,
    InvitacionGrupoDto,
    ResponderInvitacionGrupoEntrada,
    RespuestaInvitacionGrupoDto,
};

use crate::aplicacion::dto::gastos::{
    RegistrarGastoCompartidoEntrada,
    GastoCompartidoDto,
    DetalleGrupoDto,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ErrorServicioSocial {
    NoAutenticado,
    SesionExpirada,
    ServicioNoDisponible,
    NoEncontrado,
    SinPermiso,
    Conflicto {
        campo: String,
        mensaje: String,
    },
    Validacion {
        campo: String,
        mensaje: String,
    },
    ErrorTecnico(String),
}

pub trait ServicioSocial {

    // GET /usuarios?usuario=<exacto>
    fn buscar_usuario(
        &self,
        token: &str,
        nombre_usuario: &str,
    ) -> Result<UsuarioPublicoDto, ErrorServicioSocial>;

    // POST /amistades
    fn enviar_invitacion_amistad(
        &self,
        token: &str,
        entrada: &EnviarInvitacionAmistadEntrada,
    ) -> Result<InvitacionAmistadDto, ErrorServicioSocial>;

    // POST /amistades/{id}/respuesta
    fn responder_invitacion_amistad(
        &self,
        token: &str,
        amistad_id: &str,
        entrada: &ResponderInvitacionAmistadEntrada,
    ) -> Result<RespuestaAmistadDto, ErrorServicioSocial>;

    // GET /amistades
    fn listar_amistades(
        &self,
        token: &str,
    ) -> Result<ListadoAmistadesDto, ErrorServicioSocial>;

    // DELETE /amistades/{id}
    fn eliminar_amistad(
        &self,
        token: &str,
        amistad_id: &str,
    ) -> Result<(), ErrorServicioSocial>;

    // POST /grupos
    fn crear_grupo(
        &self,
        token: &str,
        entrada: &CrearGrupoEntrada,
    ) -> Result<GrupoCreadoDto, ErrorServicioSocial>;

    // GET /grupos
    fn listar_mis_grupos(
        &self,
        token: &str,
    ) -> Result<ListadoGruposDto, ErrorServicioSocial>;

    // POST /grupos/{id}/miembros
    fn invitar_miembro_grupo(
        &self,
        token: &str,
        grupo_id: &str,
        entrada: &InvitarMiembroGrupoEntrada,
    ) -> Result<InvitacionGrupoDto, ErrorServicioSocial>;

    // POST /grupos/{id}/miembros/yo/respuesta
    fn responder_invitacion_grupo(
        &self,
        token: &str,
        grupo_id: &str,
        entrada: &ResponderInvitacionGrupoEntrada,
    ) -> Result<RespuestaInvitacionGrupoDto, ErrorServicioSocial>;

    // POST /grupos/{id}/gastos
    fn registrar_gasto_compartido(
        &self,
        token: &str,
        grupo_id: &str,
        entrada: &RegistrarGastoCompartidoEntrada,
    ) -> Result<GastoCompartidoDto, ErrorServicioSocial>;

    // GET /grupos/{id}
    fn obtener_detalle_grupo(
        &self,
        token: &str,
        grupo_id: &str,
    ) -> Result<DetalleGrupoDto, ErrorServicioSocial>;
}
