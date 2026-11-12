from dataclasses import dataclass


@dataclass(frozen=True)
class GrupoResumen:
    id: str
    nombre_grupo: str
    moneda: str
    descripcion: str
    administrador_id: str


@dataclass(frozen=True)
class InvitacionGrupoResumen:
    id: str
    grupo_id: str
    invitador_id: str
    estado: str


@dataclass(frozen=True)
class ListadoGrupos:
    grupos: list[GrupoResumen]
    invitaciones_recibidas: list[InvitacionGrupoResumen]