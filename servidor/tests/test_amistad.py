import pytest

from dominio.amistad import (
    Amistad,
    EstadoAmistad,
    responder_solicitud,
    validar_solicitud_amistad,
)
from dominio.errores import ErrorConflicto, ErrorDominio, ErrorPermiso


def test_auto_invitacion():
    with pytest.raises(ErrorDominio) as exc:
        validar_solicitud_amistad("ana", "ana", [])
    assert exc.value.campo == "receptor"
    assert not isinstance(exc.value, (ErrorConflicto, ErrorPermiso))


def test_invitacion_valida():
    validar_solicitud_amistad("ana", "beto", [])


def test_pendiente_duplicada_en_el_mismo_sentido():
    with pytest.raises(ErrorConflicto):
        validar_solicitud_amistad("ana", "beto", [Amistad("ana", "beto")])


def test_pendiente_duplicada_en_sentido_contrario():
    with pytest.raises(ErrorConflicto):
        validar_solicitud_amistad("beto", "ana", [Amistad("ana", "beto")])


def test_ya_son_amigos():
    activa = Amistad("ana", "beto", EstadoAmistad.ACTIVA)
    with pytest.raises(ErrorConflicto):
        validar_solicitud_amistad("beto", "ana", [activa])


def test_rechazada_no_bloquea_reinvitar():
    validar_solicitud_amistad("ana", "beto", [Amistad("ana", "beto", EstadoAmistad.RECHAZADA)])


def test_otras_personas_no_interfieren():
    validar_solicitud_amistad("ana", "beto", [Amistad("ana", "carla"), Amistad("beto", "carla")])


def test_el_destinatario_acepta():
    r = responder_solicitud(Amistad("ana", "beto"), "beto", True)
    assert r.estado == EstadoAmistad.ACTIVA


def test_el_destinatario_rechaza():
    r = responder_solicitud(Amistad("ana", "beto"), "beto", False)
    assert r.estado == EstadoAmistad.RECHAZADA


def test_el_emisor_no_puede_responder():
    with pytest.raises(ErrorPermiso):
        responder_solicitud(Amistad("ana", "beto"), "ana", True)


def test_no_se_responde_dos_veces():
    with pytest.raises(ErrorConflicto):
        responder_solicitud(Amistad("ana", "beto", EstadoAmistad.ACTIVA), "beto", True)


def test_responder_no_modifica_la_original():
    original = Amistad("ana", "beto")
    responder_solicitud(original, "beto", True)
    assert original.estado == EstadoAmistad.PENDIENTE