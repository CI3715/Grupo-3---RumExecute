"""
Router de Sincronización de Cambios para FastAPI (Entrega 7 - Subtarea 3.2).
Endpoints cubiertos:
- GET /cambios?desde=<cursor>: Obtener eventos de cambio del usuario autenticado.
Invariantes arquitectónicos:
- INV-16 / RES-02: Principio de privacidad - la información solo baja, nunca sube.
- INV-17: Aislamiento estricto: solo se devuelven los eventos del usuario autenticado.
"""
from typing import Dict, Any, List
from datetime import datetime
from decimal import Decimal
from fastapi import APIRouter, Depends, HTTPException, Query, status
from pydantic import BaseModel, Field

from dominio.errores import ErrorDominio
from infraestructura.middleware_auth import obtener_usuario_actual
from api.dependencias import caso_uso_obtener_cambios_usuario
from aplicacion.obtener_cambios_usuario import ObtenerCambiosUsuario

router = APIRouter(tags=["Sincronización"])


# --- Modelos DTO HTTP ---

class EventoCambioResponse(BaseModel):
    id: str = Field(..., examples=["11111111-1111-1111-1111-111111111111"])
    posicion: int = Field(..., examples=[1])
    usuario_id: str = Field(..., examples=["usuario_uuid"])
    grupo_id: str = Field(..., examples=["grupo_uuid"])
    nombre_grupo: str = Field(..., examples=["Viaje a Mérida"])
    gasto_id: str = Field(..., examples=["gasto_uuid"])
    concepto: str = Field(..., examples=["Hotel"])
    fecha: str = Field(..., examples=["2026-10-07T20:00:00Z"])
    moneda: str = Field(..., examples=["USD"])
    monto_parte: Decimal = Field(..., examples=[33.33])
    incluye_redondeo: bool = Field(..., examples=[False])


class CambiosUsuarioResponse(BaseModel):
    eventos: List[EventoCambioResponse]
    nuevo_cursor: int = Field(..., examples=[1])
    quedan_mas: bool = Field(..., examples=[False])


# --- Endpoints ---

@router.get(
    "/cambios",
    status_code=status.HTTP_200_OK,
    response_model=CambiosUsuarioResponse,
    summary="Obtener eventos de cambio del usuario (INV-16 / INV-17 / Subtarea 3.2)"
)
def obtener_cambios(
    desde: Any = Query(0, description="Cursor numérico de cambios"),
    usuario_autenticado: Dict[str, Any] = Depends(obtener_usuario_actual),
    caso_uso: ObtenerCambiosUsuario = Depends(caso_uso_obtener_cambios_usuario),
):
    try:
        desde_int = int(desde)
        if desde_int < 0:
            raise ValueError()
    except (ValueError, TypeError):
        raise HTTPException(
            status_code=status.HTTP_400_BAD_REQUEST,
            detail={"campo": "desde", "mensaje": "El cursor 'desde' debe ser un entero mayor o igual a 0."},
        )

    try:
        res = caso_uso.ejecutar(
            usuario_id=usuario_autenticado["sub"],
            desde_cursor=desde_int,
            limite=100,
        )

        eventos_dto = [
            EventoCambioResponse(
                id=ev.id,
                posicion=ev.posicion,
                usuario_id=ev.usuario_id,
                grupo_id=ev.grupo_id,
                nombre_grupo=ev.nombre_grupo,
                gasto_id=ev.gasto_id,
                concepto=ev.concepto,
                fecha=(
                    ev.fecha.strftime("%Y-%m-%dT%H:%M:%SZ")
                    if isinstance(ev.fecha, datetime)
                    else str(ev.fecha)
                ),
                moneda=ev.moneda,
                monto_parte=ev.monto_parte,
                incluye_redondeo=ev.incluye_redondeo,
            )
            for ev in res.eventos
        ]

        return CambiosUsuarioResponse(
            eventos=eventos_dto,
            nuevo_cursor=res.nuevo_cursor,
            quedan_mas=res.quedan_mas,
        )
    except ErrorDominio as e:
        raise HTTPException(
            status_code=status.HTTP_400_BAD_REQUEST,
            detail={"campo": e.campo, "mensaje": e.mensaje},
        )
