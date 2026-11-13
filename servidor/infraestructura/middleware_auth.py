"""
Middleware y Dependencias de Autenticación de FastAPI.
Valida el encabezado 'Authorization: Bearer <token>' en rutas protegidas.
Responde con 401 Unauthorized si el token está ausente, alterado o vencido.
"""
from typing import Dict, Any, Optional
from fastapi import Request, HTTPException, status, Depends
from fastapi.security import HTTPBearer, HTTPAuthorizationCredentials
import jwt  # type: ignore

from .jwt_servicio import _obtener_secreto, ALGORITMO

security = HTTPBearer(auto_error=False)


async def obtener_usuario_actual(
    credenciales: Optional[HTTPAuthorizationCredentials] = Depends(security),
    request: Request = None
) -> Dict[str, Any]:
    """
    Dependencia de FastAPI para inyectar en rutas protegidas.
    
    Verifica:
    1. Si el header Authorization: Bearer existe.
    2. Si el token no está vencido ni alterado.
    
    Lanza HTTPException(401) en caso de fallo, con el mensaje esperado.
    """
    token = None
    if credenciales:
        token = credenciales.credentials
    elif request:
        auth_header = request.headers.get("Authorization")
        if auth_header and auth_header.startswith("Bearer "):
            token = auth_header.split(" ")[1]

    if not token:
        raise HTTPException(
            status_code=status.HTTP_401_UNAUTHORIZED,
            detail="Token ausente o inválido.",
            headers={"WWW-Authenticate": "Bearer"},
        )

    try:
        payload = jwt.decode(token, _obtener_secreto(), algorithms=[ALGORITMO])
        return payload
    except jwt.ExpiredSignatureError:
        raise HTTPException(
            status_code=status.HTTP_401_UNAUTHORIZED,
            detail="El token de sesión ha expirado.",
            headers={"WWW-Authenticate": "Bearer"},
        )
    except (jwt.InvalidTokenError, jwt.PyJWTError):
        raise HTTPException(
            status_code=status.HTTP_401_UNAUTHORIZED,
            detail="Token ausente, alterado o inválido.",
            headers={"WWW-Authenticate": "Bearer"},
        )

