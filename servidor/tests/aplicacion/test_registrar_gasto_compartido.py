from dataclasses import replace
from decimal import Decimal as D
from unittest.mock import Mock
from uuid import UUID

import pytest

from aplicacion.errores import GrupoNoEncontrado
from aplicacion.registrar_gasto_compartido import (
    RegistrarGastoCompartido,
)
from dominio.balance import Gasto
from dominio.errores import ErrorDominio
from dominio.grupo import GrupoCompartido


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
def caso_uso(repositorios):
    return RegistrarGastoCompartido(*repositorios)


def registrar(
    caso_uso,
    tipo="IGUALES",
    monto=D("10.00"),
    participantes=None,
    valores=None,
    pagador="usuario-1",
    autenticado="usuario-1",
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
    )


def test_registrar_gasto_igualitario(caso_uso, repositorios):
    _, _, repo_gastos = repositorios

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
    repo_gastos.guardar_gasto.assert_called_once()


def test_guarda_entidad_gasto_completa(caso_uso, repositorios):
    _, _, repo_gastos = repositorios

    respuesta = registrar(caso_uso)

    gasto = repo_gastos.guardar_gasto.call_args.args[0]

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
