from typing import Protocol

class ServicioHashContrasenas(Protocol):
    """
    Interfaz para el servicio de hashing de contraseñas.
    Proporciona métodos para generar hashes y verificar contraseñas.
    """

    def generar_hash(self, contrasena: str) -> str:
        """
        Genera un hash a partir de una contraseña.
        
        Args:
            contrasena: La contraseña que se desea hashear.

        Returns:
            El hash generado a partir de la contraseña.
        """
        ...

    def verificar(self, contrasena: str, hash_almacenado: str) -> bool:
        """
        Verifica si una contraseña coincide con un hash almacenado.

        Args:
            contrasena: La contraseña que se desea verificar.
            hash_almacenado: El hash con el que se desea comparar la contraseña.

        Returns:
            True si la contraseña coincide con el hash; False en caso contrario.
        """
        ...