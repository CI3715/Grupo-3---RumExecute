"""
Implementación del Repositorio de Eventos de Cambio en PostgreSQL con SQLAlchemy.
Permite consultar eventos posteriores a un cursor para la sincronización unidireccional (Subtarea 3.1).
"""
from typing import List
from sqlalchemy.orm import Session  # type: ignore

from dominio.evento_cambio import EventoCambioUsuario
from aplicacion.puertos.repositorio_eventos_cambio import RepositorioEventosCambio
from .modelos_bd import EventoCambioUsuarioModel


class RepositorioEventosCambioPostgres(RepositorioEventosCambio):
    def __init__(self, db: Session):
        self.db = db

    def listar_posteriores(
        self,
        usuario_id: str,
        desde: int,
        limite: int,
    ) -> List[EventoCambioUsuario]:
        """
        Devuelve los eventos de un usuario con posicion mayor a 'desde'.
        Ordenados ascendentemente por posicion y con un maximo de 'limite' registros.
        """
        registros = (
            self.db.query(EventoCambioUsuarioModel)
            .filter(
                EventoCambioUsuarioModel.usuario_id == usuario_id,
                EventoCambioUsuarioModel.posicion > desde,
            )
            .order_by(EventoCambioUsuarioModel.posicion.asc())
            .limit(limite)
            .all()
        )

        return [
            EventoCambioUsuario(
                id=r.id,
                posicion=r.posicion,
                usuario_id=r.usuario_id,
                grupo_id=r.grupo_id,
                nombre_grupo=r.nombre_grupo,
                gasto_id=r.gasto_id,
                concepto=r.concepto,
                fecha=r.fecha,
                moneda=r.moneda,
                monto_parte=r.monto_parte,
                incluye_redondeo=r.incluye_redondeo,
            )
            for r in registros
        ]
