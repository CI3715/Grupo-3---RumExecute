from dataclasses import dataclass


@dataclass(frozen=True)
class InvitacionAmistadRespuesta:
    id: str
    nombre_usuario_solicitante: str
    nombre_usuario_receptor: str
    estado: str