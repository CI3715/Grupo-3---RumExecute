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