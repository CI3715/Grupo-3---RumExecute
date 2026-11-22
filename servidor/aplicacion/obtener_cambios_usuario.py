from dataclasses import dataclass
from typing import List, Tuple
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
    a partir de una posición de cursor específica.
    """

    def __init__(self, repositorio_eventos: RepositorioEventosCambio):
        self.repositorio_eventos = repositorio_eventos

    def ejecutar(
        self,
        usuario_autenticado_id: str | None = None,
        desde: str | int | None = None,
        *,
        usuario_id: str | None = None,
        desde_cursor: str | int | None = None,
        limite: int = LIMITE_LOTE,
    ) -> CambiosRespuesta:
        # Determinar argumentos recibidos (compatibilidad con ambas firmas)
        usar_consultar_cambios = usuario_id is not None or desde_cursor is not None
        uid = usuario_id if usuario_id is not None else usuario_autenticado_id
        cursor_raw = desde_cursor if desde_cursor is not None else desde

        if uid is None:
            raise ErrorDominio("usuario_id", "El identificador del usuario es obligatorio.")

        cursor = self._validar_desde(cursor_raw)

        if usar_consultar_cambios and hasattr(self.repositorio_eventos, "consultar_cambios_usuario"):
            eventos_bd, nuevo_cursor, quedan_mas = self.repositorio_eventos.consultar_cambios_usuario(
                usuario_id=uid,
                desde_cursor=cursor,
                limite=limite,
            )
            eventos_dto = tuple(self._a_dto(ev) for ev in eventos_bd)
        else:
            # Pedimos 1 evento más que el límite para saber si quedan más eventos
            # sin requerir una consulta extra a la base de datos.
            eventos_obtenidos = self.repositorio_eventos.listar_posteriores(
                uid,
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
