from typing import Protocol, List, Tuple

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

    def consultar_cambios_usuario(
        self,
        usuario_id: str,
        desde_cursor: int,
        limite: int = 100,
    ) -> Tuple[List[EventoCambioUsuario], int, bool]:
        """
        Consulta eventos de cambio del usuario posteriores al cursor.
        Ordenados estrictamente por posicion ASC en lotes de maximo `limite`.
        Retorna (eventos, nuevo_cursor, quedan_mas).
        """
        ...
