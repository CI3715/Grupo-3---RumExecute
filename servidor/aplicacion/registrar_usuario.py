from datetime import datetime, timezone
import uuid

from aplicacion.puertos.repositorio_usuarios import RepositorioUsuarios
from aplicacion.puertos.persistencia_registro import PersistenciaRegistro
from aplicacion.puertos.servicio_hash_contrasenas import ServicioHashContrasenas

from dominio.usuario import Usuario
from dominio.credenciales import Credenciales
from dominio.errores import ErrorDominio


class RegistrarUsuario:
    """
    Caso de uso para registrar un nuevo usuario.
    """

    def __init__(
            self,
            repositorio_usuarios: RepositorioUsuarios,
            persistencia_registro: PersistenciaRegistro,
            servicio_hash: ServicioHashContrasenas):
        """
        Inicializa el caso de uso RegistrarUsuario.
        
        Args:
            repositorio_usuarios (RepositorioUsuarios): Implementación del 
                repositorio de usuarios para almacenar los datos
        """
        self.repositorio_usuarios = repositorio_usuarios
        self.persistencia_registro = persistencia_registro
        self.servicio_hash = servicio_hash

    def ejecutar(
            self,
            nombre_usuario: str,
            nombre_visible: str,
            correo: str,
            contrasena: str
        ) -> Usuario:
        """
        Ejecuta el caso de uso para registrar un nuevo usuario.
        
        Args:
            nombre_usuario (str): Nombre de usuario que el usuario desea usar
            nombre_visible (str): Nombre visible del usuario
            correo (str): Correo electrónico del usuario
            contrasena (str): Contraseña del usuario

        Returns:
            Usuario: El usuario creado, sin contraseña ni hash
            
        Raises:
            ErrorDominio: Si se encuentran errores de validación o si 
                el nombre de usuario o correo ya existen
        """

        correo = correo.strip().lower()

        self._validar_contrasena(contrasena)
        
        # Verificar unicidad del nombre de usuario y correo
        self._verificar_unicidad(nombre_usuario, correo)
        
        # Crear el usuario
        usuario = self._crear_usuario(
            nombre_usuario,
            nombre_visible,
            correo
        )

        hash_contrasena = self.servicio_hash.generar_hash(contrasena)

        credenciales = Credenciales(
            usuario_id=usuario.id,
            hash_contrasena=hash_contrasena
        )
        
        # Guardar en el repositorio
        self.persistencia_registro.guardar_registro(
            usuario,
            credenciales
        )
        
        return usuario

    def _validar_contrasena(self, contrasena: str) -> None:
        if len(contrasena) < 10:
            raise ErrorDominio(
                "contrasena",
                "La contraseña debe tener al menos 10 caracteres."
            )

    def _verificar_unicidad(
            self,
            nombre_usuario: str,
            correo: str
        ) -> None:
        """
        Verifica que el nombre de usuario y correo sean únicos.
        
        Args:
            nombre_usuario (str): Nombre de usuario a verificar
            correo (str): Correo a verificar
            
        Raises:
            ErrorDominio: Si el nombre de usuario o correo ya existen
        """
        # Verificar si el nombre de usuario ya existe
        usuario_existente = (
            self.repositorio_usuarios.
            obtener_usuario_por_nombre_usuario(nombre_usuario)
        )

        if usuario_existente is not None:
            raise ErrorDominio(
                "nombre_usuario",
                "El nombre de usuario ya está en uso."
            )
            
        # Verificar si el correo ya existe
        correo_existente = (
            self.repositorio_usuarios.
            obtener_usuario_por_correo(correo)
        )

        if correo_existente is not None:
            raise ErrorDominio(
                "correo",
                "El correo electrónico ya está en uso."
            )

    def _crear_usuario(
            self,
            nombre_usuario: str,
            nombre_visible: str,
            correo: str) -> Usuario:
        """
        Crea una nueva instancia de Usuario.
        
        Args:
            nombre_usuario (str): Nombre de usuario
            nombre_visible (str): Nombre visible
            correo (str): Correo electrónico
            
        Returns:
            Usuario: Instancia de Usuario creada
        """
        # Generar un ID único para el nuevo usuario
        usuario_id = str(uuid.uuid4())
        
        # Crear la fecha de alta
        fecha_alta = datetime.now(timezone.utc)
        
        # Creamos el usuario, lo cual invocará la validación del __post_init__
        return Usuario(
            id=usuario_id,
            nombre_usuario=nombre_usuario,
            nombre_visible=nombre_visible,
            correo=correo,
            fecha_alta=fecha_alta
        )