from unittest.mock import Mock
from uuid import UUID

import pytest

from aplicacion.crear_grupo import CrearGrupo
from aplicacion.errores import UsuarioNoEncontrado
from dominio.errores import ErrorDominio
from dominio.grupo import GrupoCompartido


@pytest.fixture
def repositorios(usuario_existente):
    repo_usuarios = Mock()
    repo_usuarios.obtener_usuario_por_id.return_value = (
        usuario_existente
    )

    repo_grupos = Mock()

    return repo_usuarios, repo_grupos


@pytest.fixture
def caso_uso(repositorios):
    repo_usuarios, repo_grupos = repositorios
    return CrearGrupo(repo_usuarios, repo_grupos)


def test_crear_grupo_correctamente(caso_uso, repositorios):
    _, repo_grupos = repositorios

    resultado = caso_uso.ejecutar(
        usuario_autenticado_id="usuario-123",
        nombre_grupo="Viaje",
        moneda="USD",
        descripcion="Vacaciones",
    )

    assert resultado.nombre_grupo == "Viaje"
    assert resultado.moneda == "USD"
    assert resultado.descripcion == "Vacaciones"
    assert resultado.administrador_id == "usuario-123"

    repo_grupos.guardar_grupo.assert_called_once()


def test_creador_es_administrador_y_primer_miembro(
    caso_uso, repositorios
):
    _, repo_grupos = repositorios

    caso_uso.ejecutar(
        "usuario-123", "Viaje", "USD"
    )

    grupo = repo_grupos.guardar_grupo.call_args.args[0]

    assert isinstance(grupo, GrupoCompartido)
    assert grupo.id_administrador == "usuario-123"
    assert grupo.id_miembros == ("usuario-123",)


def test_grupo_tiene_identificador_unico(caso_uso):
    primero = caso_uso.ejecutar(
        "usuario-123", "Viaje", "USD"
    )

    segundo = caso_uso.ejecutar(
        "usuario-123", "Viaje", "USD"
    )

    assert primero.id != segundo.id

    # Verificar que ambos IDs sean UUID válidos.
    UUID(primero.id)
    UUID(segundo.id)


def test_grupo_sin_descripcion(caso_uso):
    resultado = caso_uso.ejecutar(
        "usuario-123", "Viaje", "USD"
    )

    assert resultado.descripcion == ""


def test_normaliza_nombre_del_grupo(caso_uso, repositorios):
    _, repo_grupos = repositorios

    resultado = caso_uso.ejecutar(
        "usuario-123", "  Viaje  ", "USD"
    )

    assert resultado.nombre_grupo == "Viaje"

    grupo = repo_grupos.guardar_grupo.call_args.args[0]
    assert grupo.nombre_grupo == "Viaje"


def test_usuario_inexistente(caso_uso, repositorios):
    repo_usuarios, repo_grupos = repositorios
    repo_usuarios.obtener_usuario_por_id.return_value = None

    with pytest.raises(UsuarioNoEncontrado):
        caso_uso.ejecutar(
            "usuario-inexistente", "Viaje", "USD"
        )

    repo_grupos.guardar_grupo.assert_not_called()


def test_nombre_grupo_invalido(caso_uso, repositorios):
    _, repo_grupos = repositorios

    with pytest.raises(ErrorDominio) as exc:
        caso_uso.ejecutar(
            "usuario-123", "   ", "USD"
        )

    assert exc.value.campo == "nombre_grupo"
    repo_grupos.guardar_grupo.assert_not_called()


@pytest.mark.parametrize(
    "moneda",
    ["usd", "US", "USDT", "U1D", ""],
)
def test_moneda_invalida(caso_uso, repositorios, moneda):
    _, repo_grupos = repositorios

    with pytest.raises(ErrorDominio) as exc:
        caso_uso.ejecutar(
            "usuario-123", "Viaje", moneda
        )

    assert exc.value.campo == "moneda"
    repo_grupos.guardar_grupo.assert_not_called()


def test_consulta_usuario_autenticado(caso_uso, repositorios):
    repo_usuarios, _ = repositorios

    caso_uso.ejecutar(
        "usuario-123", "Viaje", "USD"
    )

    repo_usuarios.obtener_usuario_por_id.assert_called_once_with(
        "usuario-123"
    )


def test_devuelve_el_id_del_grupo_guardado(
    caso_uso, repositorios
):
    _, repo_grupos = repositorios

    resultado = caso_uso.ejecutar(
        "usuario-123", "Viaje", "USD"
    )

    grupo_guardado = repo_grupos.guardar_grupo.call_args.args[0]

    assert resultado.id == grupo_guardado.id
