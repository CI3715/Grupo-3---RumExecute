"""
Contenedor de dependencias e inyección para la capa API con PostgreSQL.
Instancia los casos de uso inyectando la sesión de BD de SQLAlchemy y los adaptadores.
"""
from fastapi import Depends  # type: ignore
from sqlalchemy.orm import Session  # type: ignore

from infraestructura.db import get_db
from infraestructura.repositorio_usuarios_postgres import RepositorioUsuariosPostgres
from infraestructura.repositorio_amistades_postgres import RepositorioAmistadesPostgres
from infraestructura.repositorio_grupos_postgres import RepositorioGruposPostgres
from infraestructura.repositorio_gastos_postgres import RepositorioGastosPostgres
from infraestructura.adaptadores_servicios import (
    AdaptadorServicioHash,
    AdaptadorServicioTokens,
)

# Casos de uso de autenticación (Entrega 5)
from aplicacion.registrar_usuario import RegistrarUsuario
from aplicacion.iniciar_sesion import IniciarSesion
from aplicacion.obtener_perfil_propio import ObtenerPerfilPropio

# Casos de uso de amistades y grupos (Entrega 6)
from aplicacion.buscar_usuario import BuscarUsuario
from aplicacion.enviar_invitacion_amistad import EnviarInvitacionAmistad
from aplicacion.responder_invitacion_amistad import ResponderInvitacionAmistad
from aplicacion.listar_amistades import ListarAmistades
from aplicacion.eliminar_amistad import EliminarAmistad
from aplicacion.crear_grupo import CrearGrupo
from aplicacion.listar_mis_grupos import ListarMisGrupos
from aplicacion.invitar_miembro_grupo import InvitarMiembroGrupo
from aplicacion.responder_invitacion_grupo import ResponderInvitacionGrupo
from aplicacion.registrar_gasto_compartido import RegistrarGastoCompartido
from aplicacion.obtener_detalle_grupo import ObtenerDetalleGrupo

servicio_hash_global = AdaptadorServicioHash()
servicio_tokens_global = AdaptadorServicioTokens()


# --- Factorías para Auth (Entrega 5) ---

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


# --- Factorías para Amistades (Entrega 6) ---

def caso_uso_buscar_usuario(db: Session = Depends(get_db)) -> BuscarUsuario:
    return BuscarUsuario(repositorio_usuarios=RepositorioUsuariosPostgres(db))


def caso_uso_enviar_invitacion_amistad(db: Session = Depends(get_db)) -> EnviarInvitacionAmistad:
    return EnviarInvitacionAmistad(
        repositorio_usuarios=RepositorioUsuariosPostgres(db),
        repositorio_amistades=RepositorioAmistadesPostgres(db),
    )


def caso_uso_responder_invitacion_amistad(db: Session = Depends(get_db)) -> ResponderInvitacionAmistad:
    return ResponderInvitacionAmistad(
        repositorio_amistades=RepositorioAmistadesPostgres(db),
    )


def caso_uso_listar_amistades(db: Session = Depends(get_db)) -> ListarAmistades:
    return ListarAmistades(
        repositorio_usuarios=RepositorioUsuariosPostgres(db),
        repositorio_amistades=RepositorioAmistadesPostgres(db),
    )


def caso_uso_eliminar_amistad(db: Session = Depends(get_db)) -> EliminarAmistad:
    return EliminarAmistad(
        repositorio_amistades=RepositorioAmistadesPostgres(db),
    )


# --- Factorías para Grupos y Gastos (Entrega 6) ---

def caso_uso_crear_grupo(db: Session = Depends(get_db)) -> CrearGrupo:
    return CrearGrupo(
        repositorio_usuarios=RepositorioUsuariosPostgres(db),
        repositorio_grupos=RepositorioGruposPostgres(db),
    )


def caso_uso_listar_mis_grupos(db: Session = Depends(get_db)) -> ListarMisGrupos:
    return ListarMisGrupos(
        repositorio_usuarios=RepositorioUsuariosPostgres(db),
        repositorio_grupos=RepositorioGruposPostgres(db),
    )


def caso_uso_invitar_miembro_grupo(db: Session = Depends(get_db)) -> InvitarMiembroGrupo:
    return InvitarMiembroGrupo(
        repositorio_usuarios=RepositorioUsuariosPostgres(db),
        repositorio_grupos=RepositorioGruposPostgres(db),
        repositorio_amistades=RepositorioAmistadesPostgres(db),
    )


def caso_uso_responder_invitacion_grupo(db: Session = Depends(get_db)) -> ResponderInvitacionGrupo:
    return ResponderInvitacionGrupo(
        repositorio_grupos=RepositorioGruposPostgres(db),
    )


def caso_uso_registrar_gasto_compartido(db: Session = Depends(get_db)) -> RegistrarGastoCompartido:
    return RegistrarGastoCompartido(
        repositorio_usuarios=RepositorioUsuariosPostgres(db),
        repositorio_grupos=RepositorioGruposPostgres(db),
        repositorio_gastos=RepositorioGastosPostgres(db),
    )


def caso_uso_obtener_detalle_grupo(db: Session = Depends(get_db)) -> ObtenerDetalleGrupo:
    return ObtenerDetalleGrupo(
        repositorio_usuarios=RepositorioUsuariosPostgres(db),
        repositorio_grupos=RepositorioGruposPostgres(db),
        repositorio_gastos=RepositorioGastosPostgres(db),
    )
