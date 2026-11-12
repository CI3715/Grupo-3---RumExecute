"""
Implementación del Repositorio de Gastos Compartidos en PostgreSQL con SQLAlchemy.
Cumple con aplicacion.puertos.repositorio_gastos.RepositorioGastos.
"""
from decimal import Decimal
from datetime import timezone
from sqlalchemy.orm import Session  # type: ignore

from dominio.balance import Gasto as GastoDominio
from dominio.division import CuotaDivision
from aplicacion.puertos.repositorio_gastos import RepositorioGastos

from .modelos_bd import GastoGrupo, ParteGasto, Usuario


class RepositorioGastosPostgres(RepositorioGastos):
    def __init__(self, db: Session):
        self.db = db

    def guardar_gasto(self, gasto: GastoDominio) -> None:
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

        for cuota in gasto.cuotas:
            parte = ParteGasto(
                gasto_id=gasto.id,
                usuario_id=cuota.usuario_id,
                monto_cuota=cuota.monto,
                lleva_residuo=cuota.lleva_residuo,
            )
            self.db.add(parte)

        self.db.commit()

    def listar_gastos_de_grupo(self, grupo_id: str) -> list[GastoDominio]:
        gastos_bd = self.db.query(GastoGrupo).filter(
            GastoGrupo.grupo_id == grupo_id
        ).order_by(GastoGrupo.fecha.asc()).all()

        resultado = []
        for g in gastos_bd:
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

            resultado.append(
                GastoDominio(
                    id=g.id,
                    grupo_id=g.grupo_id,
                    concepto=g.concepto,
                    pagador_id=g.pagador_id,
                    monto=Decimal(str(g.monto)),
                    cuotas=tuple(cuotas),
                    fecha=g.fecha if g.fecha.tzinfo else g.fecha.replace(tzinfo=timezone.utc),
                )
            )

        return resultado

