from typing import Protocol

from dominio.amistad import Amistad


class RepositorioAmistades(Protocol):

    def listar_entre_usuarios(
        self,
        usuario_a_id: str,
        usuario_b_id: str
    ) -> list[Amistad]:
        """Obtiene los registros entre dos usuarios,
        independientemente del sentido de la invitación."""
        ...

    def guardar_invitacion(
        self,
        amistad_id: Amistad
    ) -> None:
        """Guarda una invitación.

        Debe impedir atómicamente invitaciones pendientes
        duplicadas y amistades activas duplicadas.
        Ante un conflicto, lanza ErrorConflicto.
        """
        ...
    
    def obtener_por_id(
        self,
        amistad_id: str
    ) -> Amistad | None:
        """
        Obtiene una invitación por amistad por su ID.

        Retorna None si no existe
        """
        ...
    
    def listar_de_usuario(
        self,
        usuario_id: str
    ) -> list[Amistad]:
        """
        Devuelve las amistades del usuario. No incluye invitaciones rechazadas
        """
        ...
    
    def actualizar_respuesta(
        self,
        amistad: Amistad
    ) -> None:
        """
        Guarda la respuesta a una invitación.

        Debe actualizar una invitación solamente si su estado anterior era PENDIENTE.

        Si ya fue respondida, lanza ErrorConflicto

        La comprobación y actualización deben ser atómicas para evitar respuestas
        simultáneas.
        """
    
    def eliminar_amistad(
        self,
        amistad_id: str,
        usuario_id: str
    ) -> bool:
        """
        Elimina una amistad ACTIVA.

        Solo puede eliminarla uno de sus participantes.

        Devuelve True si se eliminó.
        Devuelve False si no existe o no está activa.

        Si existe, está activa y el usuario no participa en ella,
        lanza ErrorPermiso.

        No debe eliminar grupos ni gastos históricos.
        """