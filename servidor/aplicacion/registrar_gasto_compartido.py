from datetime import datetime, timezone
from decimal import Decimal
from uuid import uuid4

from aplicacion.dtos.gasto_compartido_respuesta import (
    CuotaRespuesta,
    GastoCompartidoRespuesta,
)
from aplicacion.errores import GrupoNoEncontrado, UsuarioNoEncontrado
from aplicacion.puertos.repositorio_grupos import RepositorioGrupos
from aplicacion.puertos.repositorio_gastos import RepositorioGastos
from aplicacion.puertos.repositorio_usuarios import RepositorioUsuarios

from dominio.balance import Gasto
from dominio.division import (
    dividir_partes_iguales,
    dividir_por_porcentajes,
    dividir_por_montos,
)
from dominio.errores import ErrorDominio
from dominio.grupo import es_miembro


class RegistrarGastoCompartido:

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
        concepto: str,
        pagador_id: str,
        monto: Decimal,
        tipo_division: str,
        participantes_ids: list[str],
        valores: dict[str, Decimal] | None = None,
    ) -> GastoCompartidoRespuesta:

        # 1. Verificar que el grupo exista y sea accesible.
        grupo = self.repositorio_grupos.obtener_por_id(grupo_id)

        if grupo is None or not es_miembro(
            grupo, usuario_autenticado_id
        ):
            raise GrupoNoEncontrado()

        # 2. Verificar que el pagador sea miembro.
        if not es_miembro(grupo, pagador_id):
            raise ErrorDominio(
                "pagador",
                "El pagador debe pertenecer al grupo.",
            )

        # 3. Validar participantes.
        if not participantes_ids:
            raise ErrorDominio(
                "participantes",
                "Debe haber al menos un participante.",
            )

        if len(set(participantes_ids)) != len(participantes_ids):
            raise ErrorDominio(
                "participantes",
                "No puede haber participantes duplicados.",
            )

        for participante_id in participantes_ids:
            if not es_miembro(grupo, participante_id):
                raise ErrorDominio(
                    "participantes",
                    "Todos los participantes deben pertenecer al grupo.",
                )

        # 4. Validar modalidad y valores.
        modalidades = ("IGUALES", "PORCENTAJES", "MONTOS")

        if tipo_division not in modalidades:
            raise ErrorDominio(
                "tipo_division",
                "La modalidad de división no es válida.",
            )

        if tipo_division == "IGUALES":
            if valores:
                raise ErrorDominio(
                    "valores",
                    "La división igualitaria no requiere valores.",
                )

        else:
            if valores is None or (
                set(valores.keys()) != set(participantes_ids)
            ):
                raise ErrorDominio(
                    "valores",
                    "Debe indicarse un valor para cada participante.",
                )

        # 5. Obtener los nombres de usuario para
        # el reparto determinista del residuo.
        nombres = {}

        for participante_id in participantes_ids:
            usuario = (
                self.repositorio_usuarios.obtener_usuario_por_id(
                    participante_id
                )
            )

            if usuario is None:
                raise UsuarioNoEncontrado()

            nombres[participante_id] = usuario.nombre_usuario

        # 6. Calcular las cuotas mediante el dominio.
        if tipo_division == "IGUALES":
            cuotas = dividir_partes_iguales(
                monto,
                nombres,
            )

        elif tipo_division == "PORCENTAJES":
            cuotas = dividir_por_porcentajes(
                monto,
                valores,
                nombres,
            )

        else:
            cuotas = dividir_por_montos(
                monto,
                valores,
                nombres,
            )

        # 7. Construir y validar el gasto.
        gasto = Gasto(
            id=str(uuid4()),
            grupo_id=grupo.id,
            concepto=concepto,
            pagador_id=pagador_id,
            monto=monto,
            cuotas=tuple(cuotas),
            fecha=datetime.now(timezone.utc),
        )

        # 8. Persistir el gasto y sus cuotas.
        self.repositorio_gastos.guardar_gasto(gasto)

        # 9. Construir respuesta pública.
        return GastoCompartidoRespuesta(
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
