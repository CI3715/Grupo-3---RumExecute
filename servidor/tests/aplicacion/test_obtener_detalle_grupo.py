from dataclasses import replace
from datetime import datetime, timezone
from decimal import Decimal as D
from unittest.mock import Mock

import pytest

from aplicacion.errores import (
    GrupoNoEncontrado,
    UsuarioNoEncontrado,
)
from aplicacion.obtener_detalle_grupo import ObtenerDetalleGrupo

from dominio.balance import Gasto
from dominio.division import dividir_partes_iguales
from dominio.errores import ErrorDominio
from dominio.grupo import GrupoCompartido


@pytest.fixture
def usuarios(usuario_existente):
    ana = replace(
        usuario_existente,
        id="usuario-1",
        nombre_usuario="ana",
        nombre_visible="Ana",
        correo="ana@example.com",
    )

    beto = replace(
        usuario_existente,
        id="usuario-2",
        nombre_usuario="beto",
        nombre_visible="Beto",
        correo="beto@example.com",
    )

    carla = replace(
        usuario_existente,
        id="usuario-3",
        nombre_usuario="carla",
        nombre_visible="Carla",
        correo="carla@example.com",
    )

    return {
        usuario.id: usuario
        for usuario in (ana, beto, carla)
    }


@pytest.fixture
def grupo():
    return GrupoCompartido(
        id="grupo-1",
        nombre_grupo="Viaje",
        moneda="USD",
        id_miembros=(
            "usuario-1",
            "usuario-2",
            "usuario-3",
        ),
        id_administrador="usuario-1",
        descripcion="Vacaciones",
    )


@pytest.fixture
def gasto():
    nombres = {
        "usuario-1": "ana",
        "usuario-2": "beto",
        "usuario-3": "carla",
    }

    cuotas = tuple(
        dividir_partes_iguales(
            D("60.00"),
            nombres,
        )
    )

    return Gasto(
        id="gasto-1",
        grupo_id="grupo-1",
        concepto="Cena",
        pagador_id="usuario-1",
        monto=D("60.00"),
        cuotas=cuotas,
        fecha=datetime.now(timezone.utc),
    )


@pytest.fixture
def repositorios(usuarios, grupo, gasto):
    repo_usuarios = Mock()
    repo_usuarios.obtener_usuario_por_id.side_effect = (
        usuarios.get
    )

    repo_grupos = Mock()
    repo_grupos.obtener_por_id.return_value = grupo

    repo_gastos = Mock()
    repo_gastos.listar_gastos_de_grupo.return_value = [gasto]

    return repo_usuarios, repo_grupos, repo_gastos


@pytest.fixture
def caso_uso(repositorios):
    return ObtenerDetalleGrupo(*repositorios)


def test_obtener_detalle_correctamente(caso_uso):
    resultado = caso_uso.ejecutar(
        "usuario-1", "grupo-1"
    )

    assert resultado.id == "grupo-1"
    assert resultado.nombre_grupo == "Viaje"
    assert resultado.moneda == "USD"
    assert resultado.descripcion == "Vacaciones"
    assert resultado.administrador_id == "usuario-1"
    assert len(resultado.miembros) == 3
    assert len(resultado.gastos) == 1


def test_calcula_balances_correctamente(caso_uso):
    resultado = caso_uso.ejecutar(
        "usuario-1", "grupo-1"
    )

    balances = {
        miembro.usuario_id: miembro.balance
        for miembro in resultado.miembros
    }

    assert balances == {
        "usuario-1": D("40.00"),
        "usuario-2": D("-20.00"),
        "usuario-3": D("-20.00"),
    }


def test_balances_suman_cero(caso_uso):
    resultado = caso_uso.ejecutar(
        "usuario-1", "grupo-1"
    )

    assert sum(
        (miembro.balance for miembro in resultado.miembros),
        D("0"),
    ) == D("0")


def test_identifica_administrador(caso_uso):
    resultado = caso_uso.ejecutar(
        "usuario-1", "grupo-1"
    )

    administradores = [
        miembro
        for miembro in resultado.miembros
        if miembro.es_administrador
    ]

    assert len(administradores) == 1
    assert administradores[0].usuario_id == "usuario-1"


def test_devuelve_gastos_y_cuotas(caso_uso):
    resultado = caso_uso.ejecutar(
        "usuario-1", "grupo-1"
    )

    gasto = resultado.gastos[0]

    assert gasto.id == "gasto-1"
    assert gasto.concepto == "Cena"
    assert gasto.pagador_id == "usuario-1"
    assert gasto.monto == D("60.00")
    assert len(gasto.cuotas) == 3

    assert sum(
        (cuota.monto for cuota in gasto.cuotas),
        D("0"),
    ) == gasto.monto


def test_miembro_no_administrador_puede_consultar(caso_uso):
    resultado = caso_uso.ejecutar(
        "usuario-2", "grupo-1"
    )

    assert resultado.id == "grupo-1"
    assert len(resultado.gastos) == 1


def test_grupo_sin_gastos(
    caso_uso, repositorios
):
    _, _, repo_gastos = repositorios
    repo_gastos.listar_gastos_de_grupo.return_value = []

    resultado = caso_uso.ejecutar(
        "usuario-1", "grupo-1"
    )

    assert resultado.gastos == ()
    assert all(
        miembro.balance == D("0")
        for miembro in resultado.miembros
    )


def test_grupo_inexistente(
    caso_uso, repositorios
):
    repo_usuarios, repo_grupos, repo_gastos = repositorios
    repo_grupos.obtener_por_id.return_value = None

    with pytest.raises(GrupoNoEncontrado):
        caso_uso.ejecutar(
            "usuario-1", "grupo-inexistente"
        )

    repo_gastos.listar_gastos_de_grupo.assert_not_called()
    repo_usuarios.obtener_usuario_por_id.assert_not_called()


def test_usuario_ajeno_no_puede_consultar(
    caso_uso, repositorios
):
    repo_usuarios, _, repo_gastos = repositorios

    with pytest.raises(GrupoNoEncontrado):
        caso_uso.ejecutar(
            "usuario-999", "grupo-1"
        )

    repo_gastos.listar_gastos_de_grupo.assert_not_called()
    repo_usuarios.obtener_usuario_por_id.assert_not_called()


def test_usuario_miembro_inexistente(
    caso_uso, repositorios
):
    repo_usuarios, _, _ = repositorios
    repo_usuarios.obtener_usuario_por_id.side_effect = (
        lambda uid: None if uid == "usuario-2"
        else {
            "usuario-1": Mock(
                id="usuario-1",
                nombre_usuario="ana",
                nombre_visible="Ana",
            ),
            "usuario-3": Mock(
                id="usuario-3",
                nombre_usuario="carla",
                nombre_visible="Carla",
            ),
        }.get(uid)
    )

    with pytest.raises(UsuarioNoEncontrado):
        caso_uso.ejecutar(
            "usuario-1", "grupo-1"
        )


def test_no_mezcla_gastos_de_otros_grupos(
    caso_uso, repositorios, gasto
):
    _, _, repo_gastos = repositorios

    repo_gastos.listar_gastos_de_grupo.return_value = [
        replace(gasto, grupo_id="grupo-2")
    ]

    with pytest.raises(ErrorDominio) as exc:
        caso_uso.ejecutar(
            "usuario-1", "grupo-1"
        )

    assert exc.value.campo == "gastos"


def test_rechaza_gasto_con_pagador_ajeno(
    caso_uso, repositorios, gasto
):
    _, _, repo_gastos = repositorios

    repo_gastos.listar_gastos_de_grupo.return_value = [
        replace(gasto, pagador_id="usuario-999")
    ]

    with pytest.raises(ErrorDominio) as exc:
        caso_uso.ejecutar(
            "usuario-1", "grupo-1"
        )

    assert exc.value.campo == "pagador"


def test_no_expone_correos(caso_uso):
    resultado = caso_uso.ejecutar(
        "usuario-1", "grupo-1"
    )

    assert all(
        not hasattr(miembro, "correo")
        for miembro in resultado.miembros
    )

    assert all(
        not hasattr(gasto, "correo")
        for gasto in resultado.gastos
    )


def test_consulta_repositorios_con_ids(
    caso_uso, repositorios
):
    _, repo_grupos, repo_gastos = repositorios

    caso_uso.ejecutar(
        "usuario-1", "grupo-1"
    )

    repo_grupos.obtener_por_id.assert_called_once_with(
        "grupo-1"
    )

    repo_gastos.listar_gastos_de_grupo.assert_called_once_with(
        "grupo-1"
    )
