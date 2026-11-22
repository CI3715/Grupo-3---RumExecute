"""
Caso de uso: ObtenerCambiosUsuario (Entrega 7 - Subtarea 2.2).
Permite a un usuario consultar sus eventos de cambio pendientes desde un cursor dado.
Invariantes garantizados:
- INV-16: La información solo baja, nunca sube.
- INV-17: Aislamiento estricto: solo se devuelven los eventos del usuario autenticado.
"""
from dataclasses import dataclass
from typing import List
from dominio.errores import ErrorDominio
from dominio.evento_cambio import EventoCambioUsuario
from aplicacion.puertos.repositorio_eventos_cambio import RepositorioEventosCambio


@dataclass(frozen=True)
class CambiosUsuarioRespuesta:
    eventos: List[EventoCambioUsuario]
    nuevo_cursor: int
    quedan_mas: bool


class ObtenerCambiosUsuario:
    def __init__(self, repositorio_eventos: RepositorioEventosCambio):
        self.repositorio_eventos = repositorio_eventos

    def ejecutar(
        self,
        usuario_id: str,
        desde_cursor: int,
        limite: int = 100,
    ) -> CambiosUsuarioRespuesta:
        if desde_cursor is None or desde_cursor < 0:
            raise ErrorDominio(
                "desde",
                "El cursor 'desde' debe ser un entero mayor o igual a 0.",
            )

        eventos, nuevo_cursor, quedan_mas = self.repositorio_eventos.consultar_cambios_usuario(
            usuario_id=usuario_id,
            desde_cursor=desde_cursor,
            limite=limite,
        )

        return CambiosUsuarioRespuesta(
            eventos=eventos,
            nuevo_cursor=nuevo_cursor,
            quedan_mas=quedan_mas,
        )
