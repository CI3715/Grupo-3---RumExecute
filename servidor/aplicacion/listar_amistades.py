from aplicacion.dtos.listado_amistades import (
    AmigoRespuesta,
    InvitacionAmistadResumen,
    ListadoAmistades,
)
from aplicacion.errores import UsuarioNoEncontrado
from aplicacion.puertos.repositorio_amistades import RepositorioAmistades
from aplicacion.puertos.repositorio_usuarios import RepositorioUsuarios
from dominio.amistad import EstadoAmistad


class ListarAmistades:

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
    ) -> ListadoAmistades:

        usuario = self.repositorio_usuarios.obtener_usuario_por_id(
            usuario_autenticado_id
        )

        if usuario is None:
            raise UsuarioNoEncontrado()

        amistades = self.repositorio_amistades.listar_de_usuario(
            usuario_autenticado_id
        )

        amigos = []
        recibidas = []
        enviadas = []

        for amistad in amistades:

            if amistad.estado == EstadoAmistad.RECHAZADA:
                continue

            # Identificar al otro usuario de la relación.
            if amistad.id_solicitante == usuario_autenticado_id:
                otro_usuario_id = amistad.id_receptor
            elif amistad.id_receptor == usuario_autenticado_id:
                otro_usuario_id = amistad.id_solicitante
            else:
                # No exponer relaciones ajenas al usuario.
                continue

            otro_usuario = self.repositorio_usuarios.obtener_usuario_por_id(
                otro_usuario_id
            )

            if otro_usuario is None:
                raise UsuarioNoEncontrado()

            if amistad.estado == EstadoAmistad.ACTIVA:
                amigos.append(
                    AmigoRespuesta(
                        amistad_id=amistad.id,
                        nombre_usuario=otro_usuario.nombre_usuario,
                        nombre_visible=otro_usuario.nombre_visible,
                    )
                )

            elif amistad.estado == EstadoAmistad.PENDIENTE:
                resumen = InvitacionAmistadResumen(
                    amistad_id=amistad.id,
                    nombre_usuario=otro_usuario.nombre_usuario,
                    nombre_visible=otro_usuario.nombre_visible,
                )

                if amistad.id_receptor == usuario_autenticado_id:
                    recibidas.append(resumen)
                else:
                    enviadas.append(resumen)

        return ListadoAmistades(
            amigos=amigos,
            invitaciones_recibidas=recibidas,
            invitaciones_enviadas=enviadas,
        )
