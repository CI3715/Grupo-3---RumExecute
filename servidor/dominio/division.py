from dataclasses import dataclass
from decimal import Decimal, ROUND_DOWN
from dominio.errores import ErrorDominio


@dataclass(frozen=True)
class CuotaDivision: 
    """
    Clase que tiene las cuotas de cada usuario 
    en una cuenta compartida
    """
    usuario_id: str
    nombre_usuario: str
    monto: Decimal
    lleva_residuo: bool



def repartir_residuo(
        total: Decimal, cuotas_exactas: dict[str, Decimal], 
        nombres: dict[str, str],  # usuario_id -> nombre_usuario (OBLIGATORIO)
) -> list[CuotaDivision]:
    # Si no hay participantes, error
    if not cuotas_exactas:
        raise ErrorDominio("participantes", "Debe de haber al menos un participante")
    # Evita indeterminismo si dos IDs distintos comparten el mismo nombre de usuario
    if len(set(nombres.values())) != len(nombres.values()):
        raise ErrorDominio("participantes", "Hay nombres de usuario repetidos entre los participantes.")
    # Asegura que cada cuota tenga su nombre correspondiente
    if set(cuotas_exactas.keys()) != set(nombres.keys()):
        raise ErrorDominio("participantes", "Los participantes y los nombres de usuario no coinciden.")
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

    ordenados = sorted(
        truncadas.keys(),
        key=lambda uid: nombres[uid]  # <-- Obligatoriamente el nombre
    )

    cuotas = []
    for indice, uid in enumerate(ordenados):
        lleva_residuo = indice < centimos_restantes #True si aun quedan, False si ya se acabaron
        monto = truncadas[uid] + (centimo if lleva_residuo else Decimal("0"))
        cuotas.append(CuotaDivision(usuario_id=uid, nombre_usuario=nombres[uid], monto=monto, lleva_residuo=lleva_residuo))
    return cuotas


def dividir_partes_iguales(
    total: Decimal, 
    nombres: dict[str, str]  # usuario_id -> nombre_usuario
) -> list[CuotaDivision]:
    if not nombres:
        raise ErrorDominio("participantes", "Debe de haber al menos un participante.")
    """
    Como aquí es en partes iguales, lo cual no siempre da porcentajes exactos,
    no nos interesan los porcentajes en general, solo los montos
    """
    cuota = total / len(nombres)
    # cuotas_exactas tiene {uid: cuota}
    cuotas_exactas = {uid: cuota for uid in nombres.keys()}
    return repartir_residuo(total, cuotas_exactas, nombres)


def dividir_por_porcentajes(
    total: Decimal, 
    porcentajes: dict[str, Decimal], # usuario_id -> porcentaje
    nombres: dict[str, str], # usuario_id -> nombre_usuario
) -> list[CuotaDivision]:
    if not porcentajes:
        raise ErrorDominio("participantes", "Debe de haber al menos un participante con su porcentaje")

    # Chequeo de que el porcentajes introducidos son, en efecto, porcentajes válidos
    for p in porcentajes.values():
        if p < Decimal("0") or p > Decimal("100"):
            raise ErrorDominio("porcentajes", "Los porcentajes deben de estar entre 0 y 100")
        if p != p.quantize(Decimal("0.01")):
            raise ErrorDominio("porcentajes", "Los porcentajes deben tener como máximo 2 decimales")

    suma = sum(porcentajes.values(), Decimal("0"))
    # Chequeo de que la suma de los porcentajes da, en efecto, el total (100%)
    if suma < Decimal("100"):
        raise ErrorDominio(
            "porcentajes",
            f"Los porcentajes suman {suma}, es decir, faltan {Decimal('100') - suma}",
        )
    elif suma > Decimal("100"):
        raise ErrorDominio(
            "porcentajes",
            f"Los porcentajes suman {suma}, es decir, sobran {suma - Decimal('100')}", 
        )
    # A cada nombre se le asigna su monto referente al porcentaje de su cuota
    cuotas = {uid: total * p / Decimal("100") for uid, p in porcentajes.items()}
    return repartir_residuo(total, cuotas, nombres)

def dividir_por_montos(
        total: Decimal, 
        montos: dict[str, Decimal], # usuario_id -> monto
        nombres: dict[str, str], # usuario_id -> nombre_usuario
) -> list[CuotaDivision]:
    if not montos:
        raise ErrorDominio("participantes", "Debe de haber al menos un participante con su monto")
    for m in montos.values():
        if m < Decimal("0") or m > total:
            raise ErrorDominio("monto", f"El monto debe de estar entre 0 y {total}")
        if m != m.quantize(Decimal("0.01")):
            raise ErrorDominio("monto", "El monto debe tener como máximo 2 decimales")

    suma = sum(montos.values(), Decimal("0"))

    if suma < total:
        raise ErrorDominio(
            "montos",
            f"Los montos suman {suma}, es decir, faltan {total - suma}"
        )
    elif suma > total:
        raise ErrorDominio(
            "montos",
            f"Los montos suman {suma}, es decir, sobran {suma - total}"
        )
    return repartir_residuo(total, montos, nombres)