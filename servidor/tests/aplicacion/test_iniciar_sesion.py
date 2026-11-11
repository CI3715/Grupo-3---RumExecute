from datetime import datetime, timezone
from unittest.mock import Mock

import pytest

from aplicacion.dtos.sesion_respuesta import SesionRespuesta
from aplicacion.errores import CredencialesInvalidas
from aplicacion.iniciar_sesion import IniciarSesion


# ==========================================
# INICIO DE SESIÓN EXITOSO
# ==========================================

def test_inicio_sesion_exitoso(
    caso_uso_iniciar_sesion,
    usuario_existente
):
    resultado = caso_uso_iniciar_sesion.ejecutar(
        correo="eduardo@example.com",
        contrasena="contrasena123"
    )

    assert isinstance(resultado, SesionRespuesta)
    assert resultado.usuario == usuario_existente
    assert resultado.token == "token_de_prueba"
    assert isinstance(resultado.fecha_expiracion, datetime)


def test_correo_se_normaliza(
    caso_uso_iniciar_sesion
):
    caso_uso_iniciar_sesion.ejecutar(
        correo="  EDUARDO@EXAMPLE.COM  ",
        contrasena="contrasena123"
    )

    repositorio = caso_uso_iniciar_sesion.repositorio_usuarios

    repositorio.obtener_usuario_por_correo.assert_called_once_with(
        "eduardo@example.com"
    )


def test_verifica_contrasena_con_hash_almacenado(
    caso_uso_iniciar_sesion,
    servicio_hash
):
    caso_uso_iniciar_sesion.ejecutar(
        correo="eduardo@example.com",
        contrasena="contrasena123"
    )

    servicio_hash.verificar_contrasena.assert_called_once_with(
        "contrasena123",
        "hash_de_prueba"
    )


def test_genera_token_para_usuario_correcto(
    caso_uso_iniciar_sesion,
    servicio_tokens
):
    caso_uso_iniciar_sesion.ejecutar(
        correo="eduardo@example.com",
        contrasena="contrasena123"
    )

    servicio_tokens.generar_token.assert_called_once_with(
        usuario_id="usuario-123",
        duracion_minutos=60
    )


def test_fecha_expiracion_tiene_zona_horaria(
    caso_uso_iniciar_sesion
):
    resultado = caso_uso_iniciar_sesion.ejecutar(
        correo="eduardo@example.com",
        contrasena="contrasena123"
    )

    assert resultado.fecha_expiracion.tzinfo is not None
    assert resultado.fecha_expiracion > datetime.now(timezone.utc)


# ==========================================
# CREDENCIALES INVÁLIDAS
# ==========================================

def test_correo_inexistente(
    caso_uso_iniciar_sesion,
    servicio_tokens
):
    repositorio = caso_uso_iniciar_sesion.repositorio_usuarios
    repositorio.obtener_usuario_por_correo.return_value = None

    with pytest.raises(CredencialesInvalidas) as error:
        caso_uso_iniciar_sesion.ejecutar(
            correo="inexistente@example.com",
            contrasena="contrasena123"
        )

    assert str(error.value) == "Correo o contraseña incorrectos."

    servicio_tokens.generar_token.assert_not_called()


def test_contrasena_incorrecta(
    caso_uso_iniciar_sesion,
    servicio_hash,
    servicio_tokens
):
    servicio_hash.verificar_contrasena.return_value = False

    with pytest.raises(CredencialesInvalidas) as error:
        caso_uso_iniciar_sesion.ejecutar(
            correo="eduardo@example.com",
            contrasena="incorrecta123"
        )

    assert str(error.value) == "Correo o contraseña incorrectos."

    servicio_tokens.generar_token.assert_not_called()


def test_credenciales_no_encontradas(
    caso_uso_iniciar_sesion,
    servicio_tokens
):
    repositorio = caso_uso_iniciar_sesion.repositorio_credenciales
    repositorio.obtener_por_usuario_id.return_value = None

    with pytest.raises(CredencialesInvalidas):
        caso_uso_iniciar_sesion.ejecutar(
            correo="eduardo@example.com",
            contrasena="contrasena123"
        )

    servicio_tokens.generar_token.assert_not_called()


def test_mismo_error_si_usuario_no_existe_o_contrasena_incorrecta(
    caso_uso_iniciar_sesion,
    servicio_hash
):
    repositorio = caso_uso_iniciar_sesion.repositorio_usuarios

    repositorio.obtener_usuario_por_correo.return_value = None

    with pytest.raises(CredencialesInvalidas) as error_usuario:
        caso_uso_iniciar_sesion.ejecutar(
            "inexistente@example.com",
            "contrasena123"
        )

    repositorio.obtener_usuario_por_correo.return_value = Mock()

    repositorio_credenciales = (
        caso_uso_iniciar_sesion.repositorio_credenciales
    )

    servicio_hash.verificar_contrasena.return_value = False

    with pytest.raises(CredencialesInvalidas) as error_contrasena:
        caso_uso_iniciar_sesion.ejecutar(
            "eduardo@example.com",
            "incorrecta123"
        )

    assert str(error_usuario.value) == str(error_contrasena.value)


# ==========================================
# INTERACCIONES CON LOS REPOSITORIOS
# ==========================================

def test_consulta_credenciales_por_usuario_id(
    caso_uso_iniciar_sesion
):
    caso_uso_iniciar_sesion.ejecutar(
        "eduardo@example.com",
        "contrasena123"
    )

    repositorio = caso_uso_iniciar_sesion.repositorio_credenciales

    repositorio.obtener_por_usuario_id.assert_called_once_with(
        "usuario-123"
    )


def test_no_consulta_credenciales_si_usuario_no_existe(
    caso_uso_iniciar_sesion
):
    repositorio_usuarios = (
        caso_uso_iniciar_sesion.repositorio_usuarios
    )

    repositorio_credenciales = (
        caso_uso_iniciar_sesion.repositorio_credenciales
    )

    repositorio_usuarios.obtener_usuario_por_correo.return_value = None

    with pytest.raises(CredencialesInvalidas):
        caso_uso_iniciar_sesion.ejecutar(
            "inexistente@example.com",
            "contrasena123"
        )

    repositorio_credenciales.obtener_por_usuario_id.assert_not_called()


def test_no_modifica_usuario(
    caso_uso_iniciar_sesion,
    usuario_existente
):
    usuario_original = usuario_existente

    resultado = caso_uso_iniciar_sesion.ejecutar(
        "eduardo@example.com",
        "contrasena123"
    )

    assert resultado.usuario == usuario_original
    assert resultado.usuario.id == usuario_original.id


# ==========================================
# FALLOS DE SERVICIOS EXTERNOS
# ==========================================

def test_propaga_error_generacion_token(
    caso_uso_iniciar_sesion,
    servicio_tokens
):
    servicio_tokens.generar_token.side_effect = RuntimeError(
        "Fallo simulado generando token"
    )

    with pytest.raises(RuntimeError):
        caso_uso_iniciar_sesion.ejecutar(
            "eduardo@example.com",
            "contrasena123"
        )