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
    nombre_grupo: str
    invitado: str
    invitador: str
    estado: EstadoInvitacionGrupo = EstadoInvitacionGrupo.PENDIENTE

def _son_amigos(a: str, b: str, amistades: list[Amistad]) -> bool:
    return any(
        {x.nombre_usuario_solicitante, x.nombre_usuario_receptor} == {a, b}
        and x.estado == EstadoAmistad.ACTIVA
        for x in amistades
    )


def validar_invitacion_grupo(
    grupo: GrupoCompartido,
    invitador: str,
    invitado: str,
    amistades: list[Amistad],
    invitaciones: list[InvitacionGrupo],
) -> None:
    validar_es_admin(grupo, invitador)
    if not _son_amigos(invitador, invitado, amistades):
        raise ErrorPermiso("invitado", "Solo puedes invitar a tus amigos.")
    if invitado in grupo.miembros:
        raise ErrorConflicto("invitado", "El usuario ya es miembro del grupo.")
    for inv in invitaciones:
        if (
            inv.nombre_grupo == grupo.nombre_grupo
            and inv.invitado == invitado
            and inv.estado == EstadoInvitacionGrupo.PENDIENTE
        ):
            raise ErrorConflicto(
                "invitado", "Ya existe una invitación pendiente para este usuario."
            )

def responder_invitacion_grupo(
    invitacion: InvitacionGrupo, quien_responde: str, aceptar: bool
) -> InvitacionGrupo:
    if quien_responde != invitacion.invitado:
        raise ErrorPermiso("usuario", "Solo el invitado puede responder la invitación.")
    if invitacion.estado != EstadoInvitacionGrupo.PENDIENTE:
        raise ErrorConflicto("invitacion", "La invitación ya fue respondida.")
    nuevo = (
        EstadoInvitacionGrupo.ACEPTADA if aceptar else EstadoInvitacionGrupo.RECHAZADA
    )
    return InvitacionGrupo(
        invitacion.nombre_grupo, invitacion.invitado, invitacion.invitador, nuevo
    )