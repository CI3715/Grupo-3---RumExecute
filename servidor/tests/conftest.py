from unittest.mock import Mock



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
    servicio.generar_hash.return_value = "hash de prueba"
    return servicio