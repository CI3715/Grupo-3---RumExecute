from aplicacion.errores import AmistadNoEncontrada
from aplicacion.puertos.repositorio_amistades import RepositorioAmistades
from dominio.amistad import EstadoAmistad
from dominio.errores import ErrorConflicto, ErrorPermiso


class EliminarAmistad:

    def __init__(
        self,
        repositorio_amistades: RepositorioAmistades,
    ):
        self.repositorio_amistades = repositorio_amistades

    def ejecutar(
        self,
        usuario_autenticado_id: str,
        amistad_id: str,
    ) -> None:

        amistad = self.repositorio_amistades.obtener_por_id(
            amistad_id
        )

        if amistad is None:
            raise AmistadNoEncontrada()

        if usuario_autenticado_id not in (
            amistad.id_solicitante,
            amistad.id_receptor,
        ):
            raise ErrorPermiso(
                "usuario",
                "Solo los participantes pueden eliminar la amistad.",
            )

        if amistad.estado != EstadoAmistad.ACTIVA:
            raise ErrorConflicto(
                "amistad",
                "Solo se pueden eliminar amistades activas.",
            )

        eliminada = self.repositorio_amistades.eliminar_amistad(
            amistad_id,
            usuario_autenticado_id,
        )

        if not eliminada:
            raise AmistadNoEncontrada()
