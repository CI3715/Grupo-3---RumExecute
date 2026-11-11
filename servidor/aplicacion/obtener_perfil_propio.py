from aplicacion.puertos.repositorio_usuarios import RepositorioUsuarios
from aplicacion.puertos.servicio_tokens import ServicioTokens
from aplicacion.errores import TokenInvalido

from dominio.usuario import Usuario


class ObtenerPerfilPropio:

    def __init__(
        self,
        repositorio_usuarios: RepositorioUsuarios,
        servicio_tokens: ServicioTokens
    ):
        self.repositorio_usuarios = repositorio_usuarios
        self.servicio_tokens = servicio_tokens

    def ejecutar(self, token: str) -> Usuario:

        if not token:
            raise TokenInvalido()

        usuario_id = self.servicio_tokens.verificar_token(token)

        usuario = self.repositorio_usuarios.obtener_usuario_por_id(
            usuario_id
        )

        if usuario is None:
            raise TokenInvalido()

        return usuario