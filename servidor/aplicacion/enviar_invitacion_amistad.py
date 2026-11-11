from aplicacion.puertos.repositorio_usuarios import (
    RepositorioUsuarios,
)
from aplicacion.puertos.repositorio_amistades import (
    RepositorioAmistades,
)
from aplicacion.dtos.invitacion_amistad_respuesta import (
    InvitacionAmistadRespuesta,
)
from aplicacion.errores import UsuarioNoEncontrado

from dominio.amistad import (
    Amistad,
    validar_solicitud_amistad,
)


class EnviarInvitacionAmistad:

    def __init__(
        self,
        repositorio_usuarios: RepositorioUsuarios,
        repositorio_amistades: RepositorioAmistades,
    ):
        self.repositorio_usuarios = repositorio_usuarios
        self.repositorio_amistades = repositorio_amistades

    def ejecutar(
        self,
        usuario_autenticado_id: str,
        nombre_usuario_receptor: str,
    ) -> InvitacionAmistadRespuesta:

        # 1. Obtener el usuario autenticado
        solicitante = (
            self.repositorio_usuarios
            .obtener_usuario_por_id(usuario_autenticado_id)
        )

        if solicitante is None:
            raise UsuarioNoEncontrado()

        # 2. Buscar al destinatario por nombre exacto
        receptor = (
            self.repositorio_usuarios
            .obtener_usuario_por_nombre_usuario(
                nombre_usuario_receptor
            )
        )

        if receptor is None:
            raise UsuarioNoEncontrado()

        # 3. Consultar las relaciones existentes
        amistades_existentes = (
            self.repositorio_amistades.listar_entre_usuarios(
                solicitante.nombre_usuario,
                receptor.nombre_usuario,
            )
        )

        # 4. Validar las reglas del dominio
        validar_solicitud_amistad(
            solicitante.nombre_usuario,
            receptor.nombre_usuario,
            amistades_existentes,
        )

        # 5. Crear invitación pendiente
        invitacion = Amistad(
            nombre_usuario_solicitante=solicitante.nombre_usuario,
            nombre_usuario_receptor=receptor.nombre_usuario,
        )

        # 6. Persistir y obtener el identificador
        invitacion_id = (
            self.repositorio_amistades.guardar_invitacion(
                invitacion
            )
        )

        # 7. Devolver información pública
        return InvitacionAmistadRespuesta(
            id=invitacion_id,
            nombre_usuario_solicitante=(
                invitacion.nombre_usuario_solicitante
            ),
            nombre_usuario_receptor=(
                invitacion.nombre_usuario_receptor
            ),
            estado=invitacion.estado.name,
        )