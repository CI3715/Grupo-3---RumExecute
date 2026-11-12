"""
Contenedor de dependencias e inyección para la capa API con PostgreSQL.
Instancia los casos de uso inyectando la sesión de BD de SQLAlchemy y los adaptadores.
"""
from fastapi import Depends  # type: ignore
from sqlalchemy.orm import Session  # type: ignore

from infraestructura.db import get_db
from infraestructura.repositorio_usuarios_postgres import RepositorioUsuariosPostgres
from infraestructura.adaptadores_servicios import (
    AdaptadorServicioHash,
    AdaptadorServicioTokens,
)
from aplicacion.registrar_usuario import RegistrarUsuario
from aplicacion.iniciar_sesion import IniciarSesion
from aplicacion.obtener_perfil_propio import ObtenerPerfilPropio

servicio_hash_global = AdaptadorServicioHash()
servicio_tokens_global = AdaptadorServicioTokens()


def caso_uso_registrar_usuario(db: Session = Depends(get_db)) -> RegistrarUsuario:
    repo = RepositorioUsuariosPostgres(db)
    return RegistrarUsuario(
        repositorio_usuarios=repo,
        persistencia_registro=repo,
        servicio_hash=servicio_hash_global,
    )


def caso_uso_iniciar_sesion(db: Session = Depends(get_db)) -> IniciarSesion:
    repo = RepositorioUsuariosPostgres(db)
    return IniciarSesion(
        repositorio_usuarios=repo,
        repositorio_credenciales=repo,
        servicio_hash=servicio_hash_global,
        servicio_tokens=servicio_tokens_global,
    )


def caso_uso_obtener_perfil_propio(db: Session = Depends(get_db)) -> ObtenerPerfilPropio:
    repo = RepositorioUsuariosPostgres(db)
    return ObtenerPerfilPropio(
        repositorio_usuarios=repo,
        servicio_tokens=servicio_tokens_global,
    )
