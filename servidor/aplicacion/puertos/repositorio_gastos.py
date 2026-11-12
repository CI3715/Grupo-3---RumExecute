
from typing import Protocol

from dominio.balance import Gasto


class RepositorioGastos(Protocol):

    def guardar_gasto(
        self,
        gasto: Gasto,
    ) -> None:
        """Guarda un gasto compartido.

        Debe persistir:
        - El identificador del gasto.
        - El identificador del grupo.
        - El usuario que realizó el pago.
        - El concepto, monto y fecha.
        - Todas las cuotas de los participantes.
        - La marca lleva_residuo de cada cuota.

        El gasto y sus cuotas deben guardarse en
        una única transacción.

        Si ocurre un error, no debe persistirse
        ninguna parte del gasto.
        """
        ...

    def listar_gastos_de_grupo(
        self,
        grupo_id: str,
    ) -> list[Gasto]:
        """Devuelve todos los gastos de un grupo.

        Cada gasto debe incluir sus cuotas completas.

        La capa de aplicación debe verificar primero
        que el usuario autenticado pertenece al grupo.

        Devuelve una lista vacía si el grupo todavía
        no tiene gastos registrados.
        """
        ...
