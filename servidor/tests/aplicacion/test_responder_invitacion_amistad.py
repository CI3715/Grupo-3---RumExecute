
from unittest.mock import Mock

import pytest

from aplicacion.errores import AmistadNoEncontrada
from aplicacion.responder_invitacion_amistad import ResponderInvitacionAmistad
from dominio.amistad import Amistad, EstadoAmistad
from dominio.errores import ErrorConflicto, ErrorPermiso


@pytest.fixture
def invitacion():
    return Amistad(
        id="amistad-1",
        id_solicitante="usuario-1",
        id_receptor="usuario-2",
    )


@pytest.fixture
def repositorio_amistades(invitacion):
    repositorio = Mock()
    repositorio.obtener_por_id.return_value = invitacion
    return repositorio


@pytest.fixture
def caso_uso(repositorio_amistades):
    return ResponderInvitacionAmistad(
        repositorio_amistades
    )


def test_aceptar_invitacion(
    caso_uso,
    repositorio_amistades,
):
    respuesta = caso_uso.ejecutar(
        "usuario-2", "amistad-1", True
    )

    assert respuesta.id == "amistad-1"
    assert respuesta.estado == "ACTIVA"

    amistad_guardada = (
        repositorio_amistades.actualizar_respuesta
        .call_args.args[0]
    )

    assert amistad_guardada.estado == EstadoAmistad.ACTIVA
    assert amistad_guardada.id_solicitante == "usuario-1"
    assert amistad_guardada.id_receptor == "usuario-2"


def test_rechazar_invitacion(
    caso_uso,
    repositorio_amistades,
):
    respuesta = caso_uso.ejecutar(
        "usuario-2", "amistad-1", False
    )

    assert respuesta.estado == "RECHAZADA"

    amistad_guardada = (
        repositorio_amistades.actualizar_respuesta
        .call_args.args[0]
    )

    assert amistad_guardada.estado == EstadoAmistad.RECHAZADA


def test_invitacion_no_encontrada(
    caso_uso,
    repositorio_amistades,
):
    repositorio_amistades.obtener_por_id.return_value = None

    with pytest.raises(AmistadNoEncontrada):
        caso_uso.ejecutar(
            "usuario-2", "inexistente", True
        )

    repositorio_amistades.actualizar_respuesta.assert_not_called()


def test_solo_el_receptor_puede_responder(
    caso_uso,
    repositorio_amistades,
):
    with pytest.raises(ErrorPermiso):
        caso_uso.ejecutar(
            "usuario-1", "amistad-1", True
        )

    repositorio_amistades.actualizar_respuesta.assert_not_called()


def test_un_tercero_no_puede_responder(
    caso_uso,
    repositorio_amistades,
):
    with pytest.raises(ErrorPermiso):
        caso_uso.ejecutar(
            "usuario-3", "amistad-1", True
        )

    repositorio_amistades.actualizar_respuesta.assert_not_called()


@pytest.mark.parametrize(
    "estado",
    [
        EstadoAmistad.ACTIVA,
        EstadoAmistad.RECHAZADA,
    ],
)
def test_no_se_puede_responder_dos_veces(
    caso_uso,
    repositorio_amistades,
    invitacion,
    estado,
):
    repositorio_amistades.obtener_por_id.return_value = (
        Amistad(
            id=invitacion.id,
            id_solicitante=invitacion.id_solicitante,
            id_receptor=invitacion.id_receptor,
            estado=estado,
        )
    )

    with pytest.raises(ErrorConflicto):
        caso_uso.ejecutar(
            "usuario-2", "amistad-1", True
        )

    repositorio_amistades.actualizar_respuesta.assert_not_called()


def test_consulta_invitacion_por_id(
    caso_uso,
    repositorio_amistades,
):
    caso_uso.ejecutar(
        "usuario-2", "amistad-1", True
    )

    repositorio_amistades.obtener_por_id.assert_called_once_with(
        "amistad-1"
    )


def test_no_modifica_la_invitacion_original(
    caso_uso,
    invitacion,
):
    caso_uso.ejecutar(
        "usuario-2", "amistad-1", True
    )

    assert invitacion.estado == EstadoAmistad.PENDIENTE


def test_propaga_conflicto_de_persistencia(
    caso_uso,
    repositorio_amistades,
):
    repositorio_amistades.actualizar_respuesta.side_effect = (
        ErrorConflicto(
            "amistad",
            "La invitación ya fue respondida.",
        )
    )

    with pytest.raises(ErrorConflicto):
        caso_uso.ejecutar(
            "usuario-2", "amistad-1", True
        )
