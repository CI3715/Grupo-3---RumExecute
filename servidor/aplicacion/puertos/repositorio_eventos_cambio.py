"""
Puerto para el repositorio de eventos de cambio de usuario (Entrega 7).
"""
from typing import Protocol, List, Tuple
from dominio.evento_cambio import EventoCambioUsuario


class RepositorioEventosCambio(Protocol):
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
