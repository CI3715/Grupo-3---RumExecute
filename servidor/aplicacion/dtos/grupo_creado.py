from dataclasses import dataclass


@dataclass(frozen=True)
class GrupoCreado:
    id: str
    nombre_grupo: str
    moneda: str
    administrador_id: str
    descripcion: str