
from dataclasses import replace
from unittest.mock import Mock

import pytest

from aplicacion.errores import UsuarioNoEncontrado
from aplicacion.listar_amistades import ListarAmistades
from dominio.amistad import Amistad, EstadoAmistad


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
    return {u.id: u for u in (ana, beto, carla)}


@pytest.fixture
def repositorios(usuarios):
    repo_usuarios = Mock()
    repo_usuarios.obtener_usuario_por_id.side_effect = usuarios.get

    repo_amistades = Mock()
    repo_amistades.listar_de_usuario.return_value = []

    return repo_usuarios, repo_amistades


@pytest.fixture
def caso_uso(repositorios):
    repo_usuarios, repo_amistades = repositorios
    return ListarAmistades(repo_usuarios, repo_amistades)


def test_usuario_sin_amistades(caso_uso):
    resultado = caso_uso.ejecutar("usuario-1")

    assert resultado.amigos == []
    assert resultado.invitaciones_recibidas == []
    assert resultado.invitaciones_enviadas == []


def test_lista_amistades_activas(caso_uso, repositorios):
    _, repo_amistades = repositorios
    repo_amistades.listar_de_usuario.return_value = [
        Amistad(
            "amistad-1", "usuario-1", "usuario-2",
            EstadoAmistad.ACTIVA,
        )
    ]

    resultado = caso_uso.ejecutar("usuario-1")

    assert len(resultado.amigos) == 1
    assert resultado.amigos[0].amistad_id == "amistad-1"
    assert resultado.amigos[0].nombre_usuario == "beto"


def test_amistad_activa_en_sentido_inverso(caso_uso, repositorios):
    _, repo_amistades = repositorios
    repo_amistades.listar_de_usuario.return_value = [
        Amistad(
            "amistad-1", "usuario-2", "usuario-1",
            EstadoAmistad.ACTIVA,
        )
    ]

    resultado = caso_uso.ejecutar("usuario-1")

    assert resultado.amigos[0].nombre_usuario == "beto"


def test_clasifica_invitaciones_pendientes(caso_uso, repositorios):
    _, repo_amistades = repositorios
    repo_amistades.listar_de_usuario.return_value = [
        Amistad("amistad-1", "usuario-2", "usuario-1"),
        Amistad("amistad-2", "usuario-1", "usuario-3"),
    ]

    resultado = caso_uso.ejecutar("usuario-1")

    assert len(resultado.invitaciones_recibidas) == 1
    assert resultado.invitaciones_recibidas[0].nombre_usuario == "beto"
    assert resultado.invitaciones_recibidas[0].amistad_id == "amistad-1"

    assert len(resultado.invitaciones_enviadas) == 1
    assert resultado.invitaciones_enviadas[0].nombre_usuario == "carla"
    assert resultado.invitaciones_enviadas[0].amistad_id == "amistad-2"


def test_excluye_invitaciones_rechazadas(caso_uso, repositorios):
    _, repo_amistades = repositorios
    repo_amistades.listar_de_usuario.return_value = [
        Amistad(
            "amistad-1", "usuario-1", "usuario-2",
            EstadoAmistad.RECHAZADA,
        )
    ]

    resultado = caso_uso.ejecutar("usuario-1")

    assert resultado.amigos == []
    assert resultado.invitaciones_recibidas == []
    assert resultado.invitaciones_enviadas == []


def test_no_expone_relaciones_ajenas(caso_uso, repositorios):
    _, repo_amistades = repositorios
    repo_amistades.listar_de_usuario.return_value = [
        Amistad(
            "amistad-1", "usuario-2", "usuario-3",
            EstadoAmistad.ACTIVA,
        )
    ]

    resultado = caso_uso.ejecutar("usuario-1")

    assert resultado.amigos == []


def test_usuario_autenticado_inexistente(caso_uso, repositorios):
    _, repo_amistades = repositorios

    with pytest.raises(UsuarioNoEncontrado):
        caso_uso.ejecutar("usuario-inexistente")

    repo_amistades.listar_de_usuario.assert_not_called()


def test_no_devuelve_correos(caso_uso, repositorios):
    _, repo_amistades = repositorios
    repo_amistades.listar_de_usuario.return_value = [
        Amistad(
            "amistad-1", "usuario-1", "usuario-2",
            EstadoAmistad.ACTIVA,
        )
    ]

    resultado = caso_uso.ejecutar("usuario-1")

    assert not hasattr(resultado.amigos[0], "correo")
