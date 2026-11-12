import pytest

from aplicacion.dtos.usuario_publico import UsuarioPublico
from aplicacion.errores import UsuarioNoEncontrado


def test_busqueda_exitosa(
    caso_uso_buscar_usuario,
    repositorio_usuarios,
    usuario_existente
):
    repositorio_usuarios.obtener_usuario_por_nombre_usuario.return_value = (
        usuario_existente
    )

    resultado = caso_uso_buscar_usuario.ejecutar("eduardo")

    assert isinstance(resultado, UsuarioPublico)
    assert resultado.nombre_usuario == "eduardo"
    assert resultado.nombre_visible == "Eduardo"


def test_no_devuelve_datos_privados(
    caso_uso_buscar_usuario,
    repositorio_usuarios,
    usuario_existente
):
    repositorio_usuarios.obtener_usuario_por_nombre_usuario.return_value = (
        usuario_existente
    )

    resultado = caso_uso_buscar_usuario.ejecutar("eduardo")

    assert not hasattr(resultado, "correo")
    assert not hasattr(resultado, "id")
    assert not hasattr(resultado, "fecha_alta")


def test_usuario_no_encontrado(
    caso_uso_buscar_usuario,
    repositorio_usuarios
):
    repositorio_usuarios.obtener_usuario_por_nombre_usuario.return_value = None

    with pytest.raises(UsuarioNoEncontrado):
        caso_uso_buscar_usuario.ejecutar("inexistente")


@pytest.mark.parametrize(
    "nombre",
    [
        "",
        "ed",
        "EDUARDO",
        "edu ardo",
        "edu%",
        " eduardo ",
        "a" * 31,
    ]
)
def test_rechaza_busquedas_invalidas(
    caso_uso_buscar_usuario,
    repositorio_usuarios,
    nombre
):
    with pytest.raises(UsuarioNoEncontrado):
        caso_uso_buscar_usuario.ejecutar(nombre)

    repositorio_usuarios.obtener_usuario_por_nombre_usuario.assert_not_called()


def test_busqueda_por_nombre_exacto(
    caso_uso_buscar_usuario,
    repositorio_usuarios,
    usuario_existente
):
    repositorio_usuarios.obtener_usuario_por_nombre_usuario.return_value = (
        usuario_existente
    )

    caso_uso_buscar_usuario.ejecutar("eduardo")

    repositorio_usuarios.obtener_usuario_por_nombre_usuario.assert_called_once_with(
        "eduardo"
    )


def test_no_realiza_busqueda_parcial(
    caso_uso_buscar_usuario,
    repositorio_usuarios
):
    repositorio_usuarios.obtener_usuario_por_nombre_usuario.return_value = None

    with pytest.raises(UsuarioNoEncontrado):
        caso_uso_buscar_usuario.ejecutar("eduar")

    repositorio_usuarios.obtener_usuario_por_nombre_usuario.assert_called_once_with(
        "eduar"
    )