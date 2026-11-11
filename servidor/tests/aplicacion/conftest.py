import pytest
from unittest.mock import Mock

from datetime import datetime, timezone, timedelta

from aplicacion.registrar_usuario import RegistrarUsuario
from aplicacion.iniciar_sesion import IniciarSesion
from aplicacion.puertos.servicio_tokens import TokenSesion
from aplicacion.obtener_perfil_propio import ObtenerPerfilPropio

from dominio.usuario import Usuario
from dominio.credenciales import Credenciales



@pytest.fixture
def repositorio_usuarios():
    repositorio = Mock()
    repositorio.obtener_usuario_por_nombre_usuario.return_value = None
    repositorio.obtener_usuario_por_correo.return_value = None
    return repositorio


@pytest.fixture
def persistencia_registro():
    return Mock()



@pytest.fixture
def servicio_hash():
    servicio = Mock()
    servicio.generar_hash.return_value = "hash_de_prueba"
    return servicio



@pytest.fixture
def caso_uso(
    repositorio_usuarios,
    persistencia_registro,
    servicio_hash
):
    return RegistrarUsuario(
        repositorio_usuarios=repositorio_usuarios,
        persistencia_registro=persistencia_registro,
        servicio_hash=servicio_hash
    )



@pytest.fixture
def datos_validos():
    return {
        "nombre_usuario": "eduardo",
        "nombre_visible": "Eduardo",
        "correo": "eduardo@example.com",
        "contrasena": "contrasena123"
    }



@pytest.fixture
def usuario_existente():
    return Usuario(
        id="usuario-123",
        nombre_usuario="eduardo",
        nombre_visible="Eduardo",
        correo="eduardo@example.com",
        fecha_alta=datetime.now(timezone.utc)
    )


@pytest.fixture
def credenciales_existentes():
    return Credenciales(
        usuario_id="usuario-123",
        hash_contrasena="hash_de_prueba"
    )


@pytest.fixture
def servicio_tokens():
    servicio = Mock()

    servicio.generar_token.return_value = TokenSesion(
        valor="token_de_prueba",
        fecha_expiracion=datetime.now(timezone.utc)
        + timedelta(minutes=60)
    )

    return servicio


@pytest.fixture
def caso_uso_iniciar_sesion(
    repositorio_usuarios,
    servicio_hash,
    servicio_tokens,
    usuario_existente,
    credenciales_existentes
):
    repositorio_credenciales = Mock()

    repositorio_usuarios.obtener_usuario_por_correo.return_value = (
        usuario_existente
    )

    repositorio_credenciales.obtener_por_usuario_id.return_value = (
        credenciales_existentes
    )

    servicio_hash.verificar_contrasena.return_value = True

    return IniciarSesion(
        repositorio_usuarios=repositorio_usuarios,
        repositorio_credenciales=repositorio_credenciales,
        servicio_hash=servicio_hash,
        servicio_tokens=servicio_tokens
    )


@pytest.fixture
def caso_uso_obtener_perfil_propio(
    repositorio_usuarios,
    servicio_tokens,
    usuario_existente
):
    servicio_tokens.verificar_token.return_value = usuario_existente.id

    repositorio_usuarios.obtener_usuario_por_id.return_value = (
        usuario_existente
    )

    return ObtenerPerfilPropio(
        repositorio_usuarios=repositorio_usuarios,
        servicio_tokens=servicio_tokens
    )