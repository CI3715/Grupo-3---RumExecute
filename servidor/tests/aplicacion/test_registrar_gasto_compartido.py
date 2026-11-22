from dataclasses import replace
from decimal import Decimal as D
from unittest.mock import Mock
from uuid import UUID
from datetime import datetime, timezone

import pytest

from aplicacion.errores import GrupoNoEncontrado
from aplicacion.registrar_gasto_compartido import (
    RegistrarGastoCompartido,
)
from dominio.balance import Gasto
from dominio.errores import ErrorDominio
from dominio.grupo import GrupoCompartido
from dominio.idempotencia import (
    calcular_huella,
    RegistroIdempotenciaGasto,
    ErrorConflictoIdempotencia
)
from dominio.division import dividir_partes_iguales


@pytest.fixture
def repositorios(usuario_existente):
    ana = replace(
        usuario_existente,
        id="usuario-1",
        nombre_usuario="ana",
    )
    beto = replace(
        usuario_existente,
        id="usuario-2",
        nombre_usuario="beto",
    )
    carla = replace(
        usuario_existente,
        id="usuario-3",
        nombre_usuario="carla",
    )

    usuarios = {
        usuario.id: usuario
        for usuario in (ana, beto, carla)
    }

    repo_usuarios = Mock()
    repo_usuarios.obtener_usuario_por_id.side_effect = usuarios.get

    repo_grupos = Mock()
    repo_grupos.obtener_por_id.return_value = GrupoCompartido(
        id="grupo-1",
        nombre_grupo="Viaje",
        moneda="USD",
        id_miembros=tuple(usuarios.keys()),
        id_administrador="usuario-1",
        descripcion="Vacaciones",
    )

    repo_gastos = Mock()

    return repo_usuarios, repo_grupos, repo_gastos

@pytest.fixture
def repo_claves():
    claves = Mock()
    claves.buscar.return_value = None
    return claves

@pytest.fixture
def persistencia():
    return Mock()


@pytest.fixture
def caso_uso(repositorios, repo_claves, persistencia):
    return RegistrarGastoCompartido(*repositorios, repo_claves, persistencia)


def registrar(
    caso_uso,
    tipo="IGUALES",
    monto=D("10.00"),
    participantes=None,
    valores=None,
    pagador="usuario-1",
    autenticado="usuario-1",
    clave=None,
):
    if participantes is None:
        participantes = ["usuario-1", "usuario-2", "usuario-3"]

    return caso_uso.ejecutar(
        usuario_autenticado_id=autenticado,
        grupo_id="grupo-1",
        concepto="Cena",
        pagador_id=pagador,
        monto=monto,
        tipo_division=tipo,
        participantes_ids=participantes,
        valores=valores,
        clave_idempotencia=clave,
    )

def test_registrar_gasto_igualitario(caso_uso, persistencia):
    respuesta = registrar(caso_uso)

    assert respuesta.monto == D("10.00")
    assert respuesta.concepto == "Cena"
    assert respuesta.grupo_id == "grupo-1"
    assert respuesta.pagador_id == "usuario-1"

    assert [c.monto for c in respuesta.cuotas] == [
        D("3.34"),
        D("3.33"),
        D("3.33"),
    ]

    assert sum(
        (c.monto for c in respuesta.cuotas),
        D("0"),
    ) == D("10.00")

    UUID(respuesta.id)
    persistencia.guardar_gasto.assert_called_once()


def test_guarda_entidad_gasto_completa(caso_uso, persistencia):

    respuesta = registrar(caso_uso)

    gasto = persistencia.guardar_gasto.call_args.args[0]

    assert isinstance(gasto, Gasto)
    assert gasto.id == respuesta.id
    assert gasto.grupo_id == "grupo-1"
    assert gasto.pagador_id == "usuario-1"
    assert len(gasto.cuotas) == 3
    assert gasto.fecha.tzinfo is not None


def test_registrar_por_porcentajes(caso_uso):
    respuesta = registrar(
        caso_uso,
        tipo="PORCENTAJES",
        monto=D("100.00"),
        valores={
            "usuario-1": D("50"),
            "usuario-2": D("30"),
            "usuario-3": D("20"),
        },
    )

    assert [c.monto for c in respuesta.cuotas] == [
        D("50.00"),
        D("30.00"),
        D("20.00"),
    ]


def test_registrar_por_montos_fijos(caso_uso):
    respuesta = registrar(
        caso_uso,
        tipo="MONTOS",
        monto=D("40.00"),
        participantes=["usuario-1", "usuario-2"],
        valores={
            "usuario-1": D("15.00"),
            "usuario-2": D("25.00"),
        },
    )

    assert [c.monto for c in respuesta.cuotas] == [
        D("15.00"),
        D("25.00"),
    ]


def test_un_miembro_que_no_es_admin_puede_registrar(caso_uso):
    respuesta = registrar(
        caso_uso,
        autenticado="usuario-2",
    )

    assert respuesta.grupo_id == "grupo-1"


def test_usuario_ajeno_no_registra_gastos(caso_uso, repositorios):
    _, _, repo_gastos = repositorios

    with pytest.raises(GrupoNoEncontrado):
        registrar(caso_uso, autenticado="usuario-999")

    repo_gastos.guardar_gasto.assert_not_called()


def test_grupo_inexistente(caso_uso, repositorios):
    _, repo_grupos, repo_gastos = repositorios
    repo_grupos.obtener_por_id.return_value = None

    with pytest.raises(GrupoNoEncontrado):
        registrar(caso_uso)

    repo_gastos.guardar_gasto.assert_not_called()


def test_pagador_debe_ser_miembro(caso_uso, repositorios):
    _, _, repo_gastos = repositorios

    with pytest.raises(ErrorDominio) as exc:
        registrar(caso_uso, pagador="usuario-999")

    assert exc.value.campo == "pagador"
    repo_gastos.guardar_gasto.assert_not_called()


def test_participantes_deben_ser_miembros(caso_uso, repositorios):
    _, _, repo_gastos = repositorios

    with pytest.raises(ErrorDominio) as exc:
        registrar(
            caso_uso,
            participantes=["usuario-1", "usuario-999"],
        )

    assert exc.value.campo == "participantes"
    repo_gastos.guardar_gasto.assert_not_called()


def test_no_permite_participantes_duplicados(
    caso_uso, repositorios
):
    _, _, repo_gastos = repositorios

    with pytest.raises(ErrorDominio) as exc:
        registrar(
            caso_uso,
            participantes=["usuario-1", "usuario-1"],
        )

    assert exc.value.campo == "participantes"
    repo_gastos.guardar_gasto.assert_not_called()


def test_rechaza_lista_vacia(caso_uso, repositorios):
    _, _, repo_gastos = repositorios

    with pytest.raises(ErrorDominio):
        registrar(caso_uso, participantes=[])

    repo_gastos.guardar_gasto.assert_not_called()


def test_porcentajes_deben_sumar_cien(
    caso_uso, repositorios
):
    _, _, repo_gastos = repositorios

    with pytest.raises(ErrorDominio):
        registrar(
            caso_uso,
            tipo="PORCENTAJES",
            valores={
                "usuario-1": D("50"),
                "usuario-2": D("30"),
                "usuario-3": D("10"),
            },
        )

    repo_gastos.guardar_gasto.assert_not_called()


def test_montos_fijos_deben_sumar_total(
    caso_uso, repositorios
):
    _, _, repo_gastos = repositorios

    with pytest.raises(ErrorDominio):
        registrar(
            caso_uso,
            tipo="MONTOS",
            valores={
                "usuario-1": D("5"),
                "usuario-2": D("3"),
                "usuario-3": D("1"),
            },
        )

    repo_gastos.guardar_gasto.assert_not_called()


def test_valores_deben_corresponder_a_participantes(
    caso_uso, repositorios
):
    _, _, repo_gastos = repositorios

    with pytest.raises(ErrorDominio) as exc:
        registrar(
            caso_uso,
            tipo="PORCENTAJES",
            valores={"usuario-1": D("100")},
        )

    assert exc.value.campo == "valores"
    repo_gastos.guardar_gasto.assert_not_called()


def test_tipo_division_invalido(caso_uso, repositorios):
    _, _, repo_gastos = repositorios

    with pytest.raises(ErrorDominio) as exc:
        registrar(caso_uso, tipo="DESCONOCIDO")

    assert exc.value.campo == "tipo_division"
    repo_gastos.guardar_gasto.assert_not_called()


def test_guarda_residuo_correctamente(caso_uso):
    respuesta = registrar(caso_uso)

    assert [c.lleva_residuo for c in respuesta.cuotas] == [
        True,
        False,
        False,
    ]


def test_no_expone_datos_privados(caso_uso):
    respuesta = registrar(caso_uso)

    assert not hasattr(respuesta, "correo")
    assert all(
        not hasattr(cuota, "correo")
        for cuota in respuesta.cuotas
    )

def test_guarda_por_persistencia_y_genera_un_evento_por_participante(
    caso_uso, persistencia, repositorios
):
    _, _, repo_gastos = repositorios

    respuesta = registrar(caso_uso)

    persistencia.guardar_gasto.assert_called_once()
    repo_gastos.guardar_gasto.assert_not_called()

    gasto, eventos, registro = persistencia.guardar_gasto.call_args.args

    assert registro is None
    assert len(eventos) == 3
    assert {e.usuario_id for e in eventos} == {
        "usuario-1", "usuario-2", "usuario-3",
    }
    assert all(e.posicion is None for e in eventos)
    assert sum((e.monto_parte for e in eventos), D("0")) == respuesta.monto

def test_con_clave_nueva_guarda_el_registro_de_idempotencia(
    caso_uso, persistencia, repo_claves
):
    repo_claves.buscar.return_value = None

    registrar(caso_uso, clave="clave-abc")

    repo_claves.buscar.assert_called_once_with("usuario-1", "clave-abc")

    _, _, registro = persistencia.guardar_gasto.call_args.args
    assert registro is not None
    assert registro.clave == "clave-abc"
    assert registro.usuario_id == "usuario-1"

def test_reintento_con_misma_huella_no_crea_otro_gasto(
    caso_uso, persistencia, repo_claves, repositorios
):
    _, _, repo_gastos = repositorios

    huella = calcular_huella(
        grupo_id="grupo-1",
        concepto="Cena",
        pagador_id="usuario-1",
        monto=D("10.00"),
        tipo_division="IGUALES",
        participantes_ids=["usuario-1", "usuario-2", "usuario-3"],
        valores=None,
    )

    gasto_previo = Gasto(
        id="gasto-existente",
        grupo_id="grupo-1",
        concepto="Cena",
        pagador_id="usuario-1",
        monto=D("10.00"),
        cuotas=dividir_partes_iguales(
            D("10.00"),
            {"usuario-1": "ana", "usuario-2": "beto", "usuario-3": "carla"},
        ),
        fecha=datetime(2026, 11, 20, 12, 0, tzinfo=timezone.utc)
    )

    repo_claves.buscar.return_value = RegistroIdempotenciaGasto(
        clave="clave-abc",
        usuario_id="usuario-1",
        huella=huella,
        gasto_id="gasto-existente",
    )
    repo_gastos.obtener.return_value = gasto_previo

    respuesta = registrar(caso_uso, clave="clave-abc")

    persistencia.guardar_gasto.assert_not_called()
    assert respuesta.creado is False
    assert respuesta.id == "gasto-existente"

def test_misma_clave_con_datos_distintos_lanza_conflicto(
    caso_uso, persistencia, repo_claves
):
    huella_otra = calcular_huella(
        grupo_id="grupo-1",
        concepto="Almuerzo",
        pagador_id="usuario-1",
        monto=D("10.00"),
        tipo_division="IGUALES",
        participantes_ids=["usuario-1", "usuario-2", "usuario-3"],
        valores=None,
    )
    repo_claves.buscar.return_value = RegistroIdempotenciaGasto(
        clave="clave-abc",
        usuario_id="usuario-1",
        huella=huella_otra,
        gasto_id="gasto-existente",
    )

    with pytest.raises(ErrorConflictoIdempotencia):
        registrar(caso_uso, clave="clave-abc")

    persistencia.guardar_gasto.assert_not_called()

def test_sin_clave_no_consulta_ni_guarda_registro_de_idempotencia(
    caso_uso, persistencia, repo_claves
):
    registrar(caso_uso)

    repo_claves.buscar.assert_not_called()
    _, _, registro = persistencia.guardar_gasto.call_args.args
    assert registro is None 