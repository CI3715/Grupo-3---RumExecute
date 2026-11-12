"""
Implementación del Repositorio de Usuarios y Credenciales en PostgreSQL con SQLAlchemy.
Cumple con RepositorioUsuarios, RepositorioCredenciales y PersistenciaRegistro.
"""
from typing import Optional
from sqlalchemy.orm import Session  # type: ignore
from sqlalchemy.exc import IntegrityError  # type: ignore
from sqlalchemy import func  # type: ignore

from dominio.usuario import Usuario as UsuarioDominio
from dominio.credenciales import Credenciales
from aplicacion.errores import UsuarioYaExistente
from aplicacion.puertos.repositorio_usuarios import RepositorioUsuarios
from aplicacion.puertos.repositorio_credenciales import RepositorioCredenciales
from aplicacion.puertos.persistencia_registro import PersistenciaRegistro

from .modelos_bd import Usuario


class RepositorioUsuariosPostgres(RepositorioUsuarios, RepositorioCredenciales, PersistenciaRegistro):
    def __init__(self, db: Session):
        self.db = db

    def _mapear_a_dominio(self, modelo: Usuario) -> UsuarioDominio:
        return UsuarioDominio(
            id=modelo.id,
            nombre_usuario=modelo.nombre_usuario,
            nombre_visible=modelo.nombre_visible,
            correo=modelo.correo,
            fecha_alta=modelo.creado_en
        )

    def obtener_usuario_por_nombre_usuario(self, nombre_usuario: str) -> Optional[UsuarioDominio]:
        nombre_norm = nombre_usuario.strip().lower()
        registro = self.db.query(Usuario).filter(
            func.lower(Usuario.nombre_usuario) == nombre_norm
        ).first()
        return self._mapear_a_dominio(registro) if registro else None

    def obtener_usuario_por_correo(self, correo: str) -> Optional[UsuarioDominio]:
        correo_norm = correo.strip().lower()
        registro = self.db.query(Usuario).filter(
            func.lower(Usuario.correo) == correo_norm
        ).first()
        return self._mapear_a_dominio(registro) if registro else None

    def obtener_usuario_por_id(self, usuario_id: str) -> Optional[UsuarioDominio]:
        registro = self.db.query(Usuario).filter(Usuario.id == usuario_id).first()
        return self._mapear_a_dominio(registro) if registro else None

    def obtener_por_usuario_id(self, usuario_id: str) -> Optional[Credenciales]:
        registro = self.db.query(Usuario).filter(Usuario.id == usuario_id).first()
        if not registro:
            return None
        return Credenciales(
            usuario_id=registro.id,
            hash_contrasena=registro.password_hash
        )

    def guardar_registro(self, usuario: UsuarioDominio, credenciales: Credenciales) -> None:
        """
        Inserta el usuario y sus credenciales de forma atómica en PostgreSQL.
        Maneja errores de unicidad para levantar UsuarioYaExistente.
        """
        if self.obtener_usuario_por_nombre_usuario(usuario.nombre_usuario):
            raise UsuarioYaExistente("nombre_usuario")
        if self.obtener_usuario_por_correo(usuario.correo):
            raise UsuarioYaExistente("correo")

        nuevo_usuario = Usuario(
            id=usuario.id,
            nombre_usuario=usuario.nombre_usuario.lower(),
            nombre_visible=usuario.nombre_visible,
            correo=usuario.correo.lower(),
            password_hash=credenciales.hash_contrasena,
            creado_en=usuario.fecha_alta
        )

        try:
            self.db.add(nuevo_usuario)
            self.db.commit()
            self.db.refresh(nuevo_usuario)
        except IntegrityError as e:
            self.db.rollback()
            msg = str(e.orig).lower()
            if "nombre_usuario" in msg:
                raise UsuarioYaExistente("nombre_usuario")
            elif "correo" in msg:
                raise UsuarioYaExistente("correo")
            raise
