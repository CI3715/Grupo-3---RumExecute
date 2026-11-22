import hashlib
import json
from dataclasses import dataclass
from decimal import Decimal

from dominio.errores import ErrorConflicto


class ErrorConflictoIdempotencia(ErrorConflicto):
    """Lanzada cuando una misma clave de idempotencia se reutiliza con datos distintos (HTTP 409)."""

    def __init__(
        self,
        campo_o_mensaje: str = "La clave de idempotencia ya fue utilizada con datos distintos.",
        mensaje: str | None = None,
    ):
        if mensaje is None:
            # Llamado con 0 o 1 argumento: ErrorConflictoIdempotencia() o ErrorConflictoIdempotencia("mensaje")
            super().__init__("idempotency_key", campo_o_mensaje)
        else:
            # Llamado con 2 argumentos: ErrorConflictoIdempotencia("clave", "mensaje")
            super().__init__(campo_o_mensaje, mensaje)


@dataclass(frozen=True)
class RegistroIdempotenciaGasto:
    clave: str
    usuario_id: str
    huella: str
    gasto_id: str


def calcular_huella(
    grupo_id: str,
    concepto: str,
    pagador_id: str,
    monto: Decimal,
    tipo_division: str,
    participantes_ids: list[str],
    valores: dict[str, Decimal] | None = None
) -> str:
    datos = {
        "grupo_id": grupo_id,
        "concepto": concepto.strip(),
        "pagador_id": pagador_id,
        "monto": format(monto.normalize(), "f"),
        "tipo_division": tipo_division,
        "valores": {uid: format(v.normalize(), "f") for uid, v in (valores or {}).items()},
        "participantes_ids": sorted(participantes_ids),
    }
    texto = json.dumps(datos, sort_keys=True)
    return hashlib.sha256(texto.encode()).hexdigest()
