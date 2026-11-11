from dataclasses import dataclass
from datetime import datetime
from decimal import Decimal

from dominio.division import CuotaDivision
from dominio.errores import ErrorDominio

@dataclass(frozen=True)
class Gasto:
    concepto: str
    pagador: str
    monto: Decimal
    cuotas: tuple[CuotaDivision, ...]
    fecha: datetime

    def __post_init__(self):
        object.__setattr__(self, "concepto", self.concepto.strip())
        if not self.concepto:
            raise ErrorDominio("concepto", "El concepto es obligatorio.")
        if self.fecha.tzinfo is None:
            raise ErrorDominio("fecha", "La fecha debe incluir zona horaria (UTC).")
        if self.monto <= Decimal("0"):
            raise ErrorDominio("monto", "El monto debe ser mayor que cero.")
        if self.monto != self.monto.quantize(Decimal("0.01")):
            raise ErrorDominio("monto", "El monto admite como máximo dos decimales.")
        if not self.cuotas:
            raise ErrorDominio("cuotas", "El gasto debe tener al menos una cuota.")
        if sum((c.monto for c in self.cuotas), Decimal("0")) != self.monto:
            raise ErrorDominio("cuotas", "Las cuotas no suman el monto del gasto.")


def validar_balances_suma_cero(
        balances: dict[str, Decimal]
) -> None:
    if not balances:
        raise ErrorDominio ("balances", "Debe haber al menos un balance.")
    suma = sum(balances.values(), Decimal("0"))

    if suma != Decimal("0"):
        raise ErrorDominio("balances", "Los balances de los miembros del grupo no suman 0")

def calcular_balances(
    miembros: tuple[str, ...], gastos: list[Gasto]
) -> dict[str, Decimal]:
    balances = {m: Decimal("0") for m in miembros}
    for gasto in gastos:
        if gasto.pagador not in balances:
            raise ErrorDominio("pagador", "El pagador debe ser miembro del grupo.")
        balances[gasto.pagador] += gasto.monto
        for cuota in gasto.cuotas:
            if cuota.nombre_usuario not in balances:
                raise ErrorDominio(
                    "participantes", "Todos los participantes deben ser miembros del grupo."
                )
            balances[cuota.nombre_usuario] -= cuota.monto
    validar_balances_suma_cero(balances)
    return balances