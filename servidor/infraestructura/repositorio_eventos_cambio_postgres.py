"""
Implementación del Repositorio de Eventos de Cambio en PostgreSQL con SQLAlchemy.
Permite consultar eventos posteriores a un cursor para la sincronización unidireccional (Subtarea 3.1).
"""
from typing import List, Tuple
from sqlalchemy.orm import Session  # type: ignore

from dominio.evento_cambio import EventoCambioUsuario
from aplicacion.puertos.repositorio_eventos_cambio import RepositorioEventosCambio
from .modelos_bd import EventoCambioUsuarioModel


class RepositorioEventosCambioPostgres(RepositorioEventosCambio):
    def __init__(self, db: Session):
        self.db = db

    def consultar_cambios_usuario(
        self,
        usuario_id: str,
        desde_cursor: int,
        limite: int = 100,
    ) -> Tuple[List[EventoCambioUsuario], int, bool]:
        """
        Consulta eventos de cambio del usuario posteriores al cursor.
        Ordenados estrictamente por posicion ASC en lotes de maximo `limite`.
        Retorna (eventos, nuevo_cursor, quedan_mas).
        """
        # Consultamos limite + 1 para saber si quedan mas registros pendientes
        registros = (
            self.db.query(EventoCambioUsuarioModel)
            .filter(
                EventoCambioUsuarioModel.usuario_id == usuario_id,
                EventoCambioUsuarioModel.posicion > desde_cursor,
            )
            .order_by(EventoCambioUsuarioModel.posicion.asc())
            .limit(limite + 1)
            .all()
        )

        quedan_mas = len(registros) > limite
        lote = registros[:limite]

        eventos = [
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
            for r in lote
        ]

        nuevo_cursor = lote[-1].posicion if lote else desde_cursor

        return eventos, nuevo_cursor, quedan_mas
