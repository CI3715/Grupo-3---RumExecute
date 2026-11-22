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
        limite: int,
    ) -> list[EventoCambioUsuario]:
        """
        Devuelve los eventos de un usuario con posicion mayor a 'desde'.

        Los eventos vienen en orden creciente de posicion y son como
        máximo 'limite'. Solo se consultan eventos del usuario indicado.
        """
        ...
