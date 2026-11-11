from typing import Protocol

from dominio.amistad import Amistad


class RepositorioAmistades(Protocol):

    def listar_entre_usuarios(
        self,
        usuario_a: str,
        usuario_b: str
    ) -> list[Amistad]:
        """Obtiene los registros entre dos usuarios,
        independientemente del sentido de la invitación."""
        ...

    def guardar_invitacion(
        self,
        amistad: Amistad
    ) -> str:
        """Guarda una invitación y devuelve su ID.

        Debe impedir atómicamente invitaciones pendientes
        duplicadas y amistades activas duplicadas.
        Ante un conflicto, lanza ErrorConflicto.
        """
        ...