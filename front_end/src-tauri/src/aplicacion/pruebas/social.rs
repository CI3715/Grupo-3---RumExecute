
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
    pub llamadas_buscar: RefCell<Vec<(String, String)>>,
    pub respuesta_buscar: Result<UsuarioPublicoDto, ErrorServicioSocial>,
}

impl Default for ServicioSocialFalso {
    fn default() -> Self {
        Self {
            llamadas_buscar: RefCell::new(Vec::new()),
            respuesta_buscar: Ok(UsuarioPublicoDto {
                nombre_usuario: "beto".into(),
                nombre_visible: "Beto".into(),
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
        &self, _: &str, _: &EnviarInvitacionAmistadEntrada,
    ) -> Result<InvitacionAmistadDto, ErrorServicioSocial> {
        unimplemented!()
    }

    fn responder_invitacion_amistad(
        &self, _: &str, _: &str,
        _: &ResponderInvitacionAmistadEntrada,
    ) -> Result<RespuestaAmistadDto, ErrorServicioSocial> {
        unimplemented!()
    }

    fn listar_amistades(
        &self, _: &str,
    ) -> Result<ListadoAmistadesDto, ErrorServicioSocial> {
        unimplemented!()
    }

    fn eliminar_amistad(
        &self, _: &str, _: &str,
    ) -> Result<(), ErrorServicioSocial> {
        unimplemented!()
    }

    fn crear_grupo(
        &self, _: &str, _: &CrearGrupoEntrada,
    ) -> Result<GrupoCreadoDto, ErrorServicioSocial> {
        unimplemented!()
    }

    fn listar_mis_grupos(
        &self, _: &str,
    ) -> Result<ListadoGruposDto, ErrorServicioSocial> {
        unimplemented!()
    }

    fn invitar_miembro_grupo(
        &self, _: &str, _: &str,
        _: &InvitarMiembroGrupoEntrada,
    ) -> Result<InvitacionGrupoDto, ErrorServicioSocial> {
        unimplemented!()
    }

    fn responder_invitacion_grupo(
        &self, _: &str, _: &str,
        _: &ResponderInvitacionGrupoEntrada,
    ) -> Result<RespuestaInvitacionGrupoDto, ErrorServicioSocial> {
        unimplemented!()
    }

    fn registrar_gasto_compartido(
        &self, _: &str, _: &str,
        _: &RegistrarGastoCompartidoEntrada,
    ) -> Result<GastoCompartidoDto, ErrorServicioSocial> {
        unimplemented!()
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
