from dataclasses import dataclass
from enum import Enum, auto

from dominio.amistad import Amistad, EstadoAmistad
from dominio.errores import ErrorConflicto, ErrorPermiso
from dominio.grupo import GrupoCompartido, validar_es_admin


class EstadoInvitacionGrupo(Enum):
    PENDIENTE = auto()
    ACEPTADA = auto()
    RECHAZADA = auto()


@dataclass(frozen=True)
class InvitacionGrupo:
    id: str
    grupo_id: str
    invitado_id: str
    invitador_id: str
    estado: EstadoInvitacionGrupo = EstadoInvitacionGrupo.PENDIENTE

def _son_amigos(a: str, b: str, amistades: list[Amistad]) -> bool:
    return any(
        {x.id_solicitante, x.id_receptor} == {a, b}
        and x.estado == EstadoAmistad.ACTIVA
        for x in amistades
    )


def validar_invitacion_grupo(
    grupo: GrupoCompartido,
    invitador_id: str,
    invitado_id: str,
    amistades: list[Amistad],
    invitaciones: list[InvitacionGrupo],
) -> None:
    validar_es_admin(grupo, invitador_id)
    if not _son_amigos(invitador_id, invitado_id, amistades):
        raise ErrorPermiso("invitado", "Solo puedes invitar a tus amigos.")
    if invitado_id in grupo.id_miembros:
        raise ErrorConflicto("invitado", "El usuario ya es miembro del grupo.")
    for inv in invitaciones:
        if (
            inv.grupo_id == grupo.id
            and inv.invitado_id == invitado_id
            and inv.estado == EstadoInvitacionGrupo.PENDIENTE
        ):
            raise ErrorConflicto(
                "invitado", "Ya existe una invitación pendiente para este usuario."
            )

def responder_invitacion_grupo(
    invitacion: InvitacionGrupo, quien_responde_id: str, aceptar: bool
) -> InvitacionGrupo:
    if quien_responde_id != invitacion.invitado_id:
        raise ErrorPermiso("usuario", "Solo el invitado puede responder la invitación.")
    if invitacion.estado != EstadoInvitacionGrupo.PENDIENTE:
        raise ErrorConflicto("invitacion", "La invitación ya fue respondida.")
    nuevo_estado = (
        EstadoInvitacionGrupo.ACEPTADA if aceptar else EstadoInvitacionGrupo.RECHAZADA
    )
    return InvitacionGrupo(
        id=invitacion.id,
        grupo_id=invitacion.grupo_id,
        invitado_id=invitacion.invitado_id,
        invitador_id=invitacion.invitador_id,
        estado=nuevo_estado,
    )