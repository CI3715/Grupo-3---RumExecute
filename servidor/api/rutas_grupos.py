"""
Router de Grupos, Miembros y Gastos Compartidos para FastAPI (Entrega 6).
Endpoints cubiertos:
- POST /grupos: Crear grupo (RF-COM-01)
- GET /grupos: Listar grupos e invitaciones del usuario
- POST /grupos/{id}/miembros: Invitar amigos al grupo (RF-COM-02)
- POST /grupos/{id}/miembros/yo/respuesta: Responder invitación de grupo
- POST /grupos/{id}/gastos: Registrar gasto compartido (RF-COM-04, RF-COM-05)
- GET /grupos/{id}: Ver grupo, gastos y balances (RF-COM-08, ESC-08/INV-17)
"""
from typing import Dict, Any, List, Optional
from decimal import Decimal
from fastapi import APIRouter, Depends, HTTPException, status
from pydantic import BaseModel, Field

from dominio.errores import ErrorDominio, ErrorConflicto, ErrorPermiso
from aplicacion.errores import GrupoNoEncontrado, UsuarioNoEncontrado
from infraestructura.middleware_auth import obtener_usuario_actual

from api.dependencias import (
    caso_uso_crear_grupo,
    caso_uso_listar_mis_grupos,
    caso_uso_invitar_miembro_grupo,
    caso_uso_responder_invitacion_grupo,
    caso_uso_registrar_gasto_compartido,
    caso_uso_obtener_detalle_grupo,
)
from aplicacion.crear_grupo import CrearGrupo
from aplicacion.listar_mis_grupos import ListarMisGrupos
from aplicacion.invitar_miembro_grupo import InvitarMiembroGrupo
from aplicacion.responder_invitacion_grupo import ResponderInvitacionGrupo
from aplicacion.registrar_gasto_compartido import RegistrarGastoCompartido
from aplicacion.obtener_detalle_grupo import ObtenerDetalleGrupo

router = APIRouter(tags=["Grupos y Gastos"])


# --- Modelos DTO HTTP ---

class CrearGrupoRequest(BaseModel):
    nombre: str = Field(..., examples=["Viaje a Mérida"])
    moneda: str = Field(..., examples=["USD"])
    descripcion: Optional[str] = Field("", examples=["Vacaciones de marzo"])


class GrupoCreadoResponse(BaseModel):
    id: str
    nombre_grupo: str
    moneda: str
    administrador_id: str
    descripcion: str


class GrupoResumenResponse(BaseModel):
    id: str
    nombre_grupo: str
    moneda: str
    descripcion: str
    administrador_id: str


class InvitacionGrupoResumenResponse(BaseModel):
    id: str
    grupo_id: str
    invitador_id: str
    estado: str


class ListadoGruposResponse(BaseModel):
    grupos: List[GrupoResumenResponse]
    invitaciones_recibidas: List[InvitacionGrupoResumenResponse]


class InvitarMiembroRequest(BaseModel):
    nombre_usuario: str = Field(..., examples=["beto"])


class InvitacionGrupoResponse(BaseModel):
    id: str
    grupo_id: str
    invitado_id: str
    invitador_id: str
    estado: str


class ResponderInvitacionGrupoRequest(BaseModel):
    aceptar: bool = Field(..., examples=[True])


class RespuestaInvitacionGrupoResponse(BaseModel):
    id: str
    grupo_id: str
    estado: str


class RegistrarGastoRequest(BaseModel):
    concepto: str = Field(..., examples=["Cena de bienvenida"])
    monto: Decimal = Field(..., examples=[100.00])
    pagador_id: str = Field(..., examples=["usuario_uuid"])
    regla_division: str = Field(..., examples=["IGUALES"], description="IGUALES, PORCENTAJES, MONTOS")
    participantes: List[str] = Field(..., examples=[["uuid1", "uuid2"]])
    valores: Optional[Dict[str, Decimal]] = Field(None, examples=[{"uuid1": 50.0, "uuid2": 50.0}])


class CuotaResponse(BaseModel):
    usuario_id: str
    nombre_usuario: str
    monto: Decimal
    lleva_residuo: bool


class GastoResponse(BaseModel):
    id: str
    grupo_id: str
    concepto: str
    pagador_id: str
    monto: Decimal
    fecha: str
    cuotas: List[CuotaResponse]


class MiembroDetalleResponse(BaseModel):
    usuario_id: str
    nombre_usuario: str
    nombre_visible: str
    es_administrador: bool
    balance: Decimal


class DetalleGrupoResponse(BaseModel):
    id: str
    nombre_grupo: str
    moneda: str
    descripcion: str
    miembros: List[MiembroDetalleResponse]
    gastos: List[GastoResponse]


# --- Endpoints ---

@router.post(
    "/grupos",
    status_code=status.HTTP_201_CREATED,
    response_model=GrupoCreadoResponse,
    summary="Crear grupo (RF-COM-01)"
)
def crear_grupo(
    payload: CrearGrupoRequest,
    usuario_autenticado: Dict[str, Any] = Depends(obtener_usuario_actual),
    caso_uso: CrearGrupo = Depends(caso_uso_crear_grupo),
):
    try:
        res = caso_uso.ejecutar(
            usuario_autenticado_id=usuario_autenticado["sub"],
            nombre_grupo=payload.nombre,
            moneda=payload.moneda,
            descripcion=payload.descripcion or "",
        )
        return GrupoCreadoResponse(
            id=res.id,
            nombre_grupo=res.nombre_grupo,
            moneda=res.moneda,
            administrador_id=res.administrador_id,
            descripcion=res.descripcion,
        )
    except ErrorDominio as e:
        raise HTTPException(
            status_code=status.HTTP_400_BAD_REQUEST,
            detail={"campo": e.campo, "mensaje": e.mensaje},
        )
    except UsuarioNoEncontrado:
        raise HTTPException(
            status_code=status.HTTP_404_NOT_FOUND,
            detail="Usuario no encontrado.",
        )


@router.get(
    "/grupos",
    status_code=status.HTTP_200_OK,
    response_model=ListadoGruposResponse,
    summary="Listar mis grupos e invitaciones pendientes"
)
def listar_mis_grupos(
    usuario_autenticado: Dict[str, Any] = Depends(obtener_usuario_actual),
    caso_uso: ListarMisGrupos = Depends(caso_uso_listar_mis_grupos),
):
    try:
        res = caso_uso.ejecutar(usuario_autenticado_id=usuario_autenticado["sub"])
        return ListadoGruposResponse(
            grupos=[
                GrupoResumenResponse(
                    id=g.id,
                    nombre_grupo=g.nombre_grupo,
                    moneda=g.moneda,
                    descripcion=g.descripcion,
                    administrador_id=g.administrador_id,
                )
                for g in res.grupos
            ],
            invitaciones_recibidas=[
                InvitacionGrupoResumenResponse(
                    id=i.id,
                    grupo_id=i.grupo_id,
                    invitador_id=i.invitador_id,
                    estado=i.estado,
                )
                for i in res.invitaciones_recibidas
            ],
        )
    except UsuarioNoEncontrado:
        raise HTTPException(
            status_code=status.HTTP_404_NOT_FOUND,
            detail="Usuario no encontrado.",
        )


@router.post(
    "/grupos/{id}/miembros",
    status_code=status.HTTP_201_CREATED,
    response_model=InvitacionGrupoResponse,
    summary="Invitar amigo a un grupo (RF-COM-02)"
)
def invitar_miembro(
    id: str,
    payload: InvitarMiembroRequest,
    usuario_autenticado: Dict[str, Any] = Depends(obtener_usuario_actual),
    caso_uso: InvitarMiembroGrupo = Depends(caso_uso_invitar_miembro_grupo),
):
    try:
        res = caso_uso.ejecutar(
            usuario_autenticado_id=usuario_autenticado["sub"],
            grupo_id=id,
            nombre_usuario_invitado=payload.nombre_usuario,
        )
        return InvitacionGrupoResponse(
            id=res.id,
            grupo_id=res.grupo_id,
            invitado_id=res.invitado_id,
            invitador_id=res.invitador_id,
            estado=res.estado,
        )
    except GrupoNoEncontrado:
        # Aislamiento (ESC-08): No revelar si el grupo existe si no es miembro
        raise HTTPException(
            status_code=status.HTTP_404_NOT_FOUND,
            detail="Grupo no encontrado.",
        )
    except UsuarioNoEncontrado:
        raise HTTPException(
            status_code=status.HTTP_404_NOT_FOUND,
            detail="Usuario a invitar no encontrado.",
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


@router.post(
    "/grupos/{id}/miembros/yo/respuesta",
    status_code=status.HTTP_200_OK,
    response_model=RespuestaInvitacionGrupoResponse,
    summary="Responder invitación de grupo (Aceptar/Rechazar)"
)
def responder_invitacion_grupo(
    id: str,
    payload: ResponderInvitacionGrupoRequest,
    usuario_autenticado: Dict[str, Any] = Depends(obtener_usuario_actual),
    caso_uso: ResponderInvitacionGrupo = Depends(caso_uso_responder_invitacion_grupo),
):
    try:
        res = caso_uso.ejecutar(
            usuario_autenticado_id=usuario_autenticado["sub"],
            grupo_id=id,
            aceptar=payload.aceptar,
        )
        return RespuestaInvitacionGrupoResponse(
            id=res.id,
            grupo_id=res.grupo_id,
            estado=res.estado,
        )
    except GrupoNoEncontrado:
        raise HTTPException(
            status_code=status.HTTP_404_NOT_FOUND,
            detail="Grupo o invitación no encontrada.",
        )
    except ErrorConflicto as e:
        raise HTTPException(
            status_code=status.HTTP_409_CONFLICT,
            detail={"campo": e.campo, "mensaje": e.mensaje},
        )


@router.post(
    "/grupos/{id}/gastos",
    status_code=status.HTTP_201_CREATED,
    response_model=GastoResponse,
    summary="Registrar gasto compartido (RF-COM-04 / RF-COM-05)"
)
def registrar_gasto(
    id: str,
    payload: RegistrarGastoRequest,
    usuario_autenticado: Dict[str, Any] = Depends(obtener_usuario_actual),
    caso_uso: RegistrarGastoCompartido = Depends(caso_uso_registrar_gasto_compartido),
):
    try:
        res = caso_uso.ejecutar(
            usuario_autenticado_id=usuario_autenticado["sub"],
            grupo_id=id,
            concepto=payload.concepto,
            pagador_id=payload.pagador_id,
            monto=payload.monto,
            tipo_division=payload.regla_division.upper(),
            participantes_ids=payload.participantes,
            valores=payload.valores,
        )
        return GastoResponse(
            id=res.id,
            grupo_id=res.grupo_id,
            concepto=res.concepto,
            pagador_id=res.pagador_id,
            monto=res.monto,
            fecha=res.fecha.strftime("%Y-%m-%dT%H:%M:%SZ"),
            cuotas=[
                CuotaResponse(
                    usuario_id=c.usuario_id,
                    nombre_usuario=c.nombre_usuario,
                    monto=c.monto,
                    lleva_residuo=c.lleva_residuo,
                )
                for c in res.cuotas
            ],
        )
    except GrupoNoEncontrado:
        raise HTTPException(
            status_code=status.HTTP_404_NOT_FOUND,
            detail="Grupo no encontrado.",
        )
    except ErrorDominio as e:
        # Porcentajes o montos que no cuadran (indicando la diferencia exacta)
        raise HTTPException(
            status_code=status.HTTP_400_BAD_REQUEST,
            detail={"campo": e.campo, "mensaje": e.mensaje},
        )


@router.get(
    "/grupos/{id}",
    status_code=status.HTTP_200_OK,
    response_model=DetalleGrupoResponse,
    summary="Ver grupo, gastos y balances (RF-COM-08 / ESC-08)"
)
def obtener_detalle_grupo(
    id: str,
    usuario_autenticado: Dict[str, Any] = Depends(obtener_usuario_actual),
    caso_uso: ObtenerDetalleGrupo = Depends(caso_uso_obtener_detalle_grupo),
):
    """
    Invariante Crítico (INV-17 / ESC-08):
    Si el usuario no es miembro, responde 404 Not Found idéntico a que si no existiera.
    """
    try:
        res = caso_uso.ejecutar(
            usuario_autenticado_id=usuario_autenticado["sub"],
            grupo_id=id,
        )
        return DetalleGrupoResponse(
            id=res.id,
            nombre_grupo=res.nombre_grupo,
            moneda=res.moneda,
            descripcion=res.descripcion,
            miembros=[
                MiembroDetalleResponse(
                    usuario_id=m.usuario_id,
                    nombre_usuario=m.nombre_usuario,
                    nombre_visible=m.nombre_visible,
                    es_administrador=m.es_administrador,
                    balance=m.balance,
                )
                for m in res.miembros
            ],
            gastos=[
                GastoResponse(
                    id=g.id,
                    grupo_id=g.grupo_id,
                    concepto=g.concepto,
                    pagador_id=g.pagador_id,
                    monto=g.monto,
                    fecha=g.fecha.strftime("%Y-%m-%dT%H:%M:%SZ"),
                    cuotas=[
                        CuotaResponse(
                            usuario_id=c.usuario_id,
                            nombre_usuario=c.nombre_usuario,
                            monto=c.monto,
                            lleva_residuo=c.lleva_residuo,
                        )
                        for c in g.cuotas
                    ],
                )
                for g in res.gastos
            ],
        )
    except GrupoNoEncontrado:
        raise HTTPException(
            status_code=status.HTTP_404_NOT_FOUND,
            detail="Grupo no encontrado.",
        )

