
from aplicacion.dtos.respuesta_amistad import RespuestaAmistad
from aplicacion.errores import AmistadNoEncontrada
from aplicacion.puertos.repositorio_amistades import RepositorioAmistades
from dominio.amistad import responder_solicitud


class ResponderInvitacionAmistad:

    def __init__(
        self,
        repositorio_amistades: RepositorioAmistades,
    ):
        self.repositorio_amistades = repositorio_amistades

    def ejecutar(
        self,
        usuario_autenticado_id: str,
        amistad_id: str,
        aceptar: bool,
    ) -> RespuestaAmistad:

        amistad = self.repositorio_amistades.obtener_por_id(
            amistad_id
        )

        if amistad is None:
            raise AmistadNoEncontrada()

        amistad_actualizada = responder_solicitud(
            amistad,
            usuario_autenticado_id,
            aceptar,
        )

        self.repositorio_amistades.actualizar_respuesta(
            amistad_actualizada
        )

        return RespuestaAmistad(
            id=amistad_actualizada.id,
            estado=amistad_actualizada.estado.name,
        )
