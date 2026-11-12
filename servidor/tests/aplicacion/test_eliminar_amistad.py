from unittest.mock import Mock

import pytest

from aplicacion.eliminar_amistad import EliminarAmistad
from aplicacion.errores import AmistadNoEncontrada
from dominio.amistad import Amistad, EstadoAmistad
from dominio.errores import ErrorConflicto, ErrorPermiso


@pytest.fixture
def amistad():
    return Amistad(
        id="amistad-1",
        id_solicitante="usuario-1",
        id_receptor="usuario-2",
        estado=EstadoAmistad.ACTIVA,
    )


@pytest.fixture
def repositorio_amistades(amistad):
    repositorio = Mock()
    repositorio.obtener_por_id.return_value = amistad
    repositorio.eliminar_amistad.return_value = True
    return repositorio


@pytest.fixture
def caso_uso(repositorio_amistades):
    return EliminarAmistad(repositorio_amistades)


def test_solicitante_puede_eliminar_amistad(
    caso_uso, repositorio_amistades
):
    resultado = caso_uso.ejecutar(
        "usuario-1", "amistad-1"
    )

    assert resultado is None
    repositorio_amistades.eliminar_amistad.assert_called_once_with(
        "amistad-1", "usuario-1"
    )


def test_receptor_puede_eliminar_amistad(
    caso_uso, repositorio_amistades
):
    caso_uso.ejecutar("usuario-2", "amistad-1")

    repositorio_amistades.eliminar_amistad.assert_called_once_with(
        "amistad-1", "usuario-2"
    )


def test_amistad_no_encontrada(
    caso_uso, repositorio_amistades
):
    repositorio_amistades.obtener_por_id.return_value = None

    with pytest.raises(AmistadNoEncontrada):
        caso_uso.ejecutar("usuario-1", "inexistente")

    repositorio_amistades.eliminar_amistad.assert_not_called()


def test_tercero_no_puede_eliminar_amistad(
    caso_uso, repositorio_amistades
):
    with pytest.raises(ErrorPermiso):
        caso_uso.ejecutar("usuario-3", "amistad-1")

    repositorio_amistades.eliminar_amistad.assert_not_called()


@pytest.mark.parametrize(
    "estado",
    [
        EstadoAmistad.PENDIENTE,
        EstadoAmistad.RECHAZADA,
    ],
)
def test_no_se_pueden_eliminar_amistades_no_activas(
    caso_uso, repositorio_amistades, amistad, estado
):
    repositorio_amistades.obtener_por_id.return_value = Amistad(
        id=amistad.id,
        id_solicitante=amistad.id_solicitante,
        id_receptor=amistad.id_receptor,
        estado=estado,
    )

    with pytest.raises(ErrorConflicto):
        caso_uso.ejecutar("usuario-1", "amistad-1")

    repositorio_amistades.eliminar_amistad.assert_not_called()


def test_repositorio_no_logra_eliminar(
    caso_uso, repositorio_amistades
):
    repositorio_amistades.eliminar_amistad.return_value = False

    with pytest.raises(AmistadNoEncontrada):
        caso_uso.ejecutar("usuario-1", "amistad-1")


def test_consulta_amistad_por_id(
    caso_uso, repositorio_amistades
):
    caso_uso.ejecutar("usuario-1", "amistad-1")

    repositorio_amistades.obtener_por_id.assert_called_once_with(
        "amistad-1"
    )


def test_no_modifica_entidad_original(
    caso_uso, amistad
):
    caso_uso.ejecutar("usuario-1", "amistad-1")

    assert amistad.estado == EstadoAmistad.ACTIVA
