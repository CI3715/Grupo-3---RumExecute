from typing import Protocol, Optional

from dominio.usuario import Usuario


class RepositorioUsuarios(Protocol):
    """
    Interfaz del repositorio de usuarios para la capa de aplicación.
    Esta interfaz será implementada por la capa de infraestructura.
    """
    
    def obtener_usuario_por_nombre_usuario(self, nombre_usuario: str) -> Usuario | None:
        """
        Obtiene un usuario por su nombre de usuario.
        
        Args:
            nombre_usuario (str): Nombre de usuario a buscar
            
        Returns:
            Usuario o None si no se encuentra
        """
        ...

    def obtener_usuario_por_correo(self, correo: str) -> Usuario | None:
        """
        Obtiene un usuario por su correo.
        
        Args:
            correo (str): Correo a buscar
            
        Returns:
            Usuario o None si no se encuentra
        """
        ...
    
    def obtener_usuario_por_id(self, usuario_id: str) -> Usuario | None:
        """
        Obtiene un usuario por su ID.
        
        Args:
            usuario_id (str): ID del usuario a buscar
            
        Returns:
            Usuario o None si no se encuentra
        """
        ...