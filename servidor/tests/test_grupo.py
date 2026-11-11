import pytest

from dominio.amistad import Amistad, EstadoAmistad
from dominio.errores import ErrorConflicto, ErrorDominio, ErrorPermiso
from dominio.grupo import GrupoCompartido, es_miembro, validar_es_admin
from dominio.invitacion_grupo import (
    EstadoInvitacionGrupo,
    InvitacionGrupo,
    responder_invitacion_grupo,
    validar_invitacion_grupo,
)


def grupo(miembros=("ana", "beto"), admin="ana"):
    return GrupoCompartido("Viaje", "USD", miembros, admin)


def campo_grupo(*args):
    with pytest.raises(ErrorDominio) as exc:
        GrupoCompartido(*args)
    return exc.value.campo


# ---------- GrupoCompartido ----------

def test_grupo_valido():
    assert grupo().administrador == "ana"


def test_nombre_solo_espacios():
    assert campo_grupo("   ", "USD", ("ana",), "ana") == "nombre_grupo"


@pytest.mark.parametrize("moneda", ["usd", "US", "USDT", "U1D", ""])
def test_moneda_invalida(moneda):
    assert campo_grupo("Viaje", moneda, ("ana",), "ana") == "moneda"


def test_sin_miembros():
    assert campo_grupo("Viaje", "USD", (), "ana") == "miembros"


def test_miembros_duplicados():
    assert campo_grupo("Viaje", "USD", ("ana", "ana"), "ana") == "miembros"


def test_administrador_debe_ser_miembro():
    assert campo_grupo("Viaje", "USD", ("ana", "beto"), "carla") == "administrador"


# ---------- pertenencia (INV-17) ----------

def test_es_miembro():
    g = grupo()
    assert es_miembro(g, "beto") and not es_miembro(g, "carla")


def test_solo_el_admin_pasa_validar_es_admin():
    validar_es_admin(grupo(), "ana")
    with pytest.raises(ErrorPermiso):
        validar_es_admin(grupo(), "beto")


# ---------- invitación a grupo ----------

AMIGOS = [Amistad("ana", "beto", EstadoAmistad.ACTIVA)]


def test_invitacion_valida():
    validar_invitacion_grupo(grupo(("ana",)), "ana", "beto", AMIGOS, [])


def test_solo_el_admin_invita():
    with pytest.raises(ErrorPermiso):
        validar_invitacion_grupo(grupo(("ana", "beto")), "beto", "carla", AMIGOS, [])


def test_solo_se_invita_a_amigos_confirmados():
    with pytest.raises(ErrorPermiso):
        validar_invitacion_grupo(grupo(("ana",)), "ana", "carla", AMIGOS, [])
    pendiente = [Amistad("ana", "carla", EstadoAmistad.PENDIENTE)]
    with pytest.raises(ErrorPermiso):
        validar_invitacion_grupo(grupo(("ana",)), "ana", "carla", pendiente, [])


def test_no_se_invita_a_quien_ya_es_miembro():
    with pytest.raises(ErrorConflicto):
        validar_invitacion_grupo(grupo(("ana", "beto")), "ana", "beto", AMIGOS, [])


def test_invitacion_pendiente_duplicada():
    pend = [InvitacionGrupo("Viaje", "beto", "ana")]
    with pytest.raises(ErrorConflicto):
        validar_invitacion_grupo(grupo(("ana",)), "ana", "beto", AMIGOS, pend)


def test_invitacion_rechazada_no_bloquea_reinvitar():
    rech = [InvitacionGrupo("Viaje", "beto", "ana", EstadoInvitacionGrupo.RECHAZADA)]
    validar_invitacion_grupo(grupo(("ana",)), "ana", "beto", AMIGOS, rech)


def test_el_invitado_acepta_o_rechaza():
    inv = InvitacionGrupo("Viaje", "beto", "ana")
    assert responder_invitacion_grupo(inv, "beto", True).estado == EstadoInvitacionGrupo.ACEPTADA
    assert responder_invitacion_grupo(inv, "beto", False).estado == EstadoInvitacionGrupo.RECHAZADA


def test_solo_el_invitado_responde():
    with pytest.raises(ErrorPermiso):
        responder_invitacion_grupo(InvitacionGrupo("Viaje", "beto", "ana"), "ana", True)


def test_invitacion_no_se_responde_dos_veces():
    inv = InvitacionGrupo("Viaje", "beto", "ana", EstadoInvitacionGrupo.ACEPTADA)
    with pytest.raises(ErrorConflicto):
        responder_invitacion_grupo(inv, "beto", True)