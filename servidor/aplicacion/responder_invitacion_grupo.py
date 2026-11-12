from aplicacion.dtos.respuesta_invitacion_grupo import (
    RespuestaInvitacionGrupo,
)
from aplicacion.errores import GrupoNoEncontrado
from aplicacion.puertos.repositorio_grupos import RepositorioGrupos

from dominio.errores import ErrorConflicto
from dominio.grupo import es_miembro
from dominio.invitacion_grupo import (
    EstadoInvitacionGrupo,
    responder_invitacion_grupo,
)


class ResponderInvitacionGrupo:

    def __init__(
        self,
        repositorio_grupos: RepositorioGrupos,
    ):
        self.repositorio_grupos = repositorio_grupos

    def ejecutar(
        self,
        usuario_autenticado_id: str,
        grupo_id: str,
        aceptar: bool,
    ) -> RespuestaInvitacionGrupo:

        # Buscar las invitaciones del grupo.
        invitaciones = (
            self.repositorio_grupos.listar_invitaciones_de_grupo(
                grupo_id
            )
        )

        # Considerar únicamente las del usuario autenticado.
        propias = [
            inv for inv in invitaciones
            if (
                inv.grupo_id == grupo_id
                and inv.invitado_id == usuario_autenticado_id
            )
        ]

        # No revelar información de grupos ajenos.
        if not propias:
            raise GrupoNoEncontrado()

        # Preferir una invitación pendiente si hubo
        # invitaciones anteriores rechazadas.
        invitacion = next(
            (
                inv for inv in propias
                if inv.estado == EstadoInvitacionGrupo.PENDIENTE
            ),
            None,
        )

        if invitacion is None:
            raise ErrorConflicto(
                "invitacion",
                "La invitación ya fue respondida.",
            )

        grupo = self.repositorio_grupos.obtener_por_id(grupo_id)

        if grupo is None:
            raise GrupoNoEncontrado()

        if es_miembro(grupo, usuario_autenticado_id):
            raise ErrorConflicto(
                "usuario",
                "El usuario ya pertenece al grupo.",
            )

        # El dominio comprueba quién responde y
        # que el estado sea PENDIENTE.
        actualizada = responder_invitacion_grupo(
            invitacion,
            usuario_autenticado_id,
            aceptar,
        )

        # Infraestructura actualiza la invitación y,
        # si fue aceptada, agrega al miembro atómicamente.
        self.repositorio_grupos.actualizar_respuesta_invitacion(
            actualizada
        )

        return RespuestaInvitacionGrupo(
            id=actualizada.id,
            grupo_id=actualizada.grupo_id,
            estado=actualizada.estado.name,
        )
