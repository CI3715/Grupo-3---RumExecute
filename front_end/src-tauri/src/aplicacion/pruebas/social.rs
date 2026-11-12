
use std::cell::RefCell;

use crate::aplicacion::dto::{
    SesionDto,
    UsuarioRegistradoDto,
};
use crate::aplicacion::dto::usuario_publico::UsuarioPublicoDto;
use crate::aplicacion::dto::amistades::*;
use crate::aplicacion::dto::grupos::*;
use crate::aplicacion::dto::gastos::*;

use crate::aplicacion::puertos::{
    AlmacenSesiones,
    ErrorSesion,
};
use crate::aplicacion::puertos::servicio_social::{
    ErrorServicioSocial,
    ServicioSocial,
};


// Servicio falso para los tests sociales.
pub struct ServicioSocialFalso {
    pub llamadas_buscar:
        RefCell<Vec<(String, String)>>,

    pub respuesta_buscar:
        Result<UsuarioPublicoDto, ErrorServicioSocial>,

    pub llamadas_enviar_invitacion:
        RefCell<Vec<(String, EnviarInvitacionAmistadEntrada)>>,

    pub respuesta_enviar_invitacion:
        Result<InvitacionAmistadDto, ErrorServicioSocial>,

    pub llamadas_responder_amistad:
        RefCell<Vec<(String, String, ResponderInvitacionAmistadEntrada)>>,

    pub respuesta_responder_amistad:
        Result<RespuestaAmistadDto, ErrorServicioSocial>,

    pub llamadas_listar_amistades:
        RefCell<Vec<String>>,

    pub respuesta_listar_amistades:
        Result<ListadoAmistadesDto, ErrorServicioSocial>,

    pub llamadas_eliminar_amistad:
        RefCell<Vec<(String, String)>>,

    pub respuesta_eliminar_amistad:
        Result<(), ErrorServicioSocial>,
    
    pub llamadas_crear_grupo:
        RefCell<Vec<(String, CrearGrupoEntrada)>>,

    pub respuesta_crear_grupo:
        Result<GrupoCreadoDto, ErrorServicioSocial>,
    
    pub llamadas_listar_mis_grupos:
        RefCell<Vec<String>>,

    pub respuesta_listar_mis_grupos:
        Result<ListadoGruposDto, ErrorServicioSocial>,
    
    pub llamadas_invitar_miembro_grupo:
        RefCell<Vec<(String, String, InvitarMiembroGrupoEntrada)>>,

    pub respuesta_invitar_miembro_grupo:
        Result<InvitacionGrupoDto, ErrorServicioSocial>,
    
    pub llamadas_responder_invitacion_grupo:
        RefCell<Vec<(String, String, ResponderInvitacionGrupoEntrada)>>,

    pub respuesta_responder_invitacion_grupo:
        Result<RespuestaInvitacionGrupoDto, ErrorServicioSocial>,
    
    pub llamadas_registrar_gasto:
        RefCell<Vec<(String, String, RegistrarGastoCompartidoEntrada)>>,

    pub respuesta_registrar_gasto:
        Result<GastoCompartidoDto, ErrorServicioSocial>,
}

impl Default for ServicioSocialFalso {
    fn default() -> Self {
        Self {
            llamadas_buscar: RefCell::new(Vec::new()),

            respuesta_buscar: Ok(UsuarioPublicoDto {
                nombre_usuario: "beto".into(),
                nombre_visible: "Beto".into(),
            }),

            llamadas_enviar_invitacion: RefCell::new(Vec::new()),

            respuesta_enviar_invitacion: Ok(InvitacionAmistadDto {
                id: "amistad-1".into(),
                nombre_usuario_solicitante: "ana".into(),
                nombre_usuario_receptor: "beto".into(),
                estado: "PENDIENTE".into(),
            }),

            llamadas_responder_amistad: RefCell::new(Vec::new()),

            respuesta_responder_amistad: Ok(RespuestaAmistadDto {
                id: "amistad-1".into(),
                estado: "ACTIVA".into(),
            }),

            llamadas_listar_amistades: RefCell::new(Vec::new()),

            respuesta_listar_amistades: Ok(ListadoAmistadesDto {
                amigos: vec![AmigoDto {
                    amistad_id: "amistad-1".into(),
                    nombre_usuario: "beto".into(),
                    nombre_visible: "Beto".into(),
                }],
                invitaciones_recibidas: vec![InvitacionAmistadResumenDto {
                    amistad_id: "amistad-2".into(),
                    nombre_usuario: "carla".into(),
                    nombre_visible: "Carla".into(),
                }],
                invitaciones_enviadas: vec![InvitacionAmistadResumenDto {
                    amistad_id: "amistad-3".into(),
                    nombre_usuario: "daniel".into(),
                    nombre_visible: "Daniel".into(),
                }],
            }),

            llamadas_eliminar_amistad: RefCell::new(Vec::new()),

            respuesta_eliminar_amistad: Ok(()),

            llamadas_crear_grupo: RefCell::new(Vec::new()),

            respuesta_crear_grupo: Ok(GrupoCreadoDto {
                id: "grupo-1".into(),
                nombre_grupo: "Viaje".into(),
                moneda: "USD".into(),
                administrador_id: "usuario-1".into(),
                descripcion: "Vacaciones".into(),
            }),

            llamadas_listar_mis_grupos: RefCell::new(Vec::new()),

            respuesta_listar_mis_grupos: Ok(ListadoGruposDto {
                grupos: vec![GrupoResumenDto {
                    id: "grupo-1".into(),
                    nombre_grupo: "Viaje".into(),
                    moneda: "USD".into(),
                    descripcion: "Vacaciones".into(),
                    administrador_id: "usuario-1".into(),
                }],
                invitaciones_recibidas: vec![InvitacionGrupoResumenDto {
                    id: "invitacion-1".into(),
                    grupo_id: "grupo-2".into(),
                    invitador_id: "usuario-2".into(),
                    estado: "PENDIENTE".into(),
                }],
            }),

            llamadas_invitar_miembro_grupo: RefCell::new(Vec::new()),

            respuesta_invitar_miembro_grupo: Ok(InvitacionGrupoDto {
                id: "invitacion-1".into(),
                grupo_id: "grupo-1".into(),
                invitado_id: "usuario-2".into(),
                invitador_id: "usuario-1".into(),
                estado: "PENDIENTE".into(),
            }),

            llamadas_responder_invitacion_grupo: RefCell::new(Vec::new()),

            respuesta_responder_invitacion_grupo: Ok(
                RespuestaInvitacionGrupoDto {
                    id: "invitacion-1".into(),
                    grupo_id: "grupo-1".into(),
                    estado: "ACEPTADA".into(),
                }
            ),

            llamadas_registrar_gasto: RefCell::new(Vec::new()),

            respuesta_registrar_gasto: Ok(GastoCompartidoDto {
                id: "gasto-1".into(),
                grupo_id: "grupo-1".into(),
                concepto: "Cena".into(),
                pagador_id: "usuario-1".into(),
                monto: "30.00".into(),
                fecha: "2026-11-12T12:00:00Z".into(),
                cuotas: vec![
                    CuotaCompartidaDto {
                        usuario_id: "usuario-1".into(),
                        nombre_usuario: "ana".into(),
                        monto: "15.00".into(),
                        lleva_residuo: false,
                    },
                    CuotaCompartidaDto {
                        usuario_id: "usuario-2".into(),
                        nombre_usuario: "beto".into(),
                        monto: "15.00".into(),
                        lleva_residuo: false,
                    },
                ],
            }),
        }
    }
}

impl ServicioSocial for ServicioSocialFalso {
    fn buscar_usuario(
        &self,
        token: &str,
        nombre_usuario: &str,
    ) -> Result<UsuarioPublicoDto, ErrorServicioSocial> {
        self.llamadas_buscar.borrow_mut().push((
            token.to_string(),
            nombre_usuario.to_string(),
        ));

        self.respuesta_buscar.clone()
    }

    fn enviar_invitacion_amistad(
        &self,
        token: &str,
        entrada: &EnviarInvitacionAmistadEntrada,
    ) -> Result<InvitacionAmistadDto, ErrorServicioSocial> {

        self.llamadas_enviar_invitacion.borrow_mut().push((
            token.to_string(),
            entrada.clone(),
        ));

        self.respuesta_enviar_invitacion.clone()
    }

    fn responder_invitacion_amistad(
        &self,
        token: &str,
        amistad_id: &str,
        entrada: &ResponderInvitacionAmistadEntrada,
    ) -> Result<RespuestaAmistadDto, ErrorServicioSocial> {
        self.llamadas_responder_amistad.borrow_mut().push((
            token.to_string(),
            amistad_id.to_string(),
            entrada.clone(),
        ));

        self.respuesta_responder_amistad.clone()
    }

    fn listar_amistades(
        &self,
        token: &str,
    ) -> Result<ListadoAmistadesDto, ErrorServicioSocial> {
        self.llamadas_listar_amistades
            .borrow_mut()
            .push(token.to_string());

        self.respuesta_listar_amistades.clone()
    }

    fn eliminar_amistad(
        &self,
        token: &str,
        amistad_id: &str,
    ) -> Result<(), ErrorServicioSocial> {
        self.llamadas_eliminar_amistad.borrow_mut().push((
            token.to_string(),
            amistad_id.to_string(),
        ));

        self.respuesta_eliminar_amistad.clone()
    }

    fn crear_grupo(
        &self,
        token: &str,
        entrada: &CrearGrupoEntrada,
    ) -> Result<GrupoCreadoDto, ErrorServicioSocial> {
        self.llamadas_crear_grupo.borrow_mut().push((
            token.to_string(),
            entrada.clone(),
        ));

        self.respuesta_crear_grupo.clone()
    }

    fn listar_mis_grupos(
        &self,
        token: &str,
    ) -> Result<ListadoGruposDto, ErrorServicioSocial> {
        self.llamadas_listar_mis_grupos
            .borrow_mut()
            .push(token.to_string());

        self.respuesta_listar_mis_grupos.clone()
    }

    fn invitar_miembro_grupo(
        &self,
        token: &str,
        grupo_id: &str,
        entrada: &InvitarMiembroGrupoEntrada,
    ) -> Result<InvitacionGrupoDto, ErrorServicioSocial> {
        self.llamadas_invitar_miembro_grupo.borrow_mut().push((
            token.to_string(),
            grupo_id.to_string(),
            entrada.clone(),
        ));

        self.respuesta_invitar_miembro_grupo.clone()
    }

    fn responder_invitacion_grupo(
        &self,
        token: &str,
        grupo_id: &str,
        entrada: &ResponderInvitacionGrupoEntrada,
    ) -> Result<RespuestaInvitacionGrupoDto, ErrorServicioSocial> {
        self.llamadas_responder_invitacion_grupo
            .borrow_mut()
            .push((
                token.to_string(),
                grupo_id.to_string(),
                entrada.clone(),
            ));

        self.respuesta_responder_invitacion_grupo.clone()
    }

    fn registrar_gasto_compartido(
        &self,
        token: &str,
        grupo_id: &str,
        entrada: &RegistrarGastoCompartidoEntrada,
    ) -> Result<GastoCompartidoDto, ErrorServicioSocial> {
        self.llamadas_registrar_gasto.borrow_mut().push((
            token.to_string(),
            grupo_id.to_string(),
            entrada.clone(),
        ));

        self.respuesta_registrar_gasto.clone()
    }

    fn obtener_detalle_grupo(
        &self, _: &str, _: &str,
    ) -> Result<DetalleGrupoDto, ErrorServicioSocial> {
        unimplemented!()
    }
}


// Almacén falso para simular sesiones.
pub struct AlmacenFalso {
    pub sesion: Option<SesionDto>,
}

impl AlmacenSesiones for AlmacenFalso {
    fn guardar_sesion(
        &self,
        _: &SesionDto,
    ) -> Result<(), ErrorSesion> {
        Ok(())
    }

    fn obtener_sesion(
        &self,
    ) -> Result<Option<SesionDto>, ErrorSesion> {
        Ok(self.sesion.clone())
    }

    fn eliminar_sesion(
        &self,
    ) -> Result<(), ErrorSesion> {
        Ok(())
    }
}


// Sesión predeterminada para los tests.
pub fn sesion_prueba() -> SesionDto {
    SesionDto {
        usuario: UsuarioRegistradoDto {
            id: "usuario-1".into(),
            nombre_usuario: "ana".into(),
            nombre_visible: "Ana".into(),
            correo: "ana@example.com".into(),
            fecha_alta: "2026-11-12T12:00:00Z".into(),
        },
        token: "token-prueba".into(),
        fecha_expiracion: "2026-12-12T12:00:00Z".into(),
    }
}
