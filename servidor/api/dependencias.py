"""
Contenedor de dependencias e inyección para la capa API.
Define las funciones de fábrica para los casos de uso requeridos por las rutas.
De momento con implementaciones vacías / placeholders para completar la integración más adelante.
"""
from aplicacion.registrar_usuario import RegistrarUsuario
from aplicacion.iniciar_sesion import IniciarSesion
from aplicacion.obtener_perfil_propio import ObtenerPerfilPropio


def caso_uso_registrar_usuario() -> RegistrarUsuario:
    """
    Instancia el caso de uso RegistrarUsuario con sus dependencias
    (RepositorioUsuarios, PersistenciaRegistro, ServicioHashContrasenas).
    """
    raise NotImplementedError("Pendiente de instanciar con el repositorio definitivo.")


def caso_uso_iniciar_sesion() -> IniciarSesion:
    """
    Instancia el caso de uso IniciarSesion con sus dependencias
    (RepositorioUsuarios, RepositorioCredenciales, ServicioHashContrasenas, ServicioTokens).
    """
    raise NotImplementedError("Pendiente de instanciar con el repositorio definitivo.")


def caso_uso_obtener_perfil_propio() -> ObtenerPerfilPropio:
    """
    Instancia el caso de uso ObtenerPerfilPropio con sus dependencias
    (RepositorioUsuarios, ServicioTokens).
    """
    raise NotImplementedError("Pendiente de instanciar con el repositorio definitivo.")

