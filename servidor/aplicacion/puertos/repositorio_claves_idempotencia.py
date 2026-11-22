from typing import Protocol

from dominio.idempotencia import RegistroIdempotenciaGasto

class RepositorioClavesIdempotencia (Protocol):
    """
    Puerto para consultar las claves de idempotencia ya procesadas
    """

    def buscar(
            self,
            usuario_id: str,
            clave: str
    ) -> RegistroIdempotenciaGasto | None:
        """
        Busca el registro de una clave de idempotencia de un usuario.

        La vlace solo es única dentro de un usuario: la misma clave
        enviada por otro usuario no debe coincidir.

        Args: 
            usuario_id (str): Identificador del usuario que envió la clave
            clave (str): Valor del encabezado Idempotency-Key
        
        Returns:
            RegistroIdempotenciaGasto | None: El registro si la clave ya
                fue procesada; None si es nueva.
        
        Raises:
            ErrorInfraestructura: Fallos gestionados por la capa de infraestructura.
        """
