from decimal import Decimal as D

import pytest

from dominio.idempotencia import calcular_huella


def huella(**cambios):
    """Calcula la huella con datos base; cada prueba cambia solo lo suyo."""
    datos = dict(
        grupo_id="grupo-1",
        concepto="Cena",
        pagador_id="usuario-1",
        monto=D("10.00"),
        tipo_division="IGUALES",
        participantes_ids=["usuario-1", "usuario-2", "usuario-3"],
        valores=None,
    )
    datos.update(cambios)
    return calcular_huella(**datos)


# --- Mismos datos, distinta forma: la huella debe ser IGUAL ---

def test_misma_huella_con_participantes_en_otro_orden():
    assert huella(
        participantes_ids=["usuario-1", "usuario-2", "usuario-3"]
    ) == huella(
        participantes_ids=["usuario-3", "usuario-1", "usuario-2"]
    )


def test_misma_huella_con_monto_en_distinta_escala():
    assert huella(monto=D("10.00")) == huella(monto=D("10"))


def test_misma_huella_con_valores_en_distinta_escala():
    base = dict(
        tipo_division="PORCENTAJES",
        participantes_ids=["usuario-1", "usuario-2"],
    )
    assert huella(
        **base, valores={"usuario-1": D("50"), "usuario-2": D("50")}
    ) == huella(
        **base, valores={"usuario-1": D("50.00"), "usuario-2": D("50.0")}
    )


def test_misma_huella_con_valores_en_otro_orden():
    base = dict(
        tipo_division="MONTOS",
        participantes_ids=["usuario-1", "usuario-2"],
    )
    assert huella(
        **base, valores={"usuario-1": D("4"), "usuario-2": D("6")}
    ) == huella(
        **base, valores={"usuario-2": D("6"), "usuario-1": D("4")}
    )


def test_misma_huella_con_espacios_en_el_concepto():
    assert huella(concepto="Cena") == huella(concepto="  Cena ")


def test_misma_huella_con_valores_none_o_vacio():
    assert huella(valores=None) == huella(valores={})


def test_la_huella_es_determinista():
    assert huella() == huella()


# --- Datos distintos: la huella debe ser DISTINTA ---

@pytest.mark.parametrize(
    "cambio",
    [
        {"grupo_id": "grupo-2"},
        {"concepto": "Almuerzo"},
        {"pagador_id": "usuario-2"},
        {"monto": D("20.00")},
        {"tipo_division": "PORCENTAJES"},
        {"participantes_ids": ["usuario-1", "usuario-2"]},
        {"valores": {"usuario-1": D("10")}},
    ],
)
def test_huella_distinta_si_cambia_un_dato(cambio):
    assert huella() != huella(**cambio)