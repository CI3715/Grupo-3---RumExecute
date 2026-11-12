from dataclasses import dataclass


@dataclass(frozen=True)
class AmigoRespuesta:
    amistad_id: str
    nombre_usuario: str
    nombre_visible: str


@dataclass(frozen=True)
class InvitacionAmistadResumen:
    amistad_id: str
    nombre_usuario: str
    nombre_visible: str


@dataclass(frozen=True)
class ListadoAmistades:
    amigos: list[AmigoRespuesta]
    invitaciones_recibidas: list[InvitacionAmistadResumen]
    invitaciones_enviadas: list[InvitacionAmistadResumen]
