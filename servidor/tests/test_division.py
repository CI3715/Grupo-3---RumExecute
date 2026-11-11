from decimal import Decimal as D

import pytest

from dominio.division import (
    dividir_partes_iguales,
    dividir_por_montos,
    dividir_por_porcentajes,
    repartir_residuo,
)
from dominio.errores import ErrorDominio


def como_dict(cuotas):
    return {c.nombre_usuario: (c.monto, c.lleva_residuo) for c in cuotas}


def campo(funcion, *args):
    with pytest.raises(ErrorDominio) as exc:
        funcion(*args)
    return exc.value


# ---------- partes iguales ----------

def test_ejemplo_oficial_100_entre_tres():
    r = como_dict(dividir_partes_iguales(D("100.00"), ["carla", "ana", "beto"]))
    assert r == {
        "ana": (D("33.34"), True),
        "beto": (D("33.33"), False),
        "carla": (D("33.33"), False),
    }


def test_el_resultado_sale_ordenado_alfabeticamente():
    cuotas = dividir_partes_iguales(D("100.00"), ["carla", "ana", "beto"])
    assert [c.nombre_usuario for c in cuotas] == ["ana", "beto", "carla"]


def test_un_solo_participante_no_lleva_residuo():
    r = dividir_partes_iguales(D("50.00"), ["ana"])
    assert len(r) == 1 and r[0].monto == D("50.00") and not r[0].lleva_residuo


def test_division_exacta_no_genera_residuo():
    assert all(not c.lleva_residuo for c in dividir_partes_iguales(D("90.00"), ["a", "b", "c"]))


def test_montos_grandes_suman_exactamente_el_total():
    total = D("999999999999.99")
    cuotas = dividir_partes_iguales(total, ["a", "b", "c", "d", "e", "f", "g"])
    assert sum((c.monto for c in cuotas), D("0")) == total


def test_partes_iguales_sin_participantes():
    assert campo(dividir_partes_iguales, D("10.00"), []).campo == "participantes"


def test_partes_iguales_con_duplicados():
    assert campo(dividir_partes_iguales, D("10.00"), ["ana", "ana"]).campo == "participantes"


# ---------- porcentajes ----------

def test_ejemplo_oficial_porcentajes():
    pct = {"ana": D("33.33"), "beto": D("33.33"), "carla": D("33.34")}
    r = como_dict(dividir_por_porcentajes(D("10.00"), pct))
    assert r == {
        "ana": (D("3.34"), True),
        "beto": (D("3.33"), False),
        "carla": (D("3.33"), False),
    }


def test_porcentajes_50_25_25():
    pct = {"ana": D("50"), "beto": D("25"), "carla": D("25")}
    r = como_dict(dividir_por_porcentajes(D("60.00"), pct))
    assert r["ana"][0] == D("30.00") and r["beto"][0] == D("15.00")


def test_porcentajes_que_faltan():
    e = campo(dividir_por_porcentajes, D("10.00"), {"ana": D("50"), "beto": D("45")})
    assert e.campo == "porcentajes" and "faltan" in e.mensaje and "5" in e.mensaje


def test_porcentajes_que_sobran():
    e = campo(dividir_por_porcentajes, D("10.00"), {"ana": D("60"), "beto": D("45")})
    assert e.campo == "porcentajes" and "sobran" in e.mensaje and "5" in e.mensaje


@pytest.mark.parametrize(
    "pct",
    [
        {"ana": D("-5"), "beto": D("105")},
        {"ana": D("33.333"), "beto": D("66.667")},
        {"ana": D("101"), "beto": D("-1")},
    ],
)
def test_porcentaje_individual_invalido(pct):
    assert campo(dividir_por_porcentajes, D("10.00"), pct).campo == "porcentajes"


def test_porcentajes_vacios():
    assert campo(dividir_por_porcentajes, D("10.00"), {}).campo == "participantes"


# ---------- montos fijos ----------

def test_ejemplo_oficial_montos_fijos():
    m = {"ana": D("50.00"), "beto": D("40.00"), "carla": D("30.00")}
    r = como_dict(dividir_por_montos(D("120.00"), m))
    assert r == {
        "ana": (D("50.00"), False),
        "beto": (D("40.00"), False),
        "carla": (D("30.00"), False),
    }


def test_montos_que_no_suman_indican_la_diferencia():
    e = campo(dividir_por_montos, D("120.00"), {"ana": D("50.00"), "beto": D("68.00")})
    assert e.campo == "montos" and "118" in e.mensaje and "faltan" in e.mensaje and "2" in e.mensaje


def test_montos_vacios():
    assert campo(dividir_por_montos, D("120.00"), {}).campo == "participantes"


@pytest.mark.parametrize(
    "montos",
    [
        {"ana": D("-1"), "beto": D("121")},
        {"ana": D("60.005"), "beto": D("59.995")},
    ],
)
def test_monto_individual_invalido(montos):
    assert campo(dividir_por_montos, D("120.00"), montos).campo == "monto"


# ---------- núcleo: repartir_residuo ----------

def test_total_cero_o_negativo():
    assert campo(repartir_residuo, D("0"), {"ana": D("1")}).campo == "monto"
    assert campo(repartir_residuo, D("-5"), {"ana": D("1")}).campo == "monto"


def test_total_con_mas_de_dos_decimales():
    assert campo(repartir_residuo, D("100.005"), {"ana": D("100.005")}).campo == "monto"


def test_cuotas_que_no_cuadran_con_el_total():
    e = campo(repartir_residuo, D("100.00"), {"ana": D("50"), "beto": D("40")})
    assert e.campo == "cuotas"


def test_sin_cuotas():
    assert campo(repartir_residuo, D("100.00"), {}).campo == "participantes"