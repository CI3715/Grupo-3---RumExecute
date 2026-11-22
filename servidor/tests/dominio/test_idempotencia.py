from decimal import Decimal as D

import pytest

from dominio.errores import ErrorDominio
from dominio.idempotencia import (
    calcular_huella,
    RegistroIdempotenciaGasto,
    ErrorConflictoIdempotencia,
    verificar_reintento,
)


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

# --- Validación de montos ---

@pytest.mark.parametrize("monto", [
    "10.00",
    10.0,
    D("NaN"),
    D("Infinity"),
    D("-Infinity"),
])
def test_rechaza_monto_no_decimal_o_no_finito(monto):
    with pytest.raises(ErrorDominio) as exc:
        huella(monto=monto)

    assert exc.value.campo == "monto"


@pytest.mark.parametrize("valor", [
    "50",
    50.0,
    D("NaN"),
    D("Infinity"),
])
def test_rechaza_valores_no_decimales_o_no_finitos(valor):
    with pytest.raises(ErrorDominio) as exc:
        huella(valores={"usuario-1": valor})

    assert exc.value.campo == "valores"


# --- Registro de idempotencia ---

def registro_valido():
    return RegistroIdempotenciaGasto(
        clave="clave-123",
        usuario_id="usuario-1",
        huella=huella(),
        gasto_id="gasto-1",
    )


def test_crear_registro_valido():
    registro = registro_valido()

    assert registro.clave == "clave-123"
    assert registro.usuario_id == "usuario-1"
    assert registro.huella == huella()
    assert registro.gasto_id == "gasto-1"


@pytest.mark.parametrize("campo", [
    "clave",
    "usuario_id",
    "huella",
    "gasto_id",
])
def test_rechaza_campos_vacios_en_registro(campo):
    datos = {
        "clave": "clave-123",
        "usuario_id": "usuario-1",
        "huella": huella(),
        "gasto_id": "gasto-1",
    }

    datos[campo] = "   "

    with pytest.raises(ErrorDominio) as exc:
        RegistroIdempotenciaGasto(**datos)

    assert exc.value.campo == campo


def test_registro_es_inmutable():
    from dataclasses import FrozenInstanceError

    registro = registro_valido()

    with pytest.raises(FrozenInstanceError):
        registro.gasto_id = "gasto-2"


# --- Verificación de reintentos ---

def test_reintento_identico_devuelve_gasto_original():
    registro = registro_valido()

    resultado = verificar_reintento(
        registro,
        usuario_id="usuario-1",
        huella=huella(),
    )

    assert resultado == "gasto-1"


def test_reintento_con_monto_distinto_produce_conflicto():
    registro = registro_valido()

    nueva_huella = huella(monto=D("20.00"))

    with pytest.raises(ErrorConflictoIdempotencia) as exc:
        verificar_reintento(
            registro,
            usuario_id="usuario-1",
            huella=nueva_huella,
        )

    assert exc.value.campo == "idempotency_key"


def test_reintento_con_usuario_distinto_produce_conflicto():
    registro = registro_valido()

    with pytest.raises(ErrorConflictoIdempotencia):
        verificar_reintento(
            registro,
            usuario_id="usuario-2",
            huella=huella(),
        )


def test_error_idempotencia_es_error_conflicto():
    from dominio.errores import ErrorConflicto

    error = ErrorConflictoIdempotencia()

    assert isinstance(error, ErrorConflicto)
    assert error.campo == "idempotency_key"
