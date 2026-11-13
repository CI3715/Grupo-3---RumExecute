"""
Modelos ORM de SQLAlchemy para PostgreSQL.
Uso de String(36) para compatibilidad total con UUID v4 en Python y Postgres.
Restricción INV-16: Cero tablas o campos financieros personales en el servidor.
"""
from datetime import datetime, timezone
import uuid
from sqlalchemy import (  # type: ignore
    Column,
    String,
    Text,
    DateTime,
    Numeric,
    Boolean,
    ForeignKey,
    UniqueConstraint,
    Index
)
from .db import Base


class Usuario(Base):
    __tablename__ = "usuarios"

    id = Column(String(36), primary_key=True, default=lambda: str(uuid.uuid4()), index=True)
    nombre_usuario = Column(String(30), unique=True, nullable=False, index=True)
    nombre_visible = Column(String(100), nullable=False)
    correo = Column(String(255), unique=True, nullable=False, index=True)
    password_hash = Column(String(255), nullable=False)
    creado_en = Column(
        DateTime(timezone=True),
        default=lambda: datetime.now(timezone.utc),
        nullable=False
    )


class Amistad(Base):
    __tablename__ = "amistades"

    id = Column(String(36), primary_key=True, default=lambda: str(uuid.uuid4()))
    usuario_origen_id = Column(String(36), ForeignKey("usuarios.id", ondelete="CASCADE"), nullable=False)
    usuario_destino_id = Column(String(36), ForeignKey("usuarios.id", ondelete="CASCADE"), nullable=False)
    estado = Column(String(20), nullable=False)  # PENDIENTE, ACTIVA, RECHAZADA
    creado_en = Column(
        DateTime(timezone=True),
        default=lambda: datetime.now(timezone.utc),
        nullable=False
    )
    actualizado_en = Column(
        DateTime(timezone=True),
        default=lambda: datetime.now(timezone.utc),
        onupdate=lambda: datetime.now(timezone.utc),
        nullable=False
    )

    __table_args__ = (
        UniqueConstraint("usuario_origen_id", "usuario_destino_id", name="uq_amistad_origen_destino"),
        Index("ix_amistades_estado", "estado"),
    )


class Grupo(Base):
    __tablename__ = "grupos"

    id = Column(String(36), primary_key=True, default=lambda: str(uuid.uuid4()))
    nombre = Column(String(100), nullable=False)
    descripcion = Column(Text, nullable=True)
    moneda = Column(String(3), nullable=False)  # ISO 3 letras: USD, EUR
    actividad_fecha = Column(String(100), nullable=True)
    creador_id = Column(String(36), ForeignKey("usuarios.id", ondelete="CASCADE"), nullable=False)
    creado_en = Column(
        DateTime(timezone=True),
        default=lambda: datetime.now(timezone.utc),
        nullable=False
    )


class MiembroGrupo(Base):
    __tablename__ = "miembros_grupo"

    id = Column(String(36), primary_key=True, default=lambda: str(uuid.uuid4()))
    grupo_id = Column(String(36), ForeignKey("grupos.id", ondelete="CASCADE"), nullable=False)
    usuario_id = Column(String(36), ForeignKey("usuarios.id", ondelete="CASCADE"), nullable=False)
    rol = Column(String(20), nullable=False, default="MIEMBRO")  # ADMIN, MIEMBRO
    estado = Column(String(20), nullable=False, default="PENDIENTE")  # PENDIENTE, ACEPTADO
    creado_en = Column(
        DateTime(timezone=True),
        default=lambda: datetime.now(timezone.utc),
        nullable=False
    )

    __table_args__ = (
        UniqueConstraint("grupo_id", "usuario_id", name="uq_grupo_usuario"),
        Index("ix_miembros_estado", "estado"),
    )


class GastoGrupo(Base):
    __tablename__ = "gastos_grupo"

    id = Column(String(36), primary_key=True, default=lambda: str(uuid.uuid4()))
    grupo_id = Column(String(36), ForeignKey("grupos.id", ondelete="CASCADE"), nullable=False, index=True)
    pagador_id = Column(String(36), ForeignKey("usuarios.id", ondelete="CASCADE"), nullable=False, index=True)
    concepto = Column(String(255), nullable=False)
    monto = Column(Numeric(12, 2), nullable=False)  # Decimal exacto (RNF-08)
    fecha = Column(DateTime(timezone=True), nullable=False)
    regla_division = Column(String(20), nullable=False)  # IGUALES, PORCENTAJES, MONTOS_FIJOS
    creado_en = Column(
        DateTime(timezone=True),
        default=lambda: datetime.now(timezone.utc),
        nullable=False
    )


class ParteGasto(Base):
    __tablename__ = "partes_gasto"

    id = Column(String(36), primary_key=True, default=lambda: str(uuid.uuid4()))
    gasto_id = Column(String(36), ForeignKey("gastos_grupo.id", ondelete="CASCADE"), nullable=False)
    usuario_id = Column(String(36), ForeignKey("usuarios.id", ondelete="CASCADE"), nullable=False)
    monto_cuota = Column(Numeric(12, 2), nullable=False)
    porcentaje_indicado = Column(Numeric(5, 2), nullable=True)
    monto_fijo_indicado = Column(Numeric(12, 2), nullable=True)
    lleva_residuo = Column(Boolean, nullable=False, default=False)

    __table_args__ = (
        UniqueConstraint("gasto_id", "usuario_id", name="uq_gasto_usuario"),
    )
