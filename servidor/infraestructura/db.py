"""
Módulo de Infraestructura: Configuración de Base de Datos PostgreSQL con SQLAlchemy.
Lee la URL de conexión desde variables de entorno (DATABASE_URL).
"""
import os
from sqlalchemy import create_engine  # type: ignore
from sqlalchemy.orm import declarative_base, sessionmaker  # type: ignore

# Si no está definida en el entorno, usa la configurada en docker-compose
DATABASE_URL = os.getenv(
    "DATABASE_URL",
    "postgresql://postgres:password123@localhost:5433/cuentas_claras"
)

# Motor de conexión con pool_pre_ping para reconectar si la BD se reinicia
engine = create_engine(DATABASE_URL, echo=False, pool_pre_ping=True)

# Fábrica de sesiones para transacciones
SessionLocal = sessionmaker(autocommit=False, autoflush=False, bind=engine)

# Clase base declarativa para los modelos ORM
Base = declarative_base()


def get_db():
    """
    Generador de sesiones de base de datos para FastAPI.
    Garantiza que la sesión se cierre al finalizar cada petición HTTP.
    """
    db = SessionLocal()
    try:
        yield db
    finally:
        db.close()

