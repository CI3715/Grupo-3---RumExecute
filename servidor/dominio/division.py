from dataclasses import dataclass
from decimal import Decimal, ROUND_DOWN
from dominio.errores import ErrorDominio


@dataclass(frozen=True)
class CuotaDivision: 
    """
    Clase que tiene las cuotas de cada usuario 
    en una cuenta compartida
    """
    nombre_usuario: str
    monto: Decimal
    lleva_residuo: bool

def repartir_residuo(
        total: Decimal, cuotas_exactas: dict[str, Decimal]
) -> list[CuotaDivision]:
    # Si no hay participantes, error
    if not cuotas_exactas:
        raise ErrorDominio("participantes", "Debe de haber al menos un participante")
    # Si el monto total es 0 no tiene sentido hacer una cuenta compartida
    if total <= Decimal("0"):
        raise ErrorDominio("monto", "El monto debe ser mayor que cero.")
    # Si el monto total tiene más de 2 decimales, es inválido
    if total != total.quantize(Decimal("0.01")):
        raise ErrorDominio("monto", "El monto debe tener como máximo 2 decimales.")
    centimo = Decimal("0.01")
    truncadas = {
        nombre: cuota.quantize(centimo, rounding=ROUND_DOWN) #Se redondea hacia abajo en los centimos
        for nombre, cuota in cuotas_exactas.items()
    }
    residuo = total - sum(truncadas.values(), Decimal("0"))
    centimos_restantes = int(residuo / centimo) # Cantidad de centimos restantes
    """
    Si el residuo es negativo, hay algo mal con las cuotas exactas. Mismo caso
    para cuando hay mas centimos_restantes que gente entre la cual repartir. No tiene sentido
    por multiplicidad
    """
    if residuo < Decimal("0") or centimos_restantes >= len(truncadas):
        raise ErrorDominio(
            "cuotas",
            "Las cuotas no cuadran con el monto total"
        )
    cuotas = []
    for indice, nombre in enumerate(sorted(truncadas)):
        lleva_residuo = indice < centimos_restantes #True si aun quedan, False si ya se acabaron
        monto = truncadas[nombre] + (centimo if lleva_residuo else Decimal("0"))
        cuotas.append(CuotaDivision(nombre, monto, lleva_residuo))
    return cuotas    