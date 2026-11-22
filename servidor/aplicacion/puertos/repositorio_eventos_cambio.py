from typing import Protocol

from dominio.evento_cambio import EventoCambioUsuario


class RepositorioEventosCambio(Protocol):
    """
    Puerto para consultar el registro de cambios de un usuario.
    """

    def listar_posteriores(
            self,
            usuario_id: str,
            desde: int,
            limite: int
    ) -> list[EventoCambioUsuario]:
        """
        Devuelve los eventos de un usuario con posicion mayor a 'desde'.

        Los eventos vienen en orden creciente de posicion y son como
        máximo 'limite'. Solo se consultan eventos del usuario indicado.

        Args:
            usuario_id (str): Usuario dueño de los eventos
            desde (int): Cursor; se devuelven eventos con posicion > desde
            limite (int): Cantidad máxima de eventos a devolver

        Returns:
            list[EventoCambioUsuario]: Eventos en orden creciente de
                posicion; lista vacía si no hay posteriores.

        Raises:
            ErrorInfraestructura: Fallos gestionados por la capa de infraestructura.
        """
        ...