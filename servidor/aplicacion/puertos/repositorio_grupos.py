
from typing import Protocol

from dominio.grupo import GrupoCompartido
from dominio.invitacion_grupo import InvitacionGrupo


class RepositorioGrupos(Protocol):

    def guardar_grupo(
        self,
        grupo: GrupoCompartido,
    ) -> None:
        """Guarda un nuevo grupo.

        El creador debe estar incluido en id_miembros
        y debe ser el administrador del grupo.
        """
        ...

    def obtener_por_id(
        self,
        grupo_id: str,
    ) -> GrupoCompartido | None:
        """Obtiene un grupo por su identificador.

        Devuelve None si no existe.

        La aplicación debe comprobar que el usuario
        tiene permisos para acceder al grupo.
        """
        ...

    def listar_de_usuario(
        self,
        usuario_id: str,
    ) -> list[GrupoCompartido]:
        """Devuelve los grupos en los que el usuario
        figura como miembro aceptado.

        No incluye grupos a los que solo fue invitado.
        """
        ...

    def listar_invitaciones_recibidas(
        self,
        usuario_id: str,
    ) -> list[InvitacionGrupo]:
        """Obtiene las invitaciones de grupo
        recibidas por un usuario.

        Incluye únicamente las pendientes.
        """
        ...

    def listar_invitaciones_de_grupo(
        self,
        grupo_id: str,
    ) -> list[InvitacionGrupo]:
        """Obtiene las invitaciones asociadas a un grupo.

        Se utilizará para comprobar que no exista
        una invitación pendiente duplicada.
        """
        ...

    def guardar_invitacion(
        self,
        invitacion: InvitacionGrupo,
    ) -> None:
        """Guarda una invitación de grupo pendiente.

        Debe impedir que existan dos invitaciones
        pendientes para el mismo usuario y grupo.

        También debe impedir invitaciones a usuarios
        que ya son miembros del grupo.

        Si existe un conflicto, lanza ErrorConflicto.
        """
        ...

    def obtener_invitacion_por_id(
        self,
        invitacion_id: str,
    ) -> InvitacionGrupo | None:
        """Obtiene una invitación por su identificador.

        Devuelve None si no existe.
        """
        ...

    def actualizar_respuesta_invitacion(
        self,
        invitacion: InvitacionGrupo,
    ) -> None:
        """Actualiza una invitación de grupo.

        Solo permite modificar invitaciones PENDIENTES.

        Si la invitación es aceptada, debe agregar
        al invitado a los miembros del grupo.

        Si es rechazada, no modifica la membresía.

        Tanto la actualización del estado como la
        incorporación del miembro deben realizarse
        en una única transacción.

        Si ya fue respondida, lanza ErrorConflicto.
        """
        ...
