from dataclasses import dataclass
from datetime import datetime
from decimal import Decimal


@dataclass(frozen=True)
class CuotaRespuesta:
    usuario_id: str
    nombre_usuario: str
    monto: Decimal
    lleva_residuo: bool


@dataclass(frozen=True)
class GastoCompartidoRespuesta:
    id: str
    grupo_id: str
    concepto: str
    pagador_id: str
    monto: Decimal
    fecha: datetime
    cuotas: tuple[CuotaRespuesta, ...]
    creado: bool = True
    es_reintento: bool = False

    def __post_init__(self):
        if not self.creado and not self.es_reintento:
            object.__setattr__(self, "es_reintento", True)
        elif self.es_reintento and self.creado:
            object.__setattr__(self, "creado", False)
