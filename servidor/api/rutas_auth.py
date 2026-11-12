"""
Router de Autenticación y Usuarios para FastAPI (Entrega 5).
Endpoints requeridos:
- POST /usuarios: Registro de usuario (201 Created)
- POST /sesiones: Inicio de sesión (200 OK)
- GET /usuarios/yo: Perfil propio con Token Bearer (200 OK)
"""
from typing import Optional
from fastapi import APIRouter, Depends, HTTPException, status, Header
from pydantic import BaseModel, Field

from dominio.errores import ErrorDominio
from aplicacion.errores import (
    UsuarioYaExistente,
    ContrasenaInvalida,
    CredencialesInvalidas,
    TokenInvalido,
)
from api.dependencias import (
    caso_uso_registrar_usuario,
    caso_uso_iniciar_sesion,
    caso_uso_obtener_perfil_propio,
)
from aplicacion.registrar_usuario import RegistrarUsuario
from aplicacion.iniciar_sesion import IniciarSesion
from aplicacion.obtener_perfil_propio import ObtenerPerfilPropio

router = APIRouter(tags=["Autenticación"])


# --- Modelos de Entrada / Salida (DTOs HTTP) ---

class RegistroRequest(BaseModel):
    nombre_usuario: str = Field(..., examples=["juanito123"])
    nombre_visible: str = Field(..., examples=["Juan Pérez"])
    correo: str = Field(..., examples=["juan@example.com"])
    contrasena: str = Field(..., min_length=1, examples=["MiClaveSegura123"])


class UsuarioResponse(BaseModel):
    id: str
    nombre_usuario: str
    nombre_visible: str
    correo: str
    fecha_alta: str


class LoginRequest(BaseModel):
    correo: str = Field(..., examples=["juan@example.com"])
    contrasena: str = Field(..., examples=["MiClaveSegura123"])


class SesionResponse(BaseModel):
    access_token: str
    token_type: str = "bearer"
    expires_at: str
    usuario: UsuarioResponse


# --- Endpoints ---

@router.post(
    "/usuarios",
    status_code=status.HTTP_201_CREATED,
    response_model=UsuarioResponse,
    summary="Registrar un nuevo usuario"
)
def registrar_usuario(
    payload: RegistroRequest,
    caso_uso: RegistrarUsuario = Depends(caso_uso_registrar_usuario),
):
    """
    RF-SOC-01: Registro de usuario.
    - 201 Created: Usuario creado exitosamente (sin contraseña ni hash).
    - 400 Bad Request: Contraseña corta (<10), correo mal formado o campos inválidos.
    - 409 Conflict: Nombre de usuario o correo ya registrado (indica cuál de los dos).
    """
    try:
        usuario = caso_uso.ejecutar(
            nombre_usuario=payload.nombre_usuario,
            nombre_visible=payload.nombre_visible,
            correo=payload.correo,
            contrasena=payload.contrasena,
        )
        return UsuarioResponse(
            id=usuario.id,
            nombre_usuario=usuario.nombre_usuario,
            nombre_visible=usuario.nombre_visible,
            correo=usuario.correo,
            fecha_alta=usuario.fecha_alta.strftime("%Y-%m-%dT%H:%M:%SZ"),
        )
    except ContrasenaInvalida as e:
        raise HTTPException(
            status_code=status.HTTP_400_BAD_REQUEST,
            detail={"campo": e.campo, "mensaje": e.mensaje},
        )
    except ErrorDominio as e:
        raise HTTPException(
            status_code=status.HTTP_400_BAD_REQUEST,
            detail={"campo": e.campo, "mensaje": e.mensaje},
        )
    except UsuarioYaExistente as e:
        raise HTTPException(
            status_code=status.HTTP_409_CONFLICT,
            detail={"campo": e.campo, "mensaje": e.mensaje},
        )


@router.post(
    "/sesiones",
    status_code=status.HTTP_200_OK,
    response_model=SesionResponse,
    summary="Iniciar sesión y obtener token de acceso temporal"
)
def iniciar_sesion(
    payload: LoginRequest,
    caso_uso: IniciarSesion = Depends(caso_uso_iniciar_sesion),
):
    """
    RF-SOC-02: Inicio de sesión.
    - 200 OK: Retorna access_token JWT (HS256, 60m), expires_at en UTC 0 y datos del usuario.
    - 401 Unauthorized: 'Correo o contraseña incorrectos.' (mensaje idéntico exista o no el correo).
    """
    try:
        resultado = caso_uso.ejecutar(
            correo=payload.correo,
            contrasena=payload.contrasena,
        )

        fecha_exp_str = resultado.fecha_expiracion.strftime("%Y-%m-%dT%H:%M:%SZ")

        return SesionResponse(
            access_token=resultado.token,
            token_type="bearer",
            expires_at=fecha_exp_str,
            usuario=UsuarioResponse(
                id=resultado.usuario.id,
                nombre_usuario=resultado.usuario.nombre_usuario,
                nombre_visible=resultado.usuario.nombre_visible,
                correo=resultado.usuario.correo,
                fecha_alta=resultado.usuario.fecha_alta.strftime("%Y-%m-%dT%H:%M:%SZ"),
            ),
        )
    except CredencialesInvalidas:
        raise HTTPException(
            status_code=status.HTTP_401_UNAUTHORIZED,
            detail="Correo o contraseña incorrectos.",
        )


@router.get(
    "/usuarios/yo",
    status_code=status.HTTP_200_OK,
    response_model=UsuarioResponse,
    summary="Obtener perfil propio del usuario autenticado"
)
def obtener_perfil_propio(
    authorization: Optional[str] = Header(None),
    caso_uso: ObtenerPerfilPropio = Depends(caso_uso_obtener_perfil_propio),
):
    """
    Perfil propio protegido.
    - Requiere encabezado Authorization: Bearer <token>.
    - 200 OK: Retorna id, nombre_usuario, nombre_visible, correo y fecha_alta.
    - 401 Unauthorized: Si el token está ausente, alterado o vencido.
    """
    if not authorization or not authorization.startswith("Bearer "):
        raise HTTPException(
            status_code=status.HTTP_401_UNAUTHORIZED,
            detail="Token ausente o inválido.",
            headers={"WWW-Authenticate": "Bearer"},
        )

    token = authorization.split(" ")[1]

    try:
        usuario = caso_uso.ejecutar(token=token)
        return UsuarioResponse(
            id=usuario.id,
            nombre_usuario=usuario.nombre_usuario,
            nombre_visible=usuario.nombre_visible,
            correo=usuario.correo,
            fecha_alta=usuario.fecha_alta.strftime("%Y-%m-%dT%H:%M:%SZ"),
        )
    except TokenInvalido:
        raise HTTPException(
            status_code=status.HTTP_401_UNAUTHORIZED,
            detail="Token ausente, alterado o vencido.",
            headers={"WWW-Authenticate": "Bearer"},
        )

