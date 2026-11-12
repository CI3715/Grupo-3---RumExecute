from unittest.mock import Mock

import pytest

from aplicacion.errores import UsuarioNoEncontrado
from aplicacion.listar_mis_grupos import ListarMisGrupos
from dominio.grupo import GrupoCompartido
from dominio.invitacion_grupo import (
    InvitacionGrupo,
    EstadoInvitacionGrupo,
)


@pytest.fixture
def repositorios(usuario_existente):
    repo_usuarios = Mock()
    repo_usuarios.obtener_usuario_por_id.return_value = usuario_existente

    repo_grupos = Mock()
    repo_grupos.listar_de_usuario.return_value = []
    repo_grupos.listar_invitaciones_recibidas.return_value = []

    return repo_usuarios, repo_grupos


@pytest.fixture
def caso_uso(repositorios):
    repo_usuarios, repo_grupos = repositorios
    return ListarMisGrupos(repo_usuarios, repo_grupos)


def grupo(
    grupo_id="grupo-1",
    miembros=("usuario-123",),
    administrador="usuario-123",
):
    return GrupoCompartido(
        id=grupo_id,
        nombre_grupo="Viaje",
        moneda="USD",
        id_miembros=miembros,
        id_administrador=administrador,
        descripcion="Vacaciones",
    )


def invitacion(
    estado=EstadoInvitacionGrupo.PENDIENTE,
    invitado="usuario-123",
):
    return InvitacionGrupo(
        id="invitacion-1",
        grupo_id="grupo-2",
        invitado_id=invitado,
        invitador_id="usuario-456",
        estado=estado,
    )


def test_usuario_sin_grupos(caso_uso):
    resultado = caso_uso.ejecutar("usuario-123")

    assert resultado.grupos == []
    assert resultado.invitaciones_recibidas == []


def test_lista_grupos_del_usuario(caso_uso, repositorios):
    _, repo_grupos = repositorios
    repo_grupos.listar_de_usuario.return_value = [
        grupo()
    ]

    resultado = caso_uso.ejecutar("usuario-123")

    assert len(resultado.grupos) == 1
    assert resultado.grupos[0].id == "grupo-1"
    assert resultado.grupos[0].nombre_grupo == "Viaje"
    assert resultado.grupos[0].moneda == "USD"
    assert resultado.grupos[0].descripcion == "Vacaciones"


def test_usuario_puede_ser_miembro_sin_ser_admin(
    caso_uso, repositorios
):
    _, repo_grupos = repositorios
    repo_grupos.listar_de_usuario.return_value = [
        grupo(
            miembros=("usuario-456", "usuario-123"),
            administrador="usuario-456",
        )
    ]

    resultado = caso_uso.ejecutar("usuario-123")

    assert len(resultado.grupos) == 1
    assert resultado.grupos[0].administrador_id == "usuario-456"


def test_no_expone_grupos_ajenos(caso_uso, repositorios):
    _, repo_grupos = repositorios
    repo_grupos.listar_de_usuario.return_value = [
        grupo(
            miembros=("usuario-456",),
            administrador="usuario-456",
        )
    ]

    resultado = caso_uso.ejecutar("usuario-123")

    assert resultado.grupos == []


def test_lista_invitaciones_pendientes(
    caso_uso, repositorios
):
    _, repo_grupos = repositorios
    repo_grupos.listar_invitaciones_recibidas.return_value = [
        invitacion()
    ]

    resultado = caso_uso.ejecutar("usuario-123")

    assert len(resultado.invitaciones_recibidas) == 1
    assert resultado.invitaciones_recibidas[0].id == "invitacion-1"
    assert resultado.invitaciones_recibidas[0].grupo_id == "grupo-2"
    assert resultado.invitaciones_recibidas[0].estado == "PENDIENTE"


@pytest.mark.parametrize(
    "estado",
    [
        EstadoInvitacionGrupo.ACEPTADA,
        EstadoInvitacionGrupo.RECHAZADA,
    ],
)
def test_excluye_invitaciones_respondidas(
    caso_uso, repositorios, estado
):
    _, repo_grupos = repositorios
    repo_grupos.listar_invitaciones_recibidas.return_value = [
        invitacion(estado=estado)
    ]

    resultado = caso_uso.ejecutar("usuario-123")

    assert resultado.invitaciones_recibidas == []


def test_no_expone_invitaciones_ajenas(
    caso_uso, repositorios
):
    _, repo_grupos = repositorios
    repo_grupos.listar_invitaciones_recibidas.return_value = [
        invitacion(invitado="usuario-789")
    ]

    resultado = caso_uso.ejecutar("usuario-123")

    assert resultado.invitaciones_recibidas == []


def test_invitado_no_es_automaticamente_miembro(
    caso_uso, repositorios
):
    _, repo_grupos = repositorios
    repo_grupos.listar_invitaciones_recibidas.return_value = [
        invitacion()
    ]

    resultado = caso_uso.ejecutar("usuario-123")

    assert resultado.grupos == []
    assert len(resultado.invitaciones_recibidas) == 1


def test_usuario_inexistente(caso_uso, repositorios):
    repo_usuarios, repo_grupos = repositorios
    repo_usuarios.obtener_usuario_por_id.return_value = None

    with pytest.raises(UsuarioNoEncontrado):
        caso_uso.ejecutar("usuario-inexistente")

    repo_grupos.listar_de_usuario.assert_not_called()
    repo_grupos.listar_invitaciones_recibidas.assert_not_called()


def test_consulta_repositorios_con_id_autenticado(
    caso_uso, repositorios
):
    _, repo_grupos = repositorios

    caso_uso.ejecutar("usuario-123")

    repo_grupos.listar_de_usuario.assert_called_once_with(
        "usuario-123"
    )
    repo_grupos.listar_invitaciones_recibidas.assert_called_once_with(
        "usuario-123"
    )
