"""
Ejemplo de prueba para el caso de uso RegistrarUsuario.
Este archivo muestra cómo podría utilizarse el caso de uso en un entorno de prueba.
"""

from aplicacion.registrar_usuario import RegistrarUsuario, RepositorioUsuarios
from dominio.usuario import Usuario
from datetime import datetime
from typing import Optional


class MockRepositorioUsuarios(RepositorioUsuarios):
    """
    Implementación mock del repositorio para pruebas.
    """
    
    def __init__(self):
        self.usuarios = {}
        
    def obtener_usuario_por_nombre_usuario(self, nombre_usuario: str) -> Optional[Usuario]:
        for usuario in self.usuarios.values():
            if usuario.nombre_usuario == nombre_usuario:
                return usuario
        return None
        
    def obtener_usuario_por_correo(self, correo: str) -> Optional[Usuario]:
        for usuario in self.usuarios.values():
            if usuario.correo == correo:
                return usuario
        return None
        
    def guardar_usuario(self, usuario: Usuario) -> None:
        self.usuarios[usuario.id] = usuario


def ejemplo_uso_registrar_usuario():
    """
    Ejemplo de cómo usar el caso de uso RegistrarUsuario.
    """
    # Crear repositorio mock
    repositorio = MockRepositorioUsuarios()
    
    # Crear el caso de uso
    registrar_usuario = RegistrarUsuario(repositorio)
    
    try:
        # Registrar un nuevo usuario
        nuevo_usuario = registrar_usuario.ejecutar(
            nombre_usuario="juanito123",
            nombre_visible="Juan Pérez",
            correo="juan.perez@example.com"
        )
        
        print("Usuario registrado exitosamente:")
        print(f"ID: {nuevo_usuario.id}")
        print(f"Nombre de usuario: {nuevo_usuario.nombre_usuario}")
        print(f"Nombre visible: {nuevo_usuario.nombre_visible}")
        print(f"Correo: {nuevo_usuario.correo}")
        print(f"Fecha de alta: {nuevo_usuario.fecha_alta}")
        
        # Intentar registrar el mismo usuario (debería fallar por unicidad)
        registrar_usuario.ejecutar(
            nombre_usuario="juanito123",
            nombre_visible="Juan Pérez",
            correo="juan.perez@example.com"
        )
        
    except Exception as e:
        print(f"Error al registrar usuario: {e}")


if __name__ == "__main__":
    ejemplo_uso_registrar_usuario()