"""
Implementación del Repositorio de Claves de Idempotencia en PostgreSQL con SQLAlchemy.
Cumple con aplicacion.puertos.repositorio_claves_idempotencia.RepositorioClavesIdempotencia.
"""
from typing import Optional
from sqlalchemy.orm import Session  # type: ignore

from dominio.idempotencia import RegistroIdempotenciaGasto
from aplicacion.puertos.repositorio_claves_idempotencia import RepositorioClavesIdempotencia
from .modelos_bd import ClaveIdempotenciaGasto


class RepositorioIdempotenciaPostgres(RepositorioClavesIdempotencia):
    def __init__(self, db: Session):
        self.db = db

    def buscar(self, usuario_id: str, clave: str) -> Optional[RegistroIdempotenciaGasto]:
        registro = self.db.query(ClaveIdempotenciaGasto).filter(
            ClaveIdempotenciaGasto.usuario_id == usuario_id,
            ClaveIdempotenciaGasto.clave == clave,
        ).first()

        if not registro:
            return None

        return RegistroIdempotenciaGasto(
            clave=registro.clave,
            usuario_id=registro.usuario_id,
            huella=registro.huella,
            gasto_id=registro.gasto_id,
        )
