from dataclasses import FrozenInstanceError
from datetime import datetime, timezone

import pytest

from dominio.errores import ErrorDominio
from dominio.usuario import Usuario

AHORA = datetime.now(timezone.utc)


def crear(nombre_usuario="ana", nombre_visible="Ana", correo="a@x.com", fecha=AHORA):
    return Usuario("1", nombre_usuario, nombre_visible, correo, fecha)


def campo_del_error(**kwargs):
    with pytest.raises(ErrorDominio) as exc:
        crear(**kwargs)
    return exc.value.campo


def test_usuario_valido():
    assert crear().nombre_usuario == "ana"


@pytest.mark.parametrize("nombre", ["abc", "a" * 30, "ana.lopez", "ana_01", "a1b"])
def test_nombres_de_usuario_validos(nombre):
    assert crear(nombre_usuario=nombre).nombre_usuario == nombre


@pytest.mark.parametrize(
    "nombre", ["Ana", "ab", "a" * 31, "ana-01", "ana lopez", "añá", "", "ana!"]
)
def test_nombres_de_usuario_invalidos(nombre):
    assert campo_del_error(nombre_usuario=nombre) == "nombre_usuario"


@pytest.mark.parametrize("visible", ["", "   "])
def test_nombre_visible_obligatorio(visible):
    assert campo_del_error(nombre_visible=visible) == "nombre_visible"


@pytest.mark.parametrize("correo", ["sin-arroba", "a@x", "@x.com", "a b@x.com", ""])
def test_correo_invalido(correo):
    assert campo_del_error(correo=correo) == "correo"


def test_correo_se_normaliza_a_minusculas_y_sin_espacios():
    assert crear(correo="  ANA@X.com ").correo == "ana@x.com"


def test_nombre_visible_se_guarda_sin_espacios_extremos_y_sin_cambiar_mayusculas():
    assert crear(nombre_visible="  Ana López ").nombre_visible == "Ana López"


def test_fecha_alta_exige_zona_horaria():
    assert campo_del_error(fecha=datetime.now()) == "fecha_alta"


def test_usuario_es_inmutable():
    with pytest.raises(FrozenInstanceError):
        crear().correo = "otro@x.com"