from datetime import datetime, timezone
from decimal import Decimal
from uuid import uuid4

from aplicacion.dtos.gasto_compartido_respuesta import (
    CuotaRespuesta,
    GastoCompartidoRespuesta,
)
from aplicacion.errores import GrupoNoEncontrado, UsuarioNoEncontrado, GastoDeClaveNoEncontrado
from aplicacion.puertos.repositorio_grupos import RepositorioGrupos
from aplicacion.puertos.repositorio_gastos import RepositorioGastos
from aplicacion.puertos.repositorio_usuarios import RepositorioUsuarios
from aplicacion.puertos.repositorio_claves_idempotencia import RepositorioClavesIdempotencia
from aplicacion.puertos.persistencia_gasto_compartido import PersistenciaGastoCompartido

from dominio.balance import Gasto
from dominio.division import (
    dividir_partes_iguales,
    dividir_por_porcentajes,
    dividir_por_montos,
)
from dominio.errores import ErrorDominio
from dominio.grupo import es_miembro
from dominio.evento_cambio import EventoCambioUsuario
from dominio.idempotencia import (
    RegistroIdempotenciaGasto,
    calcular_huella,
    ErrorConflictoIdempotencia,
)


class RegistrarGastoCompartido:

    def __init__(
        self,
        repositorio_usuarios: RepositorioUsuarios,
        repositorio_grupos: RepositorioGrupos,
        repositorio_gastos: RepositorioGastos,
        repositorio_claves: RepositorioClavesIdempotencia,
        persistencia: PersistenciaGastoCompartido
    ):
        self.repositorio_usuarios = repositorio_usuarios
        self.repositorio_grupos = repositorio_grupos
        self.repositorio_gastos = repositorio_gastos
        self.repositorio_claves = repositorio_claves
        self.persistencia = persistencia

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
        clave_idempotencia: str | None = None, 
    ) -> GastoCompartidoRespuesta:

        # 1. Verificar que el grupo exista y sea accesible.
        grupo = self.repositorio_grupos.obtener_por_id(grupo_id)

        if grupo is None or not es_miembro(
            grupo, usuario_autenticado_id
        ):
            raise GrupoNoEncontrado()

        registro_idempotencia = None
        id_gasto_nuevo = str(uuid4())   # generado aquí, se usa también en el paso 7

        if clave_idempotencia is not None:
            huella = calcular_huella(
                grupo_id=grupo.id,
                concepto=concepto,
                pagador_id=pagador_id,
                monto=monto,
                tipo_division=tipo_division,
                participantes_ids=participantes_ids,
                valores=valores,
            )

            previo = self.repositorio_claves.buscar(usuario_autenticado_id, clave_idempotencia)

            if previo is not None:
                if previo.huella != huella:
                    raise ErrorConflictoIdempotencia(
                        "clave",
                        "La clave de idempotencia ya fue usada con datos distintos.",
                    )
                gasto_existente = self.repositorio_gastos.obtener(previo.gasto_id)
                if gasto_existente is None:
                    raise GastoDeClaveNoEncontrado()
                return self._construir_respuesta(gasto_existente, creado=False)

            registro_idempotencia = RegistroIdempotenciaGasto(
                clave=clave_idempotencia,
                usuario_id=usuario_autenticado_id,
                huella=huella,
                gasto_id=id_gasto_nuevo,
            )

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
        eventos = tuple(
            EventoCambioUsuario(
                id=str(uuid4()),
                posicion=None,
                usuario_id=cuota.usuario_id,
                grupo_id=gasto.grupo_id,
                nombre_grupo=grupo.nombre_grupo,
                gasto_id=gasto.id,
                concepto=gasto.concepto,
                fecha=gasto.fecha,
                moneda=grupo.moneda,
                monto_parte=cuota.monto,
                incluye_redondeo=cuota.lleva_residuo,
            )
            for cuota in gasto.cuotas
        )

        self.persistencia.guardar_gasto(gasto, eventos, registro_idempotencia)

        # 9. Construir respuesta pública.
        return self._construir_respuesta(gasto, creado=True)

    def _construir_respuesta(self, gasto, creado: bool) -> GastoCompartidoRespuesta:
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
            creado=creado,
        )
