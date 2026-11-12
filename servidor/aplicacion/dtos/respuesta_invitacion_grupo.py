from dataclasses import dataclass


@dataclass(frozen=True)
class RespuestaInvitacionGrupo:
    id: str
    grupo_id: str
    estado: str