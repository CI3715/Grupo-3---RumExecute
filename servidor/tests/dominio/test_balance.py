from datetime import datetime, timezone
from decimal import Decimal as D
import pytest

from dominio.balance import Gasto, calcular_balances, validar_balances_suma_cero
from dominio.division import dividir_partes_iguales, dividir_por_porcentajes
from dominio.errores import ErrorDominio

AHORA = datetime.now(timezone.utc)
MIEMBROS = ("ana", "beto", "carla")

def gasto_iguales(pagador="ana", total="100.00", participantes=MIEMBROS, concepto="Cena"):
    nombres = {p: p for p in participantes}
    cuotas = tuple(dividir_partes_iguales(D(total), nombres))
    return Gasto(concepto, pagador, D(total), cuotas, AHORA)

def gasto_porcentajes():
    pct = {"ana": D("50"), "beto": D("25"), "carla": D("25")}
    nombres = {p: p for p in pct.keys()}
    cuotas = tuple(dividir_por_porcentajes(D("60.00"), pct, nombres))
    return Gasto("Taxi", "beto", D("60.00"), cuotas, AHORA)

def test_ejemplo_oficial_de_balances():
    b = calcular_balances(MIEMBROS, [gasto_iguales(), gasto_porcentajes()])
    assert b == {"ana": D("36.66"), "beto": D("11.67"), "carla": D("-48.33")}
    assert sum(b.values(), D("0")) == D("0")

def test_miembros_sin_gastos_quedan_en_cero():
    assert calcular_balances(MIEMBROS, []) == {m: D("0") for m in MIEMBROS}

def test_gasto_entre_un_subconjunto():
    b = calcular_balances(MIEMBROS, [gasto_iguales("ana", "10.00", ("ana", "beto"))])
    assert b == {"ana": D("5.00"), "beto": D("-5.00"), "carla": D("0")}

def test_pagador_que_no_es_miembro():
    with pytest.raises(ErrorDominio) as exc:
        calcular_balances(("beto", "carla"), [gasto_iguales("ana", "10.00", ("beto", "carla"))])
    assert exc.value.campo == "pagador"

def test_participante_que_no_es_miembro():
    with pytest.raises(ErrorDominio) as exc:
        calcular_balances(("ana", "beto"), [gasto_iguales("ana", "9.00", MIEMBROS)])
    assert exc.value.campo == "participantes"

def test_validar_balances_que_no_suman_cero():
    with pytest.raises(ErrorDominio) as exc:
        validar_balances_suma_cero({"ana": D("1.00"), "beto": D("-0.99")})
    assert exc.value.campo == "balances"

def test_validar_balances_vacios():
    with pytest.raises(ErrorDominio):
        validar_balances_suma_cero({})

def test_gasto_concepto_vacio():
    with pytest.raises(ErrorDominio) as exc:
        gasto_iguales(concepto="   ")
    assert exc.value.campo == "concepto"

def test_gasto_cuotas_que_no_suman_el_monto():
    nombres = {m: m for m in MIEMBROS}
    cuotas = tuple(dividir_partes_iguales(D("100.00"), nombres))[:1]
    with pytest.raises(ErrorDominio) as exc:
        Gasto("X", "ana", D("100.00"), cuotas, AHORA)
    assert exc.value.campo == "cuotas"

def test_gasto_sin_cuotas():
    with pytest.raises(ErrorDominio) as exc:
        Gasto("X", "ana", D("10.00"), (), AHORA)
    assert exc.value.campo == "cuotas"

def test_gasto_fecha_sin_zona_horaria():
    cuotas = tuple(dividir_partes_iguales(D("10.00"), {"ana": "ana"}))
    with pytest.raises(ErrorDominio) as exc:
        Gasto("X", "ana", D("10.00"), cuotas, datetime.now())
    assert exc.value.campo == "fecha"

@pytest.mark.parametrize("monto", ["0", "-1", "10.005"])
def test_gasto_monto_invalido(monto):
    cuotas = tuple(dividir_partes_iguales(D("10.00"), {"ana": "ana"}))
    with pytest.raises(ErrorDominio) as exc:
        Gasto("X", "ana", D(monto), cuotas, AHORA)
    assert exc.value.campo == "monto"