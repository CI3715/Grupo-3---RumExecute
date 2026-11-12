from aplicacion.puertos.repositorio_usuarios import RepositorioUsuarios
from aplicacion.dtos.usuario_publico import UsuarioPublico
from aplicacion.errores import UsuarioNoEncontrado

from dominio.usuario import PATRON_NOMBRE_USUARIO



class BuscarUsuario:

    def __init__(
        self,
        repositorio_usuarios: RepositorioUsuarios
    ):
        self.repositorio_usuarios = repositorio_usuarios
    
    def ejecutar(
        self,
        nombre_usuario: str
    ) -> UsuarioPublico:
        """
        Busca un usuario por su nombre de usuario y devuelve su información pública.
        """
        if not PATRON_NOMBRE_USUARIO.fullmatch(nombre_usuario):
            raise UsuarioNoEncontrado()
        
        usuario = (
            self.repositorio_usuarios
            .obtener_usuario_por_nombre_usuario(nombre_usuario)
        )

        if usuario is None:
            raise UsuarioNoEncontrado()
        
        return UsuarioPublico(
            nombre_usuario=usuario.nombre_usuario,
            nombre_visible=usuario.nombre_visible
        )