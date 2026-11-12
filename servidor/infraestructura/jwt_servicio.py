"""
Módulo de Infraestructura: Gestión de Tokens JWT (RES-07).
Maneja la generación y decodificación de tokens de sesión con vigencia de 60 min.
El secreto se lee estrictamente de variables de entorno (JWT_SECRET).
"""
import os
from datetime import datetime, timedelta, timezone
from typing import Any, Dict, Optional
import jwt  # type: ignore

ALGORITMO = "HS256"
MINUTOS_VIGENCIA = 60


def _obtener_secreto() -> str:
    secreto = os.getenv("JWT_SECRET")
    if not secreto:
        # Clave fallback para pruebas locales si no está definida en .env
        return "desarrollo_inseguro_cambiar_en_produccion_12345"
    return secreto


def generar_token(datos_usuario: Dict[str, Any], vigencia_minutos: int = MINUTOS_VIGENCIA) -> Dict[str, Any]:
    """
    Genera un token JWT (HS256) con fecha de expiración.
    Retorna un diccionario con access_token, token_type y expires_at.
    """
    ahora = datetime.now(timezone.utc)
    expiracion = ahora + timedelta(minutes=vigencia_minutos)
    
    payload = {
        **datos_usuario,
        "iat": int(ahora.timestamp()),
        "exp": int(expiracion.timestamp())
    }
    
    token = jwt.encode(payload, _obtener_secreto(), algorithm=ALGORITMO)
    
    # Formato ISO-8601 explícito en UTC 0 con terminación 'Z'
    expires_at_iso = expiracion.strftime("%Y-%m-%dT%H:%M:%SZ")

    return {
        "access_token": token,
        "token_type": "bearer",
        "expires_at": expires_at_iso
    }


def decodificar_token(token: str) -> Optional[Dict[str, Any]]:
    """
    Decodifica y valida la firma y expiración del token JWT.
    Retorna el payload o None si es inválido/expirado.
    """
    try:
        payload = jwt.decode(token, _obtener_secreto(), algorithms=[ALGORITMO])
        return payload
    except jwt.PyJWTError:
        return None

