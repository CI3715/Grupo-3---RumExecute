from uuid import uuid4

from aplicacion.dtos.invitacion_grupo_respuesta import (
    InvitacionGrupoRespuesta,
)
from aplicacion.errores import GrupoNoEncontrado, UsuarioNoEncontrado
from aplicacion.puertos.repositorio_usuarios import RepositorioUsuarios
from aplicacion.puertos.repositorio_grupos import RepositorioGrupos
from aplicacion.puertos.repositorio_amistades import RepositorioAmistades

from dominio.grupo import es_miembro
from dominio.invitacion_grupo import (
    InvitacionGrupo,
    validar_invitacion_grupo,
)


class InvitarMiembroGrupo:

    def __init__(
        self,
        repositorio_usuarios: RepositorioUsuarios,
        repositorio_grupos: RepositorioGrupos,
        repositorio_amistades: RepositorioAmistades,
    ):
        self.repositorio_usuarios = repositorio_usuarios
        self.repositorio_grupos = repositorio_grupos
        self.repositorio_amistades = repositorio_amistades

    def ejecutar(
        self,
        usuario_autenticado_id: str,
        grupo_id: str,
        nombre_usuario_invitado: str,
    ) -> InvitacionGrupoRespuesta:

        # Obtener el grupo.
        grupo = self.repositorio_grupos.obtener_por_id(grupo_id)

        # No revelar grupos a usuarios ajenos.
        if grupo is None or not es_miembro(
            grupo, usuario_autenticado_id
        ):
            raise GrupoNoEncontrado()

        # Verificar que quien invita sea administrador
        # antes de consultar información de otros usuarios.
        from dominio.grupo import validar_es_admin
        validar_es_admin(grupo, usuario_autenticado_id)

        # Buscar al invitado por su nombre de usuario exacto.
        invitado = (
            self.repositorio_usuarios.obtener_usuario_por_nombre_usuario(
                nombre_usuario_invitado
            )
        )

        if invitado is None:
            raise UsuarioNoEncontrado()

        # Obtener las relaciones entre ambos usuarios.
        amistades = self.repositorio_amistades.listar_entre_usuarios(
            usuario_autenticado_id,
            invitado.id,
        )

        # Obtener las invitaciones existentes del grupo.
        invitaciones = (
            self.repositorio_grupos.listar_invitaciones_de_grupo(
                grupo.id
            )
        )

        # Validar permisos, amistad y duplicados en el dominio.
        validar_invitacion_grupo(
            grupo=grupo,
            invitador_id=usuario_autenticado_id,
            invitado_id=invitado.id,
            amistades=amistades,
            invitaciones=invitaciones,
        )

        # Crear una invitación pendiente.
        invitacion = InvitacionGrupo(
            id=str(uuid4()),
            grupo_id=grupo.id,
            invitado_id=invitado.id,
            invitador_id=usuario_autenticado_id,
        )

        self.repositorio_grupos.guardar_invitacion(invitacion)

        return InvitacionGrupoRespuesta(
            id=invitacion.id,
            grupo_id=invitacion.grupo_id,
            invitado_id=invitacion.invitado_id,
            invitador_id=invitacion.invitador_id,
            estado=invitacion.estado.name,
        )
