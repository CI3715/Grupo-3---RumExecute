from typing import Protocol

from dominio.credenciales import Credenciales


class RepositorioCredenciales(Protocol):
    """
    Puerto para acceder a las credenciales almacenadas.
    """

    def obtener_por_usuario_id(
            self,
            usuario_id: str
    ) -> Credenciales | None:
        ...