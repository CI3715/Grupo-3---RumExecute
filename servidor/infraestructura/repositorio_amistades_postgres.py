"""
Implementación del Repositorio de Amistades en PostgreSQL con SQLAlchemy.
Cumple con aplicacion.puertos.repositorio_amistades.RepositorioAmistades.
"""
from typing import Optional
from sqlalchemy.orm import Session  # type: ignore
from sqlalchemy import or_, and_  # type: ignore

from dominio.amistad import Amistad as AmistadDominio, EstadoAmistad
from dominio.errores import ErrorConflicto, ErrorPermiso
from aplicacion.puertos.repositorio_amistades import RepositorioAmistades

from .modelos_bd import Amistad


class RepositorioAmistadesPostgres(RepositorioAmistades):
    def __init__(self, db: Session):
        self.db = db

    def _mapear_a_dominio(self, modelo: Amistad) -> AmistadDominio:
        estado_map = {
            "PENDIENTE": EstadoAmistad.PENDIENTE,
            "ACTIVA": EstadoAmistad.ACTIVA,
            "RECHAZADA": EstadoAmistad.RECHAZADA,
        }
        return AmistadDominio(
            id=modelo.id,
            id_solicitante=modelo.usuario_origen_id,
            id_receptor=modelo.usuario_destino_id,
            estado=estado_map.get(modelo.estado, EstadoAmistad.PENDIENTE),
        )

    def listar_entre_usuarios(self, usuario_a_id: str, usuario_b_id: str) -> list[AmistadDominio]:
        registros = self.db.query(Amistad).filter(
            or_(
                and_(Amistad.usuario_origen_id == usuario_a_id, Amistad.usuario_destino_id == usuario_b_id),
                and_(Amistad.usuario_origen_id == usuario_b_id, Amistad.usuario_destino_id == usuario_a_id),
            )
        ).all()
        return [self._mapear_a_dominio(r) for r in registros]

    def guardar_invitacion(self, amistad: AmistadDominio) -> None:
        existentes = self.listar_entre_usuarios(amistad.id_solicitante, amistad.id_receptor)
        for ex in existentes:
            if ex.estado == EstadoAmistad.PENDIENTE:
                raise ErrorConflicto("receptor", "Ya existe una solicitud de amistad pendiente con este usuario.")
            if ex.estado == EstadoAmistad.ACTIVA:
                raise ErrorConflicto("receptor", "Ya hay una amistad activa con este usuario.")

        nuevo = Amistad(
            id=amistad.id,
            usuario_origen_id=amistad.id_solicitante,
            usuario_destino_id=amistad.id_receptor,
            estado=amistad.estado.name,
        )
        self.db.add(nuevo)
        self.db.commit()

    def obtener_por_id(self, amistad_id: str) -> Optional[AmistadDominio]:
        registro = self.db.query(Amistad).filter(Amistad.id == amistad_id).first()
        return self._mapear_a_dominio(registro) if registro else None

    def listar_de_usuario(self, usuario_id: str) -> list[AmistadDominio]:
        registros = self.db.query(Amistad).filter(
            or_(Amistad.usuario_origen_id == usuario_id, Amistad.usuario_destino_id == usuario_id),
            Amistad.estado != "RECHAZADA"
        ).all()
        return [self._mapear_a_dominio(r) for r in registros]

    def actualizar_respuesta(self, amistad: AmistadDominio) -> None:
        registro = self.db.query(Amistad).filter(Amistad.id == amistad.id).with_for_update().first()
        if not registro or registro.estado != "PENDIENTE":
            raise ErrorConflicto("amistad", "La invitación ya fue respondida.")

        registro.estado = amistad.estado.name
        self.db.commit()

    def eliminar_amistad(self, amistad_id: str, usuario_id: str) -> bool:
        registro = self.db.query(Amistad).filter(Amistad.id == amistad_id).first()
        if not registro or registro.estado != "ACTIVA":
            return False

        if usuario_id not in (registro.usuario_origen_id, registro.usuario_destino_id):
            raise ErrorPermiso("usuario", "No tienes permiso para eliminar esta amistad.")

        self.db.delete(registro)
        self.db.commit()
        return True

