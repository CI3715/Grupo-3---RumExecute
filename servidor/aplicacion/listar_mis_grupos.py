from aplicacion.dtos.listado_grupos import (
    GrupoResumen,
    InvitacionGrupoResumen,
    ListadoGrupos,
)
from aplicacion.errores import UsuarioNoEncontrado
from aplicacion.puertos.repositorio_grupos import RepositorioGrupos
from aplicacion.puertos.repositorio_usuarios import RepositorioUsuarios
from dominio.grupo import es_miembro
from dominio.invitacion_grupo import EstadoInvitacionGrupo


class ListarMisGrupos:

    def __init__(
        self,
        repositorio_usuarios: RepositorioUsuarios,
        repositorio_grupos: RepositorioGrupos,
    ):
        self.repositorio_usuarios = repositorio_usuarios
        self.repositorio_grupos = repositorio_grupos

    def ejecutar(
        self,
        usuario_autenticado_id: str,
    ) -> ListadoGrupos:

        usuario = self.repositorio_usuarios.obtener_usuario_por_id(
            usuario_autenticado_id
        )

        if usuario is None:
            raise UsuarioNoEncontrado()

        grupos = self.repositorio_grupos.listar_de_usuario(
            usuario.id
        )

        invitaciones = (
            self.repositorio_grupos.listar_invitaciones_recibidas(
                usuario.id
            )
        )

        grupos_respuesta = [
            GrupoResumen(
                id=grupo.id,
                nombre_grupo=grupo.nombre_grupo,
                moneda=grupo.moneda,
                descripcion=grupo.descripcion,
                administrador_id=grupo.id_administrador,
            )
            for grupo in grupos
            if es_miembro(grupo, usuario.id)
        ]

        invitaciones_respuesta = [
            InvitacionGrupoResumen(
                id=invitacion.id,
                grupo_id=invitacion.grupo_id,
                invitador_id=invitacion.invitador_id,
                estado=invitacion.estado.name,
            )
            for invitacion in invitaciones
            if (
                invitacion.invitado_id == usuario.id
                and invitacion.estado
                == EstadoInvitacionGrupo.PENDIENTE
            )
        ]

        return ListadoGrupos(
            grupos=grupos_respuesta,
            invitaciones_recibidas=invitaciones_respuesta,
        )
