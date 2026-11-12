from aplicacion.dtos.detalle_grupo import (
    DetalleGrupo,
    MiembroGrupoRespuesta,
)
from aplicacion.dtos.gasto_compartido_respuesta import (
    CuotaRespuesta,
    GastoCompartidoRespuesta,
)
from aplicacion.errores import (
    GrupoNoEncontrado,
    UsuarioNoEncontrado,
)
from aplicacion.puertos.repositorio_grupos import RepositorioGrupos
from aplicacion.puertos.repositorio_gastos import RepositorioGastos
from aplicacion.puertos.repositorio_usuarios import RepositorioUsuarios

from dominio.balance import calcular_balances
from dominio.errores import ErrorDominio
from dominio.grupo import es_miembro


class ObtenerDetalleGrupo:

    def __init__(
        self,
        repositorio_usuarios: RepositorioUsuarios,
        repositorio_grupos: RepositorioGrupos,
        repositorio_gastos: RepositorioGastos,
    ):
        self.repositorio_usuarios = repositorio_usuarios
        self.repositorio_grupos = repositorio_grupos
        self.repositorio_gastos = repositorio_gastos

    def ejecutar(
        self,
        usuario_autenticado_id: str,
        grupo_id: str,
    ) -> DetalleGrupo:

        # 1. Obtener grupo y verificar acceso.
        grupo = self.repositorio_grupos.obtener_por_id(
            grupo_id
        )

        if grupo is None or not es_miembro(
            grupo, usuario_autenticado_id
        ):
            raise GrupoNoEncontrado()

        # 2. Consultar gastos del grupo.
        gastos = self.repositorio_gastos.listar_gastos_de_grupo(
            grupo.id
        )

        # Verificar que no se hayan recuperado gastos
        # pertenecientes a otro grupo.
        if any(gasto.grupo_id != grupo.id for gasto in gastos):
            raise ErrorDominio(
                "gastos",
                "Se encontraron gastos de otro grupo.",
            )

        # 3. Calcular balances mediante el dominio.
        balances = calcular_balances(
            grupo.id_miembros,
            gastos,
        )

        # 4. Preparar información pública de miembros.
        miembros_respuesta = []

        for miembro_id in grupo.id_miembros:

            usuario = (
                self.repositorio_usuarios.obtener_usuario_por_id(
                    miembro_id
                )
            )

            if usuario is None:
                raise UsuarioNoEncontrado()

            miembros_respuesta.append(
                MiembroGrupoRespuesta(
                    usuario_id=usuario.id,
                    nombre_usuario=usuario.nombre_usuario,
                    nombre_visible=usuario.nombre_visible,
                    es_administrador=(
                        usuario.id == grupo.id_administrador
                    ),
                    balance=balances[usuario.id],
                )
            )

        # 5. Preparar gastos y cuotas.
        gastos_respuesta = tuple(
            GastoCompartidoRespuesta(
                id=gasto.id,
                grupo_id=gasto.grupo_id,
                concepto=gasto.concepto,
                pagador_id=gasto.pagador_id,
                monto=gasto.monto,
                fecha=gasto.fecha,
                cuotas=tuple(
                    CuotaRespuesta(
                        usuario_id=cuota.usuario_id,
                        nombre_usuario=cuota.nombre_usuario,
                        monto=cuota.monto,
                        lleva_residuo=cuota.lleva_residuo,
                    )
                    for cuota in gasto.cuotas
                ),
            )
            for gasto in gastos
        )

        # 6. Construir respuesta final.
        return DetalleGrupo(
            id=grupo.id,
            nombre_grupo=grupo.nombre_grupo,
            moneda=grupo.moneda,
            descripcion=grupo.descripcion,
            administrador_id=grupo.id_administrador,
            miembros=tuple(miembros_respuesta),
            gastos=gastos_respuesta,
        )
