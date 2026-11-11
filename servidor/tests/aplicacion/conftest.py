import pytest
from unittest.mock import Mock
from aplicacion.registrar_usuario import RegistrarUsuario



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