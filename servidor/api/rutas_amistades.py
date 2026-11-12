"""
Router de Amistades y Búsqueda de Usuarios para FastAPI (Entrega 6).
Endpoints cubiertos:
- GET /usuarios?usuario=<exacto> (RF-SOC-04)
- POST /amistades (RF-SOC-05)
- POST /amistades/{id}/respuesta (INV-10)
- GET /amistades
- DELETE /amistades/{id} (RF-SOC-06)
"""
from typing import Dict, Any, List
from fastapi import APIRouter, Depends, HTTPException, Query, status
from pydantic import BaseModel, Field

from dominio.errores import ErrorDominio, ErrorConflicto, ErrorPermiso
from aplicacion.errores import UsuarioNoEncontrado, AmistadNoEncontrada
from infraestructura.middleware_auth import obtener_usuario_actual

from api.dependencias import (
    caso_uso_buscar_usuario,
    caso_uso_enviar_invitacion_amistad,
    caso_uso_responder_invitacion_amistad,
    caso_uso_listar_amistades,
    caso_uso_eliminar_amistad,
)
from aplicacion.buscar_usuario import BuscarUsuario
from aplicacion.enviar_invitacion_amistad import EnviarInvitacionAmistad
from aplicacion.responder_invitacion_amistad import ResponderInvitacionAmistad
from aplicacion.listar_amistades import ListarAmistades
from aplicacion.eliminar_amistad import EliminarAmistad

router = APIRouter(tags=["Amistades"])


# --- Modelos DTO HTTP ---

class UsuarioPublicoResponse(BaseModel):
    nombre_usuario: str
    nombre_visible: str


class EnviarInvitacionRequest(BaseModel):
    nombre_usuario: str = Field(..., examples=["beto"])


class InvitacionAmistadResponse(BaseModel):
    id: str
    nombre_usuario_solicitante: str
    nombre_usuario_receptor: str
    estado: str


class ResponderAmistadRequest(BaseModel):
    aceptar: bool = Field(..., examples=[True])


class RespuestaAmistadResponse(BaseModel):
    id: str
    estado: str


class AmigoResponse(BaseModel):
    amistad_id: str
    nombre_usuario: str
    nombre_visible: str


class InvitacionResumenResponse(BaseModel):
    amistad_id: str
    nombre_usuario: str
    nombre_visible: str


class ListadoAmistadesResponse(BaseModel):
    amigos: List[AmigoResponse]
    invitaciones_recibidas: List[InvitacionResumenResponse]
    invitaciones_enviadas: List[InvitacionResumenResponse]


# --- Endpoints ---

@router.get(
    "/usuarios",
    status_code=status.HTTP_200_OK,
    response_model=UsuarioPublicoResponse,
    summary="Buscar usuario por coincidencia exacta (RF-SOC-04)"
)
def buscar_usuario(
    usuario: str = Query(..., alias="usuario", description="Nombre de usuario exacto a buscar"),
    _token_valido: Dict[str, Any] = Depends(obtener_usuario_actual),
    caso_uso: BuscarUsuario = Depends(caso_uso_buscar_usuario),
):
    """
    Búsqueda por coincidencia exacta. No devuelve el correo.
    404 si no existe o ante búsquedas parciales.
    """
    try:
        resultado = caso_uso.ejecutar(nombre_usuario=usuario)
        return UsuarioPublicoResponse(
            nombre_usuario=resultado.nombre_usuario,
            nombre_visible=resultado.nombre_visible,
        )
    except UsuarioNoEncontrado:
        raise HTTPException(
            status_code=status.HTTP_404_NOT_FOUND,
            detail="Usuario no encontrado.",
        )


@router.post(
    "/amistades",
    status_code=status.HTTP_201_CREATED,
    response_model=InvitacionAmistadResponse,
    summary="Enviar invitación de amistad (RF-SOC-05)"
)
def invitar_amistad(
    payload: EnviarInvitacionRequest,
    usuario_autenticado: Dict[str, Any] = Depends(obtener_usuario_actual),
    caso_uso: EnviarInvitacionAmistad = Depends(caso_uso_enviar_invitacion_amistad),
):
    try:
        res = caso_uso.ejecutar(
            usuario_autenticado_id=usuario_autenticado["sub"],
            nombre_usuario_receptor=payload.nombre_usuario,
        )
        return InvitacionAmistadResponse(
            id=res.id,
            nombre_usuario_solicitante=res.nombre_usuario_solicitante,
            nombre_usuario_receptor=res.nombre_usuario_receptor,
            estado=res.estado,
        )
    except UsuarioNoEncontrado:
        raise HTTPException(
            status_code=status.HTTP_404_NOT_FOUND,
            detail="Usuario no encontrado.",
        )
    except ErrorDominio as e:
        raise HTTPException(
            status_code=status.HTTP_400_BAD_REQUEST,
            detail={"campo": e.campo, "mensaje": e.mensaje},
        )
    except ErrorConflicto as e:
        raise HTTPException(
            status_code=status.HTTP_409_CONFLICT,
            detail={"campo": e.campo, "mensaje": e.mensaje},
        )


@router.get(
    "/amistades",
    status_code=status.HTTP_200_OK,
    response_model=ListadoAmistadesResponse,
    summary="Listar amigos e invitaciones pendientes (recibidas y enviadas)"
)
def listar_amistades(
    usuario_autenticado: Dict[str, Any] = Depends(obtener_usuario_actual),
    caso_uso: ListarAmistades = Depends(caso_uso_listar_amistades),
):
    try:
        res = caso_uso.ejecutar(usuario_autenticado_id=usuario_autenticado["sub"])
        return ListadoAmistadesResponse(
            amigos=[
                AmigoResponse(
                    amistad_id=a.amistad_id,
                    nombre_usuario=a.nombre_usuario,
                    nombre_visible=a.nombre_visible,
                )
                for a in res.amigos
            ],
            invitaciones_recibidas=[
                InvitacionResumenResponse(
                    amistad_id=i.amistad_id,
                    nombre_usuario=i.nombre_usuario,
                    nombre_visible=i.nombre_visible,
                )
                for i in res.invitaciones_recibidas
            ],
            invitaciones_enviadas=[
                InvitacionResumenResponse(
                    amistad_id=i.amistad_id,
                    nombre_usuario=i.nombre_usuario,
                    nombre_visible=i.nombre_visible,
                )
                for i in res.invitaciones_enviadas
            ],
        )
    except UsuarioNoEncontrado:
        raise HTTPException(
            status_code=status.HTTP_404_NOT_FOUND,
            detail="Usuario no encontrado.",
        )


@router.post(
    "/amistades/{id}/respuesta",
    status_code=status.HTTP_200_OK,
    response_model=RespuestaAmistadResponse,
    summary="Responder (aceptar o rechazar) invitación de amistad (INV-10)"
)
def responder_invitacion(
    id: str,
    payload: ResponderAmistadRequest,
    usuario_autenticado: Dict[str, Any] = Depends(obtener_usuario_actual),
    caso_uso: ResponderInvitacionAmistad = Depends(caso_uso_responder_invitacion_amistad),
):
    try:
        res = caso_uso.ejecutar(
            usuario_autenticado_id=usuario_autenticado["sub"],
            amistad_id=id,
            aceptar=payload.aceptar,
        )
        return RespuestaAmistadResponse(
            id=res.id,
            estado=res.estado,
        )
    except AmistadNoEncontrada:
        raise HTTPException(
            status_code=status.HTTP_404_NOT_FOUND,
            detail="Invitación no encontrada.",
        )
    except ErrorPermiso as e:
        raise HTTPException(
            status_code=status.HTTP_403_FORBIDDEN,
            detail={"campo": e.campo, "mensaje": e.mensaje},
        )
    except ErrorConflicto as e:
        raise HTTPException(
            status_code=status.HTTP_409_CONFLICT,
            detail={"campo": e.campo, "mensaje": e.mensaje},
        )


@router.delete(
    "/amistades/{id}",
    status_code=status.HTTP_204_NO_CONTENT,
    summary="Eliminar amistad activa (RF-SOC-06)"
)
def eliminar_amistad(
    id: str,
    usuario_autenticado: Dict[str, Any] = Depends(obtener_usuario_actual),
    caso_uso: EliminarAmistad = Depends(caso_uso_eliminar_amistad),
):
    try:
        caso_uso.ejecutar(
            usuario_autenticado_id=usuario_autenticado["sub"],
            amistad_id=id,
        )
        return None
    except AmistadNoEncontrada:
        raise HTTPException(
            status_code=status.HTTP_404_NOT_FOUND,
            detail="Amistad no encontrada o no está activa.",
        )
    except ErrorPermiso as e:
        raise HTTPException(
            status_code=status.HTTP_403_FORBIDDEN,
            detail={"campo": e.campo, "mensaje": e.mensaje},
        )
    except ErrorConflicto as e:
        raise HTTPException(
            status_code=status.HTTP_409_CONFLICT,
            detail={"campo": e.campo, "mensaje": e.mensaje},
        )

