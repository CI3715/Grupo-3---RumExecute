from dataclasses import dataclass
from datetime import datetime, timezone
from decimal import Decimal

from dominio.division import CuotaDivision
from dominio.errores import ErrorDominio

#Clase de Gasto, con dataclass frozen para que sea inmutable
@dataclass(frozen=True)
class Gasto:
    id: str
    grupo_id: str
    concepto: str
    pagador_id: str
    monto: Decimal
    cuotas: tuple[CuotaDivision, ...]
    fecha: datetime

    def __post_init__(self):
        # Validar identificadores.
        for campo in ("id", "grupo_id", "pagador_id"):
            valor = getattr(self, campo)

            if not isinstance(valor, str) or not valor.strip():
                raise ErrorDominio(campo, f"El campo {campo} es obligatorio.")

        # Validar concepto.
        if not isinstance(self.concepto, str) or not self.concepto.strip():
            raise ErrorDominio("concepto", "El concepto es obligatorio.")

        object.__setattr__(self, "concepto", self.concepto.strip())

        # Validar fecha.
        if not isinstance(self.fecha, datetime) or self.fecha.tzinfo is None:
            raise ErrorDominio("fecha", "La fecha debe incluir zona horaria.")

        # Normalizar fecha a UTC.
        if self.fecha.utcoffset() is None:
            raise ErrorDominio("fecha", "La fecha debe tener una zona horaria válida.")

        object.__setattr__(self, "fecha", self.fecha.astimezone(timezone.utc))

        # Validar monto.
        if not isinstance(self.monto, Decimal) or not self.monto.is_finite():
            raise ErrorDominio("monto", "El monto debe ser un Decimal finito.")

        if self.monto <= Decimal("0"):
            raise ErrorDominio("monto", "El monto debe ser mayor que cero.")

        if self.monto != self.monto.quantize(Decimal("0.01")):
            raise ErrorDominio("monto", "El monto admite como máximo dos decimales.")

        # Validar cuotas.
        if not isinstance(self.cuotas, (list, tuple)) or not self.cuotas:
            raise ErrorDominio("cuotas", "El gasto debe tener al menos una cuota.")

        # Si las cuotas entran en modo de listas, las convierte a tuplas (Lo tuve que agregar
        # porque las divisiones se retornan en listas aparentemente jaja q gracioso david)
        object.__setattr__(self, "cuotas", tuple(self.cuotas))

        if not all(isinstance(c, CuotaDivision) for c in self.cuotas):
            raise ErrorDominio("cuotas", "Todas las cuotas deben ser instancias de CuotaDivision.")

        # No puede haber dos cuotas para el mismo usuario.
        participantes = [c.usuario_id for c in self.cuotas]

        if len(participantes) != len(set(participantes)):
            raise ErrorDominio("cuotas", "No puede haber cuotas repetidas para un mismo usuario.")

        # La suma de las cuotas debe coincidir exactamente con el gasto.
        total_cuotas = sum(
            (c.monto for c in self.cuotas),
            Decimal("0")
        )

        if total_cuotas != self.monto:
            raise ErrorDominio("cuotas", "Las cuotas no suman el monto del gasto.")

# Funcion para validar si la suma de los balances de cada miembro da cero
def validar_balances_suma_cero(balances: dict[str, Decimal]) -> None:
    if not balances:
        raise ErrorDominio ("balances", "Debe haber al menos un balance.")
    suma = sum(balances.values(), Decimal("0"))

    if suma != Decimal("0"):
        raise ErrorDominio("balances", "Los balances de los miembros del grupo no suman 0")

"""
Funcion que calcula los balances en base a los gastos pagados y las cuotas pendientes.
Si una persona tiene un gasto pagado mayor a sus cuotas pendientes, su balance es positivo.
Si una persona tiene un gasto pagado menor a sus cuotas pendientes, su balance es negativo.
Si una persona tiene un gasto pagado igual a sus cuotas pendientes, su balance es cero.
"""
def calcular_balances(
    miembros: tuple[str, ...], gastos: list[Gasto] # miembros es una tupla con tamaño arbitrario porque asi está en GrupoCompartido
) -> dict[str, Decimal]:
    balances = {m: Decimal("0") for m in miembros}
    for gasto in gastos:
        if gasto.pagador_id not in balances:
            raise ErrorDominio("pagador", "El pagador debe ser miembro del grupo.")
        balances[gasto.pagador_id] += gasto.monto # Se le suma lo que los gastos que ha pagado
        for cuota in gasto.cuotas:
            if cuota.usuario_id not in balances:
                raise ErrorDominio(
                    "participantes", "Todos los participantes deben ser miembros del grupo."
                )
            balances[cuota.usuario_id] -= cuota.monto # Se le suma las cuotas que tiene pendientes
    # Al terminar el ciclo, cuando ya todos los balances individuales están calculados, se verifica que su suma de cero
    validar_balances_suma_cero(balances)
    return balances
