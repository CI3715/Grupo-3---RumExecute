"""
Implementación del Repositorio de Gastos Compartidos en PostgreSQL con SQLAlchemy.
Cumple con aplicacion.puertos.repositorio_gastos.RepositorioGastos y
aplicacion.puertos.persistencia_gasto_compartido.PersistenciaGastoCompartido.
"""
from typing import Optional
from decimal import Decimal
from datetime import timezone
from sqlalchemy.orm import Session  # type: ignore

from dominio.balance import Gasto as GastoDominio
from dominio.division import CuotaDivision
from dominio.evento_cambio import EventoCambioUsuario
from dominio.idempotencia import RegistroIdempotenciaGasto
from aplicacion.puertos.repositorio_gastos import RepositorioGastos
from aplicacion.puertos.persistencia_gasto_compartido import PersistenciaGastoCompartido

from .modelos_bd import (
    GastoGrupo,
    ParteGasto,
    Usuario,
    EventoCambioUsuarioModel,
    ClaveIdempotenciaGasto,
)


class RepositorioGastosPostgres(RepositorioGastos, PersistenciaGastoCompartido):
    def __init__(self, db: Session):
        self.db = db

    def guardar_gasto(
        self,
        gasto: GastoDominio,
        eventos: tuple[EventoCambioUsuario, ...] = (),
        registro_idempotencia: Optional[RegistroIdempotenciaGasto] = None,
    ) -> None:
        """
        Guarda el gasto, sus cuotas, los eventos por participante y la clave de idempotencia
        en una única transacción atómica.
        """
        nuevo_gasto = GastoGrupo(
            id=gasto.id,
            grupo_id=gasto.grupo_id,
            pagador_id=gasto.pagador_id,
            concepto=gasto.concepto,
            monto=gasto.monto,
            fecha=gasto.fecha,
            regla_division="COMPARTIDO",
        )
        self.db.add(nuevo_gasto)
        self.db.flush()

        for cuota in gasto.cuotas:
            parte = ParteGasto(
                gasto_id=gasto.id,
                usuario_id=cuota.usuario_id,
                monto_cuota=cuota.monto,
                lleva_residuo=cuota.lleva_residuo,
            )
            self.db.add(parte)

        for ev in eventos:
            evento_modelo = EventoCambioUsuarioModel(
                id=ev.id,
                usuario_id=ev.usuario_id,
                grupo_id=ev.grupo_id,
                nombre_grupo=ev.nombre_grupo,
                gasto_id=ev.gasto_id,
                concepto=ev.concepto,
                fecha=ev.fecha,
                moneda=ev.moneda,
                monto_parte=ev.monto_parte,
                incluye_redondeo=ev.incluye_redondeo,
            )
            self.db.add(evento_modelo)

        if registro_idempotencia is not None:
            clave_modelo = ClaveIdempotenciaGasto(
                clave=registro_idempotencia.clave,
                usuario_id=registro_idempotencia.usuario_id,
                huella=registro_idempotencia.huella,
                gasto_id=registro_idempotencia.gasto_id,
            )
            self.db.add(clave_modelo)

        self.db.commit()

    def _mapear_a_dominio(self, g: GastoGrupo) -> GastoDominio:
        partes = self.db.query(ParteGasto).filter(ParteGasto.gasto_id == g.id).all()
        cuotas = []
        for p in partes:
            u = self.db.query(Usuario).filter(Usuario.id == p.usuario_id).first()
            nombre_u = u.nombre_usuario if u else p.usuario_id
            cuotas.append(
                CuotaDivision(
                    usuario_id=p.usuario_id,
                    nombre_usuario=nombre_u,
                    monto=Decimal(str(p.monto_cuota)),
                    lleva_residuo=p.lleva_residuo,
                )
            )

        return GastoDominio(
            id=g.id,
            grupo_id=g.grupo_id,
            concepto=g.concepto,
            pagador_id=g.pagador_id,
            monto=Decimal(str(g.monto)),
            cuotas=tuple(cuotas),
            fecha=g.fecha if g.fecha.tzinfo else g.fecha.replace(tzinfo=timezone.utc),
        )

    def listar_gastos_de_grupo(self, grupo_id: str) -> list[GastoDominio]:
        gastos_bd = self.db.query(GastoGrupo).filter(
            GastoGrupo.grupo_id == grupo_id
        ).order_by(GastoGrupo.fecha.asc()).all()

        return [self._mapear_a_dominio(g) for g in gastos_bd]

    def obtener_por_id(self, gasto_id: str) -> Optional[GastoDominio]:
        g = self.db.query(GastoGrupo).filter(GastoGrupo.id == gasto_id).first()
        if not g:
            return None
        return self._mapear_a_dominio(g)
