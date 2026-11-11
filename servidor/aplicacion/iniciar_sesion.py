from aplicacion.puertos.repositorio_usuarios import RepositorioUsuarios
from aplicacion.puertos.repositorio_credenciales import RepositorioCredenciales
from aplicacion.puertos.servicio_hash_contrasenas import ServicioHashContrasenas
from aplicacion.puertos.servicio_tokens import ServicioTokens

from aplicacion.dtos.sesion_respuesta import SesionRespuesta
from aplicacion.errores import CredencialesInvalidas


class IniciarSesion:
    """
    Caso de uso para iniciar sesión.

    Verifica las credenciales de un usuario
    y genera un token de sesión temporal.
    """

    DURACION_SESION_MINUTOS = 60

    def __init__(
        self,
        repositorio_usuarios: RepositorioUsuarios,
        repositorio_credenciales: RepositorioCredenciales,
        servicio_hash: ServicioHashContrasenas,
        servicio_tokens: ServicioTokens
    ):
        self.repositorio_usuarios = repositorio_usuarios
        self.repositorio_credenciales = repositorio_credenciales
        self.servicio_hash = servicio_hash
        self.servicio_tokens = servicio_tokens

    def ejecutar(
        self,
        correo: str,
        contrasena: str
    ) -> SesionRespuesta:
        """
        Inicia sesión con correo y contraseña.

        Raises:
            CredencialesInvalidas:
                Si el correo o la contraseña son incorrectos.

        Returns:
            SesionRespuesta:
                Usuario, token y fecha de expiración.
        """

        # Normalizar correo
        correo = correo.strip().lower()

        # Buscar usuario
        usuario = (
            self.repositorio_usuarios
            .obtener_usuario_por_correo(correo)
        )

        if usuario is None:
            raise CredencialesInvalidas()

        # Obtener credenciales
        credenciales = (
            self.repositorio_credenciales
            .obtener_por_usuario_id(usuario.id)
        )

        if credenciales is None:
            raise CredencialesInvalidas()

        # Verificar contraseña
        contrasena_correcta = (
            self.servicio_hash.verificar_contrasena(
                contrasena,
                credenciales.hash_contrasena
            )
        )

        if not contrasena_correcta:
            raise CredencialesInvalidas()

        # Generar token
        token_sesion = self.servicio_tokens.generar_token(
            usuario_id=usuario.id,
            duracion_minutos=self.DURACION_SESION_MINUTOS
        )

        return SesionRespuesta(
            usuario=usuario,
            token=token_sesion.valor,
            fecha_expiracion=token_sesion.fecha_expiracion
        )