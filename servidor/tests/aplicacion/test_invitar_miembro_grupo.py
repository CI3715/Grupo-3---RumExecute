from dataclasses import replace
from unittest.mock import Mock
from uuid import UUID

import pytest

from aplicacion.invitar_miembro_grupo import InvitarMiembroGrupo
from aplicacion.errores import GrupoNoEncontrado, UsuarioNoEncontrado

from dominio.amistad import Amistad, EstadoAmistad
from dominio.errores import ErrorConflicto, ErrorPermiso
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
        id_miembros=("usuario-123",),
        id_administrador="usuario-123",
        descripcion="Vacaciones",
    )


@pytest.fixture
def repositorios(usuario_existente, grupo):
    invitado = replace(
        usuario_existente,
        id="usuario-456",
        nombre_usuario="beto",
        nombre_visible="Beto",
        correo="beto@example.com",
    )

    repo_usuarios = Mock()
    repo_usuarios.obtener_usuario_por_nombre_usuario.return_value = (
        invitado
    )

    repo_grupos = Mock()
    repo_grupos.obtener_por_id.return_value = grupo
    repo_grupos.listar_invitaciones_de_grupo.return_value = []

    repo_amistades = Mock()
    repo_amistades.listar_entre_usuarios.return_value = [
        Amistad(
            id="amistad-1",
            id_solicitante="usuario-123",
            id_receptor="usuario-456",
            estado=EstadoAmistad.ACTIVA,
        )
    ]

    return repo_usuarios, repo_grupos, repo_amistades


@pytest.fixture
def caso_uso(repositorios):
    return InvitarMiembroGrupo(*repositorios)


def test_invitar_amigo_correctamente(caso_uso, repositorios):
    _, repo_grupos, _ = repositorios

    resultado = caso_uso.ejecutar(
        "usuario-123", "grupo-1", "beto"
    )

    assert resultado.grupo_id == "grupo-1"
    assert resultado.invitado_id == "usuario-456"
    assert resultado.invitador_id == "usuario-123"
    assert resultado.estado == "PENDIENTE"

    UUID(resultado.id)
    repo_grupos.guardar_invitacion.assert_called_once()


def test_guarda_invitacion_pendiente(caso_uso, repositorios):
    _, repo_grupos, _ = repositorios

    caso_uso.ejecutar("usuario-123", "grupo-1", "beto")

    invitacion = repo_grupos.guardar_invitacion.call_args.args[0]

    assert isinstance(invitacion, InvitacionGrupo)
    assert invitacion.estado == EstadoInvitacionGrupo.PENDIENTE
    assert invitacion.invitado_id == "usuario-456"


def test_no_agrega_al_invitado_automaticamente(
    caso_uso, grupo
):
    caso_uso.ejecutar("usuario-123", "grupo-1", "beto")

    assert grupo.id_miembros == ("usuario-123",)
    assert "usuario-456" not in grupo.id_miembros


def test_grupo_inexistente(caso_uso, repositorios):
    _, repo_grupos, _ = repositorios
    repo_grupos.obtener_por_id.return_value = None

    with pytest.raises(GrupoNoEncontrado):
        caso_uso.ejecutar("usuario-123", "grupo-1", "beto")

    repo_grupos.guardar_invitacion.assert_not_called()


def test_usuario_ajeno_no_puede_acceder_al_grupo(
    caso_uso, repositorios
):
    _, repo_grupos, _ = repositorios

    with pytest.raises(GrupoNoEncontrado):
        caso_uso.ejecutar("usuario-999", "grupo-1", "beto")

    repo_grupos.guardar_invitacion.assert_not_called()


def test_solo_administrador_puede_invitar(
    caso_uso, repositorios, grupo
):
    _, repo_grupos, _ = repositorios

    repo_grupos.obtener_por_id.return_value = replace(
        grupo,
        id_miembros=("usuario-123", "usuario-789"),
    )

    with pytest.raises(ErrorPermiso):
        caso_uso.ejecutar("usuario-789", "grupo-1", "beto")

    repo_grupos.guardar_invitacion.assert_not_called()


def test_invitado_inexistente(caso_uso, repositorios):
    repo_usuarios, repo_grupos, _ = repositorios
    repo_usuarios.obtener_usuario_por_nombre_usuario.return_value = None

    with pytest.raises(UsuarioNoEncontrado):
        caso_uso.ejecutar("usuario-123", "grupo-1", "nadie")

    repo_grupos.guardar_invitacion.assert_not_called()


def test_no_se_puede_invitar_a_un_no_amigo(
    caso_uso, repositorios
):
    _, repo_grupos, repo_amistades = repositorios
    repo_amistades.listar_entre_usuarios.return_value = []

    with pytest.raises(ErrorPermiso):
        caso_uso.ejecutar("usuario-123", "grupo-1", "beto")

    repo_grupos.guardar_invitacion.assert_not_called()


def test_amistad_pendiente_no_es_suficiente(
    caso_uso, repositorios
):
    _, repo_grupos, repo_amistades = repositorios

    repo_amistades.listar_entre_usuarios.return_value = [
        Amistad(
            "amistad-1",
            "usuario-123",
            "usuario-456",
            EstadoAmistad.PENDIENTE,
        )
    ]

    with pytest.raises(ErrorPermiso):
        caso_uso.ejecutar("usuario-123", "grupo-1", "beto")

    repo_grupos.guardar_invitacion.assert_not_called()


def test_no_invita_a_un_miembro_existente(
    caso_uso, repositorios, grupo
):
    _, repo_grupos, _ = repositorios
    repo_grupos.obtener_por_id.return_value = replace(
        grupo,
        id_miembros=("usuario-123", "usuario-456"),
    )

    with pytest.raises(ErrorConflicto):
        caso_uso.ejecutar("usuario-123", "grupo-1", "beto")

    repo_grupos.guardar_invitacion.assert_not_called()


def test_no_permite_invitaciones_pendientes_duplicadas(
    caso_uso, repositorios
):
    _, repo_grupos, _ = repositorios

    repo_grupos.listar_invitaciones_de_grupo.return_value = [
        InvitacionGrupo(
            id="invitacion-anterior",
            grupo_id="grupo-1",
            invitado_id="usuario-456",
            invitador_id="usuario-123",
        )
    ]

    with pytest.raises(ErrorConflicto):
        caso_uso.ejecutar("usuario-123", "grupo-1", "beto")

    repo_grupos.guardar_invitacion.assert_not_called()


def test_invitacion_rechazada_permite_reinvitar(
    caso_uso, repositorios
):
    _, repo_grupos, _ = repositorios

    repo_grupos.listar_invitaciones_de_grupo.return_value = [
        InvitacionGrupo(
            id="invitacion-anterior",
            grupo_id="grupo-1",
            invitado_id="usuario-456",
            invitador_id="usuario-123",
            estado=EstadoInvitacionGrupo.RECHAZADA,
        )
    ]

    resultado = caso_uso.ejecutar(
        "usuario-123", "grupo-1", "beto"
    )

    assert resultado.estado == "PENDIENTE"
    repo_grupos.guardar_invitacion.assert_called_once()


def test_busca_al_invitado_por_nombre_exacto(
    caso_uso, repositorios
):
    repo_usuarios, _, _ = repositorios

    caso_uso.ejecutar("usuario-123", "grupo-1", "beto")

    repo_usuarios.obtener_usuario_por_nombre_usuario.assert_called_once_with(
        "beto"
    )


def test_consulta_amistades_por_ids(
    caso_uso, repositorios
):
    _, _, repo_amistades = repositorios

    caso_uso.ejecutar("usuario-123", "grupo-1", "beto")

    repo_amistades.listar_entre_usuarios.assert_called_once_with(
        "usuario-123",
        "usuario-456",
    )
