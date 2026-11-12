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

def test_ejemplo_oficial_100_entre_tres():
    nombres = {"carla": "carla", "ana": "ana", "beto": "beto"}
    r = como_dict(dividir_partes_iguales(D("100.00"), nombres))
    assert r == {
        "ana": (D("33.34"), True),
        "beto": (D("33.33"), False),
        "carla": (D("33.33"), False),
    }

def test_el_resultado_sale_ordenado_alfabeticamente():
    nombres = {"carla": "carla", "ana": "ana", "beto": "beto"}
    cuotas = dividir_partes_iguales(D("100.00"), nombres)
    assert [c.nombre_usuario for c in cuotas] == ["ana", "beto", "carla"]

def test_un_solo_participante_no_lleva_residuo():
    r = dividir_partes_iguales(D("50.00"), {"ana": "ana"})
    assert len(r) == 1 and r[0].monto == D("50.00") and not r[0].lleva_residuo

def test_division_exacta_no_genera_residuo():
    assert all(not c.lleva_residuo for c in dividir_partes_iguales(D("90.00"), {"a": "a", "b": "b", "c": "c"}))

def test_montos_grandes_suman_exactamente_el_total():
    total = D("999999999999.99")
    nombres = {l: l for l in ["a", "b", "c", "d", "e", "f", "g"]}
    cuotas = dividir_partes_iguales(total, nombres)
    assert sum((c.monto for c in cuotas), D("0")) == total

def test_partes_iguales_sin_participantes():
    assert campo(dividir_partes_iguales, D("10.00"), {}).campo == "participantes"

def test_ejemplo_oficial_porcentajes():
    pct = {"ana": D("33.33"), "beto": D("33.33"), "carla": D("33.34")}
    nombres = {"ana": "ana", "beto": "beto", "carla": "carla"}
    r = como_dict(dividir_por_porcentajes(D("10.00"), pct, nombres))
    assert r == {
        "ana": (D("3.34"), True),
        "beto": (D("3.33"), False),
        "carla": (D("3.33"), False),
    }

def test_porcentajes_50_25_25():
    pct = {"ana": D("50"), "beto": D("25"), "carla": D("25")}
    nombres = {"ana": "ana", "beto": "beto", "carla": "carla"}
    r = como_dict(dividir_por_porcentajes(D("60.00"), pct, nombres))
    assert r["ana"][0] == D("30.00") and r["beto"][0] == D("15.00")

def test_porcentajes_que_faltan():
    nombres = {"ana": "ana", "beto": "beto"}
    e = campo(dividir_por_porcentajes, D("10.00"), {"ana": D("50"), "beto": D("45")}, nombres)
    assert e.campo == "porcentajes" and "faltan" in e.mensaje

def test_porcentajes_que_sobran():
    nombres = {"ana": "ana", "beto": "beto"}
    e = campo(dividir_por_porcentajes, D("10.00"), {"ana": D("60"), "beto": D("45")}, nombres)
    assert e.campo == "porcentajes" and "sobran" in e.mensaje

@pytest.mark.parametrize("pct", [
    {"ana": D("-5"), "beto": D("105")},
    {"ana": D("33.333"), "beto": D("66.667")},
    {"ana": D("101"), "beto": D("-1")},
])
def test_porcentaje_individual_invalido(pct):
    nombres = {"ana": "ana", "beto": "beto"}
    assert campo(dividir_por_porcentajes, D("10.00"), pct, nombres).campo == "porcentajes"

def test_porcentajes_vacios():
    assert campo(dividir_por_porcentajes, D("10.00"), {}, {}).campo == "participantes"

def test_ejemplo_oficial_montos_fijos():
    m = {"ana": D("50.00"), "beto": D("40.00"), "carla": D("30.00")}
    nombres = {"ana": "ana", "beto": "beto", "carla": "carla"}
    r = como_dict(dividir_por_montos(D("120.00"), m, nombres))
    assert r == {
        "ana": (D("50.00"), False), "beto": (D("40.00"), False), "carla": (D("30.00"), False),
    }

def test_montos_que_no_suman_indican_la_diferencia():
    nombres = {"ana": "ana", "beto": "beto"}
    e = campo(dividir_por_montos, D("120.00"), {"ana": D("50.00"), "beto": D("68.00")}, nombres)
    assert e.campo == "montos" and "faltan" in e.mensaje

def test_montos_vacios():
    assert campo(dividir_por_montos, D("120.00"), {}, {}).campo == "participantes"

@pytest.mark.parametrize("montos", [
    {"ana": D("-1"), "beto": D("121")},
    {"ana": D("60.005"), "beto": D("59.995")},
])
def test_monto_individual_invalido(montos):
    nombres = {"ana": "ana", "beto": "beto"}
    assert campo(dividir_por_montos, D("120.00"), montos, nombres).campo == "monto"

def test_total_cero_o_negativo():
    assert campo(repartir_residuo, D("0"), {"ana": D("1")}, {"ana": "ana"}).campo == "monto"
    assert campo(repartir_residuo, D("-5"), {"ana": D("1")}, {"ana": "ana"}).campo == "monto"

def test_total_con_mas_de_dos_decimales():
    assert campo(repartir_residuo, D("100.005"), {"ana": D("100.005")}, {"ana": "ana"}).campo == "monto"

def test_cuotas_que_no_cuadran_con_el_total():
    nombres = {"ana": "ana", "beto": "beto"}
    e = campo(repartir_residuo, D("100.00"), {"ana": D("50"), "beto": D("40")}, nombres)
    assert e.campo == "cuotas"

def test_sin_cuotas():
    assert campo(repartir_residuo, D("100.00"), {}, {}).campo == "participantes"

def test_rechaza_nombres_de_usuario_repetidos_entre_participantes():
    # Dos IDs distintos pero con el mismo username (rompería el orden determinista)
    nombres_con_duplicado = {"id-1": "ana", "id-2": "ana"}
    assert campo(dividir_partes_iguales, D("100.00"), nombres_con_duplicado).campo == "participantes"