from dataclasses import dataclass
from typing import List

from aplicacion.dtos.cambios_respuesta import (
    CambiosRespuesta,
    EventoCambioRespuesta,
)
from aplicacion.puertos.repositorio_eventos_cambio import (
    RepositorioEventosCambio,
)
from aplicacion.errores import ErrorAplicacion
from dominio.errores import ErrorDominio
from dominio.evento_cambio import EventoCambioUsuario

LIMITE_LOTE = 100
MENSAJE_DESDE = "El cursor 'desde' debe ser un número entero mayor o igual a 0."

# Alias para compatibilidad de tipos con rutas_cambios
CambiosUsuarioRespuesta = CambiosRespuesta


class ObtenerCambiosUsuario:
    """
    Caso de uso para consultar los eventos de cambio de un usuario
    a partir de una posición de cursor específica (Subtarea 2.2).
    Invariantes garantizados:
    - INV-16: La información solo baja, nunca sube.
    - INV-17: Aislamiento estricto: solo se devuelven los eventos del usuario autenticado.
    """

    def __init__(self, repositorio_eventos: RepositorioEventosCambio):
        self.repositorio_eventos = repositorio_eventos

    def ejecutar(
        self,
        usuario_id: str,
        desde: str | int = 0,
        limite: int = LIMITE_LOTE,
        *,
        desde_cursor: str | int | None = None,
    ) -> CambiosRespuesta:
        if not usuario_id:
            raise ErrorDominio("usuario_id", "El identificador del usuario es obligatorio.")

        cursor_raw = desde_cursor if desde_cursor is not None else desde
        cursor = self._validar_desde(cursor_raw)

        # Algoritmo N+1 de paginación: pedimos 1 evento más que el límite
        # para saber si quedan más eventos sin requerir una consulta extra.
        eventos_obtenidos = self.repositorio_eventos.listar_posteriores(
            usuario_id,
            cursor,
            limite + 1,
        )

        quedan_mas = len(eventos_obtenidos) > limite
        eventos_lote = eventos_obtenidos[:limite]

        eventos_dto = tuple(self._a_dto(ev) for ev in eventos_lote)
        nuevo_cursor = eventos_dto[-1].posicion if eventos_dto else cursor

        return CambiosRespuesta(
            eventos=eventos_dto,
            nuevo_cursor=nuevo_cursor,
            quedan_mas=quedan_mas,
        )

    def _validar_desde(self, desde: str | int | None) -> int:
        if desde is None or isinstance(desde, bool):
            raise ErrorDominio("desde", MENSAJE_DESDE)

        if isinstance(desde, str):
            if desde.isascii() and desde.isdigit():
                return int(desde)
            raise ErrorDominio("desde", MENSAJE_DESDE)

        if isinstance(desde, int) and desde >= 0:
            return desde

        raise ErrorDominio("desde", MENSAJE_DESDE)

    def _a_dto(self, evento: EventoCambioUsuario | EventoCambioRespuesta) -> EventoCambioRespuesta:
        if isinstance(evento, EventoCambioRespuesta):
            return evento

        if evento.posicion is None:
            raise ErrorAplicacion(
                "posicion",
                "El evento recuperado no tiene posición asignada.",
            )

        return EventoCambioRespuesta(
            id=evento.id,
            posicion=evento.posicion,
            usuario_id=evento.usuario_id,
            grupo_id=evento.grupo_id,
            nombre_grupo=evento.nombre_grupo,
            gasto_id=evento.gasto_id,
            concepto=evento.concepto,
            fecha=evento.fecha,
            moneda=evento.moneda,
            monto_parte=evento.monto_parte,
            incluye_redondeo=evento.incluye_redondeo,
        )
