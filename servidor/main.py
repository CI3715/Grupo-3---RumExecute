from datetime import datetime, timezone
from fastapi import FastAPI
from fastapi.middleware.cors import CORSMiddleware
from pydantic import BaseModel, Field

# 1. Definición del Esquema Pydantic (Modelo de Respuesta)
class PingResponse(BaseModel):
    estado: str = Field(..., examples=["ok"])
    mensaje: str = Field(..., examples=["Servidor Cuentas Claras activo"])
    version: str = Field(..., examples=["0.1.0"])
    timestamp: str = Field(..., examples=["2026-10-07T20:00:00Z"])


# 2. Inicialización de la Aplicación FastAPI
app = FastAPI(
    title="Cuentas Claras API",
    description="Servidor Backend para la aplicación de finanzas personales Cuentas Claras",
    version="0.1.0",
)

# 3. Configuración del Middleware CORS
app.add_middleware(
    CORSMiddleware,
    allow_origins=["*"],  # Permite peticiones locales desde la app Tauri / Next.js
    allow_credentials=True,
    allow_methods=["*"],  # Permite todos los métodos HTTP
    allow_headers=["*"],  # Permite todas las cabeceras HTTP
)


# 4. Definición del Endpoint GET /ping
@app.get("/ping", response_model=PingResponse, status_code=200)
def ping() -> PingResponse:
    """
    Endpoint de prueba (Ping Flow) para verificar la comunicación
    entre la aplicación cliente y el servidor backend.
    """
    actual_timestamp = (
        datetime.now(timezone.utc).isoformat().replace("+00:00", "Z")
    )

    return PingResponse(
        estado="ok",
        mensaje="Servidor Cuentas Claras activo",
        version="0.1.0",
        timestamp=actual_timestamp,
    )
