from typing import Protocol

from dominio.usuario import Usuario
from dominio.credenciales import Credenciales

class PersistenciaRegistro(Protocol):
    """
    Puerto para persistir atómicamente un usuario y sus credenciales.
    """

    def guardar_registro(
            self,
            usuario: Usuario,
            credenciales: Credenciales
    ) -> None:
        """
        Guarda ambas entidades en una única operación atómica.

        Si alguna falla, no debe persistirse ninguna de las dos.

        Args:
            usuario (Usuario): Usuario a persistir
            credenciales (Credenciales): Credenciales a persistir
        """
        ...