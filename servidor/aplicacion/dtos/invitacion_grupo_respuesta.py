from dataclasses import dataclass


@dataclass(frozen=True)
class InvitacionGrupoRespuesta:
    id: str
    grupo_id: str
    invitado_id: str
    invitador_id: str
    estado: str