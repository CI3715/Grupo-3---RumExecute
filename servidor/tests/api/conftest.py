"""
Configuración de fixtures para pruebas de integración de la API con PostgreSQL en Docker.
Utiliza el contenedor PostgreSQL especificado en docker-compose.yml.
Aísla cada prueba limpiando las tablas mediante TRUNCATE CASCADE para asegurar
reproducibilidad e independencia absoluta entre ejecuciones.
"""
import os
import pytest
from fastapi.testclient import TestClient
from sqlalchemy import create_engine, text
from sqlalchemy.orm import sessionmaker

from main import app
from infraestructura.db import Base, get_db
import infraestructura.modelos_bd  # Registrar todos los modelos en Base


# URL de PostgreSQL en Docker (puerto 5433 según docker-compose y .env)
DATABASE_URL = os.getenv(
    "DATABASE_URL",
    "postgresql://postgres:password123@localhost:5433/cuentas_claras"
)

engine_test = create_engine(DATABASE_URL, pool_pre_ping=True)
TestingSessionLocal = sessionmaker(autocommit=False, autoflush=False, bind=engine_test)


@pytest.fixture(scope="session", autouse=True)
def setup_database_schema():
    """Asegura que todas las tablas existan en PostgreSQL antes de correr la suite."""
    Base.metadata.create_all(bind=engine_test)
    yield


@pytest.fixture(scope="function")
def db_session():
    """
    Sesión de base de datos PostgreSQL aislada para cada test.
    Limpia las tablas con TRUNCATE CASCADE antes y después de cada prueba.
    """
    def limpiar_tablas():
        with engine_test.begin() as conn:
            conn.execute(
                text(
                    "TRUNCATE TABLE partes_gasto, gastos_grupo, miembros_grupo, grupos, amistades, usuarios RESTART IDENTITY CASCADE;"
                )
            )

    limpiar_tablas()
    db = TestingSessionLocal()
    try:
        yield db
    finally:
        db.close()
        limpiar_tablas()


@pytest.fixture(scope="function")
def client(db_session):
    """Cliente HTTP de prueba con la base de datos PostgreSQL sobreescrita."""
    def override_get_db():
        try:
            yield db_session
        finally:
            pass

    app.dependency_overrides[get_db] = override_get_db
    with TestClient(app) as test_client:
        yield test_client
    app.dependency_overrides.clear()
