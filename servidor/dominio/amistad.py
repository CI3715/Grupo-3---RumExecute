from dataclasses import dataclass
from enum import Enum, auto

from dominio.errores import ErrorConflicto, ErrorDominio, ErrorPermiso


class EstadoAmistad(Enum):
    PENDIENTE = auto()
    ACTIVA = auto()
    RECHAZADA = auto()


@dataclass(frozen=True)
class Amistad:
    id: str
    id_solicitante: str
    id_receptor: str
    estado: EstadoAmistad = EstadoAmistad.PENDIENTE

# Verifica si hay una solicitud ya sea a->b o b->a

def _es_entre(amistad: Amistad, a: str, b: str) -> bool:
    return {amistad.id_solicitante, amistad.id_receptor} == {a, b}

# Verifica si la solicitud de amistad es válida
def validar_solicitud_amistad(
    id_sol: str,
    id_recept: str,
    amistades_existentes: list[Amistad],
) -> None:
    if id_sol == id_recept:
        raise ErrorDominio(
            "receptor", "No se puede enviar una solicitud de amistad hacia uno mismo."
        )

    for existente in amistades_existentes:
        if not _es_entre(existente, id_sol, id_recept):
            continue
        if existente.estado == EstadoAmistad.PENDIENTE:
            raise ErrorConflicto(
                "receptor", "Ya existe una solicitud de amistad pendiente con este usuario."
            )
        if existente.estado == EstadoAmistad.ACTIVA:
            raise ErrorConflicto(
                "receptor", "Ya hay una amistad activa con este usuario."
            )
        # RECHAZADA: la invitación quedó cerrada, se puede volver a invitar.


def responder_solicitud(
    amistad: Amistad, id_us_responde: str, aceptar: bool
) -> Amistad:
    if id_us_responde != amistad.id_receptor:
        raise ErrorPermiso(
            "usuario", "Solo el destinatario puede responder la invitación."
        )
    if amistad.estado != EstadoAmistad.PENDIENTE:
        raise ErrorConflicto("amistad", "La invitación ya fue respondida.")
    nuevo_estado = EstadoAmistad.ACTIVA if aceptar else EstadoAmistad.RECHAZADA
    return Amistad(
        amistad.id,
        amistad.id_solicitante,
        amistad.id_receptor,
        nuevo_estado,
    )
