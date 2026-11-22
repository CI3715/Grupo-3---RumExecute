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

class ObtenerCambiosUsuario:
    """
    Caso de uso para consultar los eventos de cambio de un usuario
    a partir de una posición de cursor específica.
    """

    def __init__(self, repositorio_eventos: RepositorioEventosCambio):
        self.repositorio_eventos = repositorio_eventos

    def ejecutar(
        self,
        usuario_autenticado_id: str,
        desde: str | int,
    ) -> CambiosRespuesta:
        cursor = self._validar_desde(desde)

        # Pedimos 1 evento más que el límite para saber si quedan más eventos
        # sin requerir una consulta extra a la base de datos.
        eventos_obtenidos = self.repositorio_eventos.listar_posteriores(
            usuario_autenticado_id,
            cursor,
            LIMITE_LOTE + 1,
        )

        quedan_mas = len(eventos_obtenidos) > LIMITE_LOTE
        eventos_lote = eventos_obtenidos[:LIMITE_LOTE]

        eventos_dto = tuple(self._a_dto(ev) for ev in eventos_lote)
        nuevo_cursor = eventos_dto[-1].posicion if eventos_dto else cursor

        return CambiosRespuesta(
            eventos=eventos_dto,
            nuevo_cursor=nuevo_cursor,
            quedan_mas=quedan_mas,
        )

    def _validar_desde(self, desde: str | int) -> int:
        if isinstance(desde, bool):
            raise ErrorDominio("desde", MENSAJE_DESDE)

        if isinstance(desde, str) and desde.isascii() and desde.isdigit():
            return int(desde)

        if isinstance(desde, int) and desde >= 0:
            return desde

        raise ErrorDominio("desde", MENSAJE_DESDE)

    def _a_dto(self, evento: EventoCambioUsuario) -> EventoCambioRespuesta:
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
