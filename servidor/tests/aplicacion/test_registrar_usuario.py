from datetime import datetime
from unittest.mock import Mock

import pytest

from aplicacion.errores import (
    UsuarioYaExistente,
    ContrasenaInvalida
)
from dominio.usuario import Usuario
from dominio.credenciales import Credenciales
from dominio.errores import ErrorDominio



# ==========================================
# REGISTRO EXITOSO
# ==========================================

def test_registro_exitoso(caso_uso, datos_validos):
    usuario = caso_uso.ejecutar(**datos_validos)

    assert isinstance(usuario, Usuario)
    assert usuario.nombre_usuario == "eduardo"
    assert usuario.nombre_visible == "Eduardo"
    assert usuario.correo == "eduardo@example.com"
    assert isinstance(usuario.fecha_alta, datetime)
    assert usuario.fecha_alta.tzinfo is not None
    assert usuario.id



def test_correo_se_normaliza(caso_uso, datos_validos):
    datos_validos["correo"] = "  Eduardo@Example.COM  "

    usuario = caso_uso.ejecutar(**datos_validos)

    assert usuario.correo == "eduardo@example.com"



def test_cada_usuario_tiene_id_unico(caso_uso, datos_validos):
    primero = caso_uso.ejecutar(**datos_validos)

    datos_validos["nombre_usuario"] = "otro"
    datos_validos["correo"] = "otro@example.com"

    segundo = caso_uso.ejecutar(**datos_validos)

    assert primero.id != segundo.id



# ==========================================
# VALIDACIONES
# ==========================================

@pytest.mark.parametrize("contrasena", [
    "",
    "123",
    "123456789",
])
def test_contrasena_corta(
    caso_uso,
    datos_validos,
    contrasena
):
    datos_validos["contrasena"] = contrasena

    with pytest.raises(ContrasenaInvalida) as error:
        caso_uso.ejecutar(**datos_validos)

    assert error.value.campo == "contrasena"



@pytest.mark.parametrize("nombre_usuario", [
    "",
    "ab",
    "Eduardo",
    "usuario-invalido",
    "a" * 31,
])
def test_nombre_usuario_invalido(
    caso_uso,
    datos_validos,
    nombre_usuario
):
    datos_validos["nombre_usuario"] = nombre_usuario

    with pytest.raises(ErrorDominio) as error:
        caso_uso.ejecutar(**datos_validos)

    assert error.value.campo == "nombre_usuario"



@pytest.mark.parametrize("correo", [
    "",
    "correo_invalido",
    "usuario@",
    "@example.com",
])
def test_correo_invalido(
    caso_uso,
    datos_validos,
    correo
):
    datos_validos["correo"] = correo

    with pytest.raises(ErrorDominio) as error:
        caso_uso.ejecutar(**datos_validos)

    assert error.value.campo == "correo"



@pytest.mark.parametrize("nombre_visible", [
    "",
    " ",
    "   ",
])
def test_nombre_visible_vacio(
    caso_uso,
    datos_validos,
    nombre_visible
):
    datos_validos["nombre_visible"] = nombre_visible

    with pytest.raises(ErrorDominio) as error:
        caso_uso.ejecutar(**datos_validos)

    assert error.value.campo == "nombre_visible"



# ==========================================
# UNICIDAD
# ==========================================

def test_nombre_usuario_duplicado(
    caso_uso,
    repositorio_usuarios,
    datos_validos
):
    repositorio_usuarios.obtener_usuario_por_nombre_usuario.return_value = (
        Mock(spec=Usuario)
    )

    with pytest.raises(UsuarioYaExistente) as error:
        caso_uso.ejecutar(**datos_validos)

    assert error.value.campo == "nombre_usuario"



def test_correo_duplicado(
    caso_uso,
    repositorio_usuarios,
    datos_validos
):
    repositorio_usuarios.obtener_usuario_por_correo.return_value = (
        Mock(spec=Usuario)
    )

    with pytest.raises(UsuarioYaExistente) as error:
        caso_uso.ejecutar(**datos_validos)

    assert error.value.campo == "correo"



def test_correo_duplicado_sin_importar_mayusculas(
    caso_uso,
    repositorio_usuarios,
    datos_validos
):
    datos_validos["correo"] = "EDUARDO@EXAMPLE.COM"

    repositorio_usuarios.obtener_usuario_por_correo.return_value = (
        Mock(spec=Usuario)
    )

    with pytest.raises(UsuarioYaExistente):
        caso_uso.ejecutar(**datos_validos)

    repositorio_usuarios.obtener_usuario_por_correo.assert_called_once_with(
        "eduardo@example.com"
    )



# ==========================================
# HASH DE CONTRASEÑA
# ==========================================

def test_se_genera_hash(
    caso_uso,
    servicio_hash,
    datos_validos
):
    caso_uso.ejecutar(**datos_validos)

    servicio_hash.generar_hash.assert_called_once_with(
        datos_validos["contrasena"]
    )


def test_contrasena_no_se_guarda_en_texto_plano(
    caso_uso,
    persistencia_registro,
    servicio_hash,
    datos_validos
):
    usuario = caso_uso.ejecutar(**datos_validos)

    args, _ = persistencia_registro.guardar_registro.call_args

    usuario_guardado, credenciales = args

    assert isinstance(credenciales, Credenciales)
    assert credenciales.usuario_id == usuario.id
    assert credenciales.hash_contrasena == servicio_hash.generar_hash.return_value

    assert credenciales.hash_contrasena != datos_validos["contrasena"]
    assert not hasattr(usuario_guardado, "contrasena")
    assert not hasattr(usuario_guardado, "hash_contrasena")


# ==========================================
# PERSISTENCIA
# ==========================================

def test_persistencia_se_invoca_una_vez(
    caso_uso,
    persistencia_registro,
    datos_validos
):
    usuario = caso_uso.ejecutar(**datos_validos)

    persistencia_registro.guardar_registro.assert_called_once()

    args, _ = persistencia_registro.guardar_registro.call_args

    usuario_guardado, credenciales = args

    assert usuario_guardado == usuario
    assert credenciales.usuario_id == usuario.id


def test_no_persiste_si_contrasena_invalida(
    caso_uso,
    persistencia_registro,
    servicio_hash,
    datos_validos
):
    datos_validos["contrasena"] = "123"

    with pytest.raises(ContrasenaInvalida):
        caso_uso.ejecutar(**datos_validos)

    persistencia_registro.guardar_registro.assert_not_called()
    servicio_hash.generar_hash.assert_not_called()


def test_no_persiste_si_usuario_duplicado(
    caso_uso,
    repositorio_usuarios,
    persistencia_registro,
    datos_validos
):
    repositorio_usuarios.obtener_usuario_por_nombre_usuario.return_value = (
        Mock(spec=Usuario)
    )

    with pytest.raises(UsuarioYaExistente):
        caso_uso.ejecutar(**datos_validos)

    persistencia_registro.guardar_registro.assert_not_called()


def test_propaga_conflicto_de_persistencia(
    caso_uso,
    persistencia_registro,
    datos_validos
):
    persistencia_registro.guardar_registro.side_effect = (
        UsuarioYaExistente("correo")
    )

    with pytest.raises(UsuarioYaExistente) as error:
        caso_uso.ejecutar(**datos_validos)

    assert error.value.campo == "correo"


def test_propaga_fallo_tecnico_de_persistencia(
    caso_uso,
    persistencia_registro,
    datos_validos
):
    persistencia_registro.guardar_registro.side_effect = RuntimeError(
        "Fallo simulado de persistencia"
    )

    with pytest.raises(RuntimeError):
        caso_uso.ejecutar(**datos_validos)

    persistencia_registro.guardar_registro.assert_called_once()