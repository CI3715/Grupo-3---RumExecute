from typing import Protocol

from dominio.balance import Gasto
from dominio.evento_cambio import EventoCambioUsuario
from dominio.idempotencia import RegistroIdempotenciaGasto


class PersistenciaGastoCompartido(Protocol):
    """
    Puerto para persistir atómicamente un gasto compartido, los eventos
    de cambio de sus participantes y su clave de idempotencia.
    """

    def guardar_gasto(
            self,
            gasto: Gasto,
            eventos: tuple[EventoCambioUsuario, ...],
            registro_idempotencia: RegistroIdempotenciaGasto | None
    ) -> None:
        """
        Guarda el gasto, sus eventos y la clave en una única operación atómica.

        Si alguna falla, no debe persistirse ninguna.

        Args:
            gasto (Gasto): Gasto a persistir
            eventos (tuple[EventoCambioUsuario, ...]): Un evento por participante.
                Llegan con posicion=None; la infraestructura asigna la posición.
            registro_idempotencia (RegistroIdempotenciaGasto | None): Registro de
                la clave, o None si la petición no trajo Idempotency-Key.

        Raises:
            ErrorInfraestructura: Fallos gestionados por la capa de infraestructura.
        """
        ...