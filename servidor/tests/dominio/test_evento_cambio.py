from dataclasses import FrozenInstanceError
from datetime import datetime, timedelta, timezone
from decimal import Decimal as D

import pytest

from dominio.errores import ErrorDominio
from dominio.evento_cambio import EventoCambioUsuario


def evento_valido(**cambios):
    datos = {
        "id": "evento-1",
        "posicion": 1,
        "usuario_id": "usuario-1",
        "grupo_id": "grupo-1",
        "nombre_grupo": "Viaje",
        "gasto_id": "gasto-1",
        "concepto": "Hotel",
        "fecha": datetime(2026, 11, 22, tzinfo=timezone.utc),
        "moneda": "USD",
        "monto_parte": D("33.34"),
        "incluye_redondeo": True,
    }

    datos.update(cambios)

    return EventoCambioUsuario(**datos)


# --- Construcción ---

def test_crear_evento_valido():
    evento = evento_valido()

    assert evento.id == "evento-1"
    assert evento.posicion == 1
    assert evento.usuario_id == "usuario-1"
    assert evento.grupo_id == "grupo-1"
    assert evento.nombre_grupo == "Viaje"
    assert evento.gasto_id == "gasto-1"
    assert evento.concepto == "Hotel"
    assert evento.moneda == "USD"
    assert evento.monto_parte == D("33.34")
    assert evento.incluye_redondeo is True


def test_evento_es_inmutable():
    evento = evento_valido()

    with pytest.raises(FrozenInstanceError):
        evento.monto_parte = D("50.00")


# --- Identificadores ---

@pytest.mark.parametrize("campo", [
    "id",
    "usuario_id",
    "grupo_id",
    "gasto_id",
])
def test_rechaza_identificadores_vacios(campo):
    with pytest.raises(ErrorDominio) as exc:
        evento_valido(**{campo: "   "})

    assert exc.value.campo == campo


# --- Posición del cursor ---

@pytest.mark.parametrize("posicion", [
    0,
    -1,
    -10,
])
def test_rechaza_posiciones_no_positivas(posicion):
    with pytest.raises(ErrorDominio) as exc:
        evento_valido(posicion=posicion)

    assert exc.value.campo == "posicion"


def test_acepta_posiciones_positivas():
    evento = evento_valido(posicion=100)

    assert evento.posicion == 100


# --- Fecha ---

def test_rechaza_fecha_sin_zona_horaria():
    fecha = datetime(2026, 11, 22)

    with pytest.raises(ErrorDominio) as exc:
        evento_valido(fecha=fecha)

    assert exc.value.campo == "fecha"


def test_rechaza_fecha_fuera_de_utc():
    fecha = datetime(
        2026, 11, 22,
        tzinfo=timezone(timedelta(hours=-4))
    )

    with pytest.raises(ErrorDominio) as exc:
        evento_valido(fecha=fecha)

    assert exc.value.campo == "fecha"


def test_acepta_fecha_utc():
    fecha = datetime(
        2026, 11, 22, 15, 30,
        tzinfo=timezone.utc
    )

    evento = evento_valido(fecha=fecha)

    assert evento.fecha == fecha


# --- Montos ---

@pytest.mark.parametrize("monto", [
    D("-0.01"),
    D("-10.00"),
    D("NaN"),
    D("Infinity"),
    D("-Infinity"),
])
def test_rechaza_montos_negativos_o_no_finitos(monto):
    with pytest.raises(ErrorDominio) as exc:
        evento_valido(monto_parte=monto)

    assert exc.value.campo == "monto_parte"


@pytest.mark.parametrize("monto", [
    "10.00",
    10.0,
    10,
    None,
])
def test_rechaza_montos_que_no_son_decimal(monto):
    with pytest.raises(ErrorDominio) as exc:
        evento_valido(monto_parte=monto)

    assert exc.value.campo == "monto_parte"


def test_acepta_monto_cero():
    evento = evento_valido(monto_parte=D("0.00"))

    assert evento.monto_parte == D("0.00")


def test_acepta_monto_con_residuo():
    evento = evento_valido(
        monto_parte=D("33.34"),
        incluye_redondeo=True,
    )

    assert evento.monto_parte == D("33.34")
    assert evento.incluye_redondeo is True


def test_acepta_monto_sin_residuo():
    evento = evento_valido(
        monto_parte=D("33.33"),
        incluye_redondeo=False,
    )

    assert evento.monto_parte == D("33.33")
    assert evento.incluye_redondeo is False
