from dataclasses import dataclass
from decimal import Decimal

from aplicacion.dtos.gasto_compartido_respuesta import (
    GastoCompartidoRespuesta,
)


@dataclass(frozen=True)
class MiembroGrupoRespuesta:
    usuario_id: str
    nombre_usuario: str
    nombre_visible: str
    es_administrador: bool
    balance: Decimal


@dataclass(frozen=True)
class DetalleGrupo:
    id: str
    nombre_grupo: str
    moneda: str
    descripcion: str
    administrador_id: str
    miembros: tuple[MiembroGrupoRespuesta, ...]
    gastos: tuple[GastoCompartidoRespuesta, ...]
