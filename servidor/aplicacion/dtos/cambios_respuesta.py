from dataclasses import dataclass
from datetime import datetime
from decimal import Decimal

@dataclass(frozen=True)
class EventoCambioRespuesta:
    id: str
    posicion: int
    usuario_id: str
    grupo_id: str
    nombre_grupo: str
    gasto_id: str
    concepto: str
    fecha: datetime # con tz, en UTC-0
    moneda: str
    monto_parte: Decimal
    incluye_redondeo: bool

@dataclass(frozen=True)
class CambiosRespuesta:
    eventos: tuple[EventoCambioRespuesta, ...]
    nuevo_cursor: int
    quedan_mas: bool