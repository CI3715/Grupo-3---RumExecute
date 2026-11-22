from dataclasses import dataclass
from datetime import datetime, timedelta
from decimal import Decimal

from .errores import ErrorDominio
from dominio.balance import Gasto
from dominio.division import CuotaDivision

@dataclass(frozen=True)
class EventoCambioUsuario:
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

    def __post_init__(self):
        if not self.id.strip():
            raise ErrorDominio("id", "El ID del evento es obligatorio")

        if self.posicion < 1:
            raise ErrorDominio("posicion", "La posición debe ser positiva")

        if not self.usuario_id.strip():
            raise ErrorDominio("usuario_id", "El usuario es obligatorio")

        if not self.grupo_id.strip():
            raise ErrorDominio("grupo_id", "El grupo es obligatorio")

        if not self.gasto_id.strip():
            raise ErrorDominio("gasto_id", "El gasto es obligatorio")

        if self.fecha.tzinfo is None:
            raise ErrorDominio("fecha", "La fecha debe incluir zona horaria")

        if self.fecha.utcoffset() != timedelta(0):
            raise ErrorDominio("fecha", "La fecha debe estar expresada en UTC")

        if not isinstance(self.monto_parte, Decimal):
            raise ErrorDominio("monto_parte", "El monto debe ser Decimal")

        if not self.monto_parte.is_finite() or self.monto_parte < 0:
            raise ErrorDominio(
                "monto_parte",
                "El monto de la parte no puede ser negativo ni infinito"
            )

def crear_evento_cambio(
    gasto: Gasto,
    cuota: CuotaDivision,
    nombre_grupo: str,
    moneda: str,
    evento_id: str,
    posicion: int,
) -> EventoCambioUsuario:
    """
    Construye el evento de cambio correspondiente
    a la cuota de un participante de un gasto.

    No genera IDs, posiciones ni recalcula cuotas.
    """

    if not isinstance(gasto, Gasto):
        raise ErrorDominio("gasto", "El gasto debe ser una instancia de Gasto.")

    if not isinstance(cuota, CuotaDivision):
        raise ErrorDominio("cuota", "La cuota debe ser una instancia de CuotaDivision.")

    if cuota not in gasto.cuotas:
        raise ErrorDominio("cuota", "La cuota no pertenece al gasto indicado.")

    return EventoCambioUsuario(
        id=evento_id,
        posicion=posicion,
        usuario_id=cuota.usuario_id,
        grupo_id=gasto.grupo_id,
        nombre_grupo=nombre_grupo,
        gasto_id=gasto.id,
        concepto=gasto.concepto,
        fecha=gasto.fecha,
        moneda=moneda,
        monto_parte=cuota.monto,
        incluye_redondeo=cuota.lleva_residuo,
    )
