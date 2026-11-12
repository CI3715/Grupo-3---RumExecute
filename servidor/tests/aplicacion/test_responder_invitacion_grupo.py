from dataclasses import replace
from unittest.mock import Mock

import pytest

from aplicacion.errores import GrupoNoEncontrado
from aplicacion.responder_invitacion_grupo import (
    ResponderInvitacionGrupo,
)

from dominio.errores import ErrorConflicto
from dominio.grupo import GrupoCompartido
from dominio.invitacion_grupo import (
    InvitacionGrupo,
    EstadoInvitacionGrupo,
)


@pytest.fixture
def grupo():
    return GrupoCompartido(
        id="grupo-1",
        nombre_grupo="Viaje",
        moneda="USD",
        id_miembros=("usuario-1",),
        id_administrador="usuario-1",
        descripcion="Vacaciones",
    )


@pytest.fixture
def invitacion():
    return InvitacionGrupo(
        id="invitacion-1",
        grupo_id="grupo-1",
        invitado_id="usuario-2",
        invitador_id="usuario-1",
    )


@pytest.fixture
def repositorio_grupos(grupo, invitacion):
    repo = Mock()
    repo.obtener_por_id.return_value = grupo
    repo.listar_invitaciones_de_grupo.return_value = [invitacion]
    return repo


@pytest.fixture
def caso_uso(repositorio_grupos):
    return ResponderInvitacionGrupo(repositorio_grupos)


def test_aceptar_invitacion(caso_uso, repositorio_grupos):
    resultado = caso_uso.ejecutar(
        "usuario-2", "grupo-1", True
    )

    assert resultado.id == "invitacion-1"
    assert resultado.grupo_id == "grupo-1"
    assert resultado.estado == "ACEPTADA"

    guardada = (
        repositorio_grupos.actualizar_respuesta_invitacion
        .call_args.args[0]
    )

    assert guardada.estado == EstadoInvitacionGrupo.ACEPTADA
    assert guardada.invitado_id == "usuario-2"
    repositorio_grupos.actualizar_respuesta_invitacion.assert_called_once()


def test_rechazar_invitacion(caso_uso, repositorio_grupos):
    resultado = caso_uso.ejecutar(
        "usuario-2", "grupo-1", False
    )

    assert resultado.estado == "RECHAZADA"

    guardada = (
        repositorio_grupos.actualizar_respuesta_invitacion
        .call_args.args[0]
    )

    assert guardada.estado == EstadoInvitacionGrupo.RECHAZADA


def test_usuario_no_invitado(
    caso_uso, repositorio_grupos
):
    with pytest.raises(GrupoNoEncontrado):
        caso_uso.ejecutar("usuario-3", "grupo-1", True)

    repositorio_grupos.actualizar_respuesta_invitacion.assert_not_called()


def test_grupo_inexistente(
    caso_uso, repositorio_grupos
):
    repositorio_grupos.obtener_por_id.return_value = None

    with pytest.raises(GrupoNoEncontrado):
        caso_uso.ejecutar("usuario-2", "grupo-1", True)

    repositorio_grupos.actualizar_respuesta_invitacion.assert_not_called()


@pytest.mark.parametrize(
    "estado",
    [
        EstadoInvitacionGrupo.ACEPTADA,
        EstadoInvitacionGrupo.RECHAZADA,
    ],
)
def test_invitacion_ya_respondida(
    caso_uso, repositorio_grupos, invitacion, estado
):
    repositorio_grupos.listar_invitaciones_de_grupo.return_value = [
        replace(invitacion, estado=estado)
    ]

    with pytest.raises(ErrorConflicto):
        caso_uso.ejecutar("usuario-2", "grupo-1", True)

    repositorio_grupos.actualizar_respuesta_invitacion.assert_not_called()


def test_usuario_ya_es_miembro(
    caso_uso, repositorio_grupos, grupo
):
    repositorio_grupos.obtener_por_id.return_value = replace(
        grupo,
        id_miembros=("usuario-1", "usuario-2"),
    )

    with pytest.raises(ErrorConflicto):
        caso_uso.ejecutar("usuario-2", "grupo-1", True)

    repositorio_grupos.actualizar_respuesta_invitacion.assert_not_called()


def test_no_confunde_invitaciones_de_otros_grupos(
    caso_uso, repositorio_grupos, invitacion
):
    repositorio_grupos.listar_invitaciones_de_grupo.return_value = [
        replace(invitacion, grupo_id="grupo-2")
    ]

    with pytest.raises(GrupoNoEncontrado):
        caso_uso.ejecutar("usuario-2", "grupo-1", True)


def test_invitacion_rechazada_anterior_no_bloquea(
    caso_uso, repositorio_grupos, invitacion
):
    anterior = replace(
        invitacion,
        id="invitacion-anterior",
        estado=EstadoInvitacionGrupo.RECHAZADA,
    )

    repositorio_grupos.listar_invitaciones_de_grupo.return_value = [
        anterior,
        invitacion,
    ]

    resultado = caso_uso.ejecutar(
        "usuario-2", "grupo-1", True
    )

    assert resultado.id == "invitacion-1"
    assert resultado.estado == "ACEPTADA"


def test_no_modifica_entidades_originales(
    caso_uso, grupo, invitacion
):
    caso_uso.ejecutar("usuario-2", "grupo-1", True)

    assert invitacion.estado == EstadoInvitacionGrupo.PENDIENTE
    assert grupo.id_miembros == ("usuario-1",)


def test_propaga_conflicto_de_persistencia(
    caso_uso, repositorio_grupos
):
    repositorio_grupos.actualizar_respuesta_invitacion.side_effect = (
        ErrorConflicto(
            "invitacion",
            "La invitación ya fue respondida.",
        )
    )

    with pytest.raises(ErrorConflicto):
        caso_uso.ejecutar("usuario-2", "grupo-1", True)
