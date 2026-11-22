from dataclasses import dataclass;
from datetime import datetime;
from decimal import Decimal

@dataclass(frozen=True)
class EventoCambioUsuario:
    id: str
    posicion: int | None
    usuario_id: str
    grupo_id: str
    nombre_grupo: str
    gasto_id: str
    concepto: str
    fecha: datetime # con tz, en UTC-0
    moneda: str
    monto_parte: Decimal
    incluye_redondeo: bool


