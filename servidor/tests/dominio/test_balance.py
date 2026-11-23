from dataclasses import FrozenInstanceError
from datetime import datetime, timedelta, timezone
from decimal import Decimal as D
import pytest

from dominio.balance import Gasto, calcular_balances, validar_balances_suma_cero
from dominio.division import CuotaDivision, dividir_partes_iguales, dividir_por_porcentajes
from dominio.errores import ErrorDominio

AHORA = datetime.now(timezone.utc)
MIEMBROS = ("ana", "beto", "carla")

def gasto_valido(**cambios):
    datos = {
        "id": "gasto-1",
        "grupo_id": "grupo-1",
        "concepto": "Hotel",
        "pagador_id": "usuario-1",
        "monto": D("30.00"),
        "cuotas": (
            CuotaDivision(
                usuario_id="usuario-1",
                nombre_usuario="ana",
                monto=D("15.00"),
                lleva_residuo=False,
            ),
            CuotaDivision(
                usuario_id="usuario-2",
                nombre_usuario="beto",
                monto=D("15.00"),
                lleva_residuo=False,
            ),
        ),
        "fecha": datetime(2026, 11, 22, tzinfo=timezone.utc),
    }

    datos.update(cambios)
    return Gasto(**datos)

def gasto_iguales(pagador="ana", total="100.00", participantes=MIEMBROS, concepto="Cena"):
    nombres = {p: p for p in participantes}
    cuotas = tuple(dividir_partes_iguales(D(total), nombres))
    return Gasto("gasto-1", "grupo-1", concepto, pagador, D(total), cuotas, AHORA)

def gasto_porcentajes():
    pct = {"ana": D("50"), "beto": D("25"), "carla": D("25")}
    nombres = {p: p for p in pct.keys()}
    cuotas = tuple(dividir_por_porcentajes(D("60.00"), pct, nombres))
    return Gasto("gasto-2", "grupo-1", "Taxi", "beto", D("60.00"), cuotas, AHORA)

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
        Gasto("gasto-1", "grupo-1", "X", "ana", D("100.00"), cuotas, AHORA)
    assert exc.value.campo == "cuotas"

def test_gasto_sin_cuotas():
    with pytest.raises(ErrorDominio) as exc:
        Gasto("gasto-1", "grupo-1", "X", "ana", D("10.00"), (), AHORA)
    assert exc.value.campo == "cuotas"

def test_gasto_fecha_sin_zona_horaria():
    cuotas = tuple(dividir_partes_iguales(D("10.00"), {"ana": "ana"}))
    with pytest.raises(ErrorDominio) as exc:
        Gasto("gasto-1", "grupo-1", "X", "ana", D("10.00"), cuotas, datetime.now())
    assert exc.value.campo == "fecha"

@pytest.mark.parametrize("monto", ["0", "-1", "10.005"])
def test_gasto_monto_invalido(monto):
    cuotas = tuple(dividir_partes_iguales(D("10.00"), {"ana": "ana"}))
    with pytest.raises(ErrorDominio) as exc:
        Gasto("gasto-1", "grupo-1", "X", "ana", D(monto), cuotas, AHORA)
    assert exc.value.campo == "monto"


def test_crear_gasto_valido():
    gasto = gasto_valido()

    assert gasto.id == "gasto-1"
    assert gasto.monto == D("30.00")
    assert len(gasto.cuotas) == 2


def test_gasto_es_inmutable():
    gasto = gasto_valido()

    with pytest.raises(FrozenInstanceError):
        gasto.monto = D("50.00")


@pytest.mark.parametrize("campo", [
    "id",
    "grupo_id",
    "pagador_id",
])
def test_gasto_rechaza_identificadores_vacios(campo):
    with pytest.raises(ErrorDominio) as exc:
        gasto_valido(**{campo: "   "})

    assert exc.value.campo == campo


def test_gasto_rechaza_concepto_vacio():
    with pytest.raises(ErrorDominio) as exc:
        gasto_valido(concepto="   ")

    assert exc.value.campo == "concepto"


def test_gasto_normaliza_concepto():
    gasto = gasto_valido(concepto="  Hotel  ")

    assert gasto.concepto == "Hotel"


@pytest.mark.parametrize("monto", [
    "30.00",
    30.0,
    None,
    D("NaN"),
    D("Infinity"),
    D("-Infinity"),
])
def test_gasto_rechaza_montos_no_decimales_o_no_finitos(monto):
    with pytest.raises(ErrorDominio) as exc:
        gasto_valido(monto=monto)

    assert exc.value.campo == "monto"


@pytest.mark.parametrize("monto", [
    D("0"),
    D("-1.00"),
    D("-10.00"),
])
def test_gasto_rechaza_montos_no_positivos(monto):
    with pytest.raises(ErrorDominio) as exc:
        gasto_valido(monto=monto)

    assert exc.value.campo == "monto"


def test_gasto_rechaza_mas_de_dos_decimales():
    with pytest.raises(ErrorDominio) as exc:
        gasto_valido(monto=D("30.001"))

    assert exc.value.campo == "monto"


def test_gasto_rechaza_cuotas_vacias():
    with pytest.raises(ErrorDominio) as exc:
        gasto_valido(cuotas=())

    assert exc.value.campo == "cuotas"


def test_gasto_rechaza_cuotas_de_tipo_incorrecto():
    with pytest.raises(ErrorDominio) as exc:
        gasto_valido(cuotas=("15.00", "15.00"))

    assert exc.value.campo == "cuotas"


def test_gasto_rechaza_participantes_repetidos():
    cuota = CuotaDivision(
        usuario_id="usuario-1",
        nombre_usuario="ana",
        monto=D("15.00"),
        lleva_residuo=False,
    )

    with pytest.raises(ErrorDominio) as exc:
        gasto_valido(cuotas=(cuota, cuota))

    assert exc.value.campo == "cuotas"


def test_gasto_rechaza_suma_incorrecta():
    cuota = CuotaDivision(
        usuario_id="usuario-1",
        nombre_usuario="ana",
        monto=D("10.00"),
        lleva_residuo=False,
    )

    with pytest.raises(ErrorDominio) as exc:
        gasto_valido(cuotas=(cuota,))

    assert exc.value.campo == "cuotas"


def test_gasto_rechaza_fecha_sin_zona_horaria():
    with pytest.raises(ErrorDominio) as exc:
        gasto_valido(fecha=datetime(2026, 11, 22))

    assert exc.value.campo == "fecha"


def test_gasto_normaliza_fecha_a_utc():
    fecha = datetime(
        2026, 11, 22, 12, 0,
        tzinfo=timezone(timedelta(hours=-4))
    )

    gasto = gasto_valido(fecha=fecha)

    assert gasto.fecha.utcoffset() == timedelta(0)
    assert gasto.fecha.hour == 16
