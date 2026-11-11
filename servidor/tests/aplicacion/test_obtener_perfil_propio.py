import pytest

from aplicacion.errores import TokenInvalido
from dominio.usuario import Usuario


TOKEN_VALIDO = "token_de_prueba"


# --------------------------------------------------
# CASO EXITOSO
# --------------------------------------------------

def test_obtener_perfil_propio_exitoso(
    caso_uso_obtener_perfil_propio,
    usuario_existente
):
    usuario = caso_uso_obtener_perfil_propio.ejecutar(TOKEN_VALIDO)

    assert isinstance(usuario, Usuario)
    assert usuario == usuario_existente


def test_devuelve_datos_del_usuario_correcto(
    caso_uso_obtener_perfil_propio
):
    usuario = caso_uso_obtener_perfil_propio.ejecutar(TOKEN_VALIDO)

    assert usuario.id == "usuario-123"
    assert usuario.nombre_usuario == "eduardo"
    assert usuario.nombre_visible == "Eduardo"
    assert usuario.correo == "eduardo@example.com"


# --------------------------------------------------
# VALIDACIÓN DEL TOKEN
# --------------------------------------------------

@pytest.mark.parametrize(
    "token",
    ["", None]
)
def test_token_ausente(
    caso_uso_obtener_perfil_propio,
    token
):
    with pytest.raises(TokenInvalido):
        caso_uso_obtener_perfil_propio.ejecutar(token)


def test_token_invalido(
    caso_uso_obtener_perfil_propio
):
    caso_uso_obtener_perfil_propio.servicio_tokens.verificar_token.side_effect = (
        TokenInvalido()
    )

    with pytest.raises(TokenInvalido):
        caso_uso_obtener_perfil_propio.ejecutar("token_invalido")


def test_token_expirado(
    caso_uso_obtener_perfil_propio
):
    caso_uso_obtener_perfil_propio.servicio_tokens.verificar_token.side_effect = (
        TokenInvalido()
    )

    with pytest.raises(TokenInvalido):
        caso_uso_obtener_perfil_propio.ejecutar("token_expirado")


# --------------------------------------------------
# USUARIO INEXISTENTE
# --------------------------------------------------

def test_usuario_no_existe(
    caso_uso_obtener_perfil_propio
):
    caso_uso_obtener_perfil_propio.repositorio_usuarios.obtener_usuario_por_id.return_value = (
        None
    )

    with pytest.raises(TokenInvalido):
        caso_uso_obtener_perfil_propio.ejecutar(TOKEN_VALIDO)


# --------------------------------------------------
# INTERACCIÓN CON LOS PUERTOS
# --------------------------------------------------

def test_verifica_token_recibido(
    caso_uso_obtener_perfil_propio
):
    caso_uso_obtener_perfil_propio.ejecutar(TOKEN_VALIDO)

    caso_uso_obtener_perfil_propio.servicio_tokens.verificar_token.assert_called_once_with(
        TOKEN_VALIDO
    )


def test_consulta_usuario_por_id_obtenido_del_token(
    caso_uso_obtener_perfil_propio
):
    caso_uso_obtener_perfil_propio.ejecutar(TOKEN_VALIDO)

    caso_uso_obtener_perfil_propio.repositorio_usuarios.obtener_usuario_por_id.assert_called_once_with(
        "usuario-123"
    )


def test_no_consulta_repositorio_si_token_invalido(
    caso_uso_obtener_perfil_propio
):
    caso_uso_obtener_perfil_propio.servicio_tokens.verificar_token.side_effect = (
        TokenInvalido()
    )

    with pytest.raises(TokenInvalido):
        caso_uso_obtener_perfil_propio.ejecutar("token_invalido")

    caso_uso_obtener_perfil_propio.repositorio_usuarios.obtener_usuario_por_id.assert_not_called()


def test_no_verifica_token_si_esta_ausente(
    caso_uso_obtener_perfil_propio
):
    with pytest.raises(TokenInvalido):
        caso_uso_obtener_perfil_propio.ejecutar("")

    caso_uso_obtener_perfil_propio.servicio_tokens.verificar_token.assert_not_called()


# --------------------------------------------------
# SEGURIDAD
# --------------------------------------------------

def test_no_devuelve_credenciales(
    caso_uso_obtener_perfil_propio
):
    usuario = caso_uso_obtener_perfil_propio.ejecutar(TOKEN_VALIDO)

    assert not hasattr(usuario, "contrasena")
    assert not hasattr(usuario, "hash_contrasena")


# --------------------------------------------------
# PROPAGACIÓN DE ERRORES TÉCNICOS
# --------------------------------------------------

def test_propaga_error_del_repositorio(
    caso_uso_obtener_perfil_propio
):
    caso_uso_obtener_perfil_propio.repositorio_usuarios.obtener_usuario_por_id.side_effect = (
        RuntimeError("Error de base de datos")
    )

    with pytest.raises(RuntimeError, match="Error de base de datos"):
        caso_uso_obtener_perfil_propio.ejecutar(TOKEN_VALIDO)