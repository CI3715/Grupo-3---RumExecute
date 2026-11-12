
import pytest

from dataclasses import replace
from dominio.amistad import Amistad, EstadoAmistad
from dominio.errores import ErrorConflicto, ErrorDominio

from aplicacion.errores import UsuarioNoEncontrado
from aplicacion.dtos.invitacion_amistad_respuesta import (
    InvitacionAmistadRespuesta,
)


@pytest.fixture
def usuarios_preparados(
    repositorio_usuarios,
    usuario_existente,
):
    receptor = replace(
        usuario_existente,
        id="usuario-456",
        nombre_usuario="beto",
        nombre_visible="Beto",
        correo="beto@example.com",
    )

    repositorio_usuarios.obtener_usuario_por_id.return_value = (
        usuario_existente
    )

    repositorio_usuarios.obtener_usuario_por_nombre_usuario.return_value = (
        receptor
    )

    return usuario_existente, receptor


# --------------------------------------------------
# CASO EXITOSO
# --------------------------------------------------

def test_envia_invitacion_exitosamente(
    caso_uso_enviar_invitacion,
    usuarios_preparados,
    repositorio_amistades,
):
    resultado = caso_uso_enviar_invitacion.ejecutar(
        usuario_autenticado_id="usuario-123",
        nombre_usuario_receptor="beto",
    )

    assert isinstance(resultado, InvitacionAmistadRespuesta)

    # El ID se genera dinámicamente mediante uuid4
    assert isinstance(resultado.id, str)
    assert resultado.id

    assert resultado.nombre_usuario_solicitante == "eduardo"
    assert resultado.nombre_usuario_receptor == "beto"
    assert resultado.estado == "PENDIENTE"

    repositorio_amistades.guardar_invitacion.assert_called_once()


def test_guarda_invitacion_pendiente(
    caso_uso_enviar_invitacion,
    usuarios_preparados,
    repositorio_amistades,
):
    resultado = caso_uso_enviar_invitacion.ejecutar(
        "usuario-123",
        "beto",
    )

    invitacion = (
        repositorio_amistades
        .guardar_invitacion.call_args.args[0]
    )

    assert isinstance(invitacion, Amistad)

    assert invitacion.id == resultado.id
    assert invitacion.id_solicitante == "usuario-123"
    assert invitacion.id_receptor == "usuario-456"
    assert invitacion.estado == EstadoAmistad.PENDIENTE


def test_obtiene_solicitante_desde_identidad_autenticada(
    caso_uso_enviar_invitacion,
    usuarios_preparados,
    repositorio_usuarios,
):
    caso_uso_enviar_invitacion.ejecutar(
        "usuario-123",
        "beto",
    )

    repositorio_usuarios.obtener_usuario_por_id.assert_called_once_with(
        "usuario-123"
    )


def test_busca_receptor_por_nombre_usuario_exacto(
    caso_uso_enviar_invitacion,
    usuarios_preparados,
    repositorio_usuarios,
):
    caso_uso_enviar_invitacion.ejecutar(
        "usuario-123",
        "beto",
    )

    repositorio_usuarios.obtener_usuario_por_nombre_usuario.assert_called_once_with(
        "beto"
    )


def test_consulta_amistades_utilizando_ids(
    caso_uso_enviar_invitacion,
    usuarios_preparados,
    repositorio_amistades,
):
    caso_uso_enviar_invitacion.ejecutar(
        "usuario-123",
        "beto",
    )

    repositorio_amistades.listar_entre_usuarios.assert_called_once_with(
        "usuario-123",
        "usuario-456",
    )


# --------------------------------------------------
# VALIDACIONES DEL DOMINIO
# --------------------------------------------------

def test_no_permite_autoinvitacion(
    caso_uso_enviar_invitacion,
    repositorio_usuarios,
    repositorio_amistades,
    usuario_existente,
):
    repositorio_usuarios.obtener_usuario_por_nombre_usuario.return_value = (
        usuario_existente
    )

    with pytest.raises(ErrorDominio) as exc_info:
        caso_uso_enviar_invitacion.ejecutar(
            "usuario-123",
            "eduardo",
        )

    assert exc_info.value.campo == "receptor"

    repositorio_amistades.guardar_invitacion.assert_not_called()


@pytest.mark.parametrize(
    "estado",
    [
        EstadoAmistad.PENDIENTE,
        EstadoAmistad.ACTIVA,
    ],
)
def test_rechaza_invitacion_si_existe_relacion_incompatible(
    caso_uso_enviar_invitacion,
    usuarios_preparados,
    repositorio_amistades,
    estado,
):
    repositorio_amistades.listar_entre_usuarios.return_value = [
        Amistad(
            id="amistad-existente-123",
            id_solicitante="usuario-123",
            id_receptor="usuario-456",
            estado=estado,
        )
    ]

    with pytest.raises(ErrorConflicto):
        caso_uso_enviar_invitacion.ejecutar(
            "usuario-123",
            "beto",
        )

    repositorio_amistades.guardar_invitacion.assert_not_called()


def test_rechaza_invitacion_pendiente_en_sentido_inverso(
    caso_uso_enviar_invitacion,
    usuarios_preparados,
    repositorio_amistades,
):
    repositorio_amistades.listar_entre_usuarios.return_value = [
        Amistad(
            id="amistad-existente-123",
            id_solicitante="usuario-456",
            id_receptor="usuario-123",
            estado=EstadoAmistad.PENDIENTE,
        )
    ]

    with pytest.raises(ErrorConflicto):
        caso_uso_enviar_invitacion.ejecutar(
            "usuario-123",
            "beto",
        )

    repositorio_amistades.guardar_invitacion.assert_not_called()


def test_permite_reinvitar_tras_rechazo(
    caso_uso_enviar_invitacion,
    usuarios_preparados,
    repositorio_amistades,
):
    repositorio_amistades.listar_entre_usuarios.return_value = [
        Amistad(
            id="amistad-anterior-123",
            id_solicitante="usuario-123",
            id_receptor="usuario-456",
            estado=EstadoAmistad.RECHAZADA,
        )
    ]

    resultado = caso_uso_enviar_invitacion.ejecutar(
        "usuario-123",
        "beto",
    )

    assert resultado.estado == "PENDIENTE"

    invitacion_nueva = (
        repositorio_amistades
        .guardar_invitacion.call_args.args[0]
    )

    assert invitacion_nueva.id != "amistad-anterior-123"
    assert invitacion_nueva.estado == EstadoAmistad.PENDIENTE

    repositorio_amistades.guardar_invitacion.assert_called_once()


# --------------------------------------------------
# USUARIOS INEXISTENTES
# --------------------------------------------------

def test_rechaza_destinatario_inexistente(
    caso_uso_enviar_invitacion,
    repositorio_usuarios,
    repositorio_amistades,
):
    repositorio_usuarios.obtener_usuario_por_nombre_usuario.return_value = None

    with pytest.raises(UsuarioNoEncontrado):
        caso_uso_enviar_invitacion.ejecutar(
            "usuario-123",
            "inexistente",
        )

    repositorio_amistades.guardar_invitacion.assert_not_called()
    repositorio_amistades.listar_entre_usuarios.assert_not_called()


def test_rechaza_solicitante_inexistente(
    caso_uso_enviar_invitacion,
    repositorio_usuarios,
    repositorio_amistades,
):
    repositorio_usuarios.obtener_usuario_por_id.return_value = None

    with pytest.raises(UsuarioNoEncontrado):
        caso_uso_enviar_invitacion.ejecutar(
            "usuario-inexistente",
            "beto",
        )

    repositorio_usuarios.obtener_usuario_por_nombre_usuario.assert_not_called()

    repositorio_amistades.guardar_invitacion.assert_not_called()
    repositorio_amistades.listar_entre_usuarios.assert_not_called()


# --------------------------------------------------
# CONFLICTOS DURANTE PERSISTENCIA
# --------------------------------------------------

def test_propaga_conflicto_detectado_al_guardar(
    caso_uso_enviar_invitacion,
    usuarios_preparados,
    repositorio_amistades,
):
    repositorio_amistades.guardar_invitacion.side_effect = (
        ErrorConflicto(
            "receptor",
            "Ya existe una invitación pendiente.",
        )
    )

    with pytest.raises(ErrorConflicto):
        caso_uso_enviar_invitacion.ejecutar(
            "usuario-123",
            "beto",
        )

    repositorio_amistades.guardar_invitacion.assert_called_once()


# --------------------------------------------------
# SEGURIDAD
# --------------------------------------------------

def test_no_devuelve_datos_privados(
    caso_uso_enviar_invitacion,
    usuarios_preparados,
):
    resultado = caso_uso_enviar_invitacion.ejecutar(
        "usuario-123",
        "beto",
    )

    assert not hasattr(resultado, "correo")
    assert not hasattr(resultado, "contrasena")
    assert not hasattr(resultado, "hash_contrasena")
