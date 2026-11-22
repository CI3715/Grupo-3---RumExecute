import hashlib
import json
from dataclasses import dataclass
from decimal import Decimal

from dominio.errores import ErrorConflicto, ErrorDominio


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

    def __post_init__(self):
        for campo in ("clave", "usuario_id", "huella", "gasto_id"):
            valor = getattr(self, campo)

            if not isinstance(valor, str) or not valor.strip():
                raise ErrorDominio(campo, f"El campo {campo} es obligatorio.")


def calcular_huella(
    grupo_id: str,
    concepto: str,
    pagador_id: str,
    monto: Decimal,
    tipo_division: str,
    participantes_ids: list[str],
    valores: dict[str, Decimal] | None = None
) -> str:

    if not isinstance(monto, Decimal) or not monto.is_finite():
        raise ErrorDominio("monto", "El monto debe ser un decimal finito.")

    for valor in (valores or {}).values():
        if not isinstance(valor, Decimal) or not valor.is_finite():
            raise ErrorDominio("valores", "Los valores deben ser decimales finitos.")

    datos = {
        "grupo_id": grupo_id,
        "concepto": concepto.strip(),
        "pagador_id": pagador_id,
        "monto": format(monto.normalize(), "f"),
        "tipo_division": tipo_division,
        "valores": {
            uid: format(valor.normalize(), "f")
            for uid, valor in (valores or {}).items()
        },
        "participantes_ids": sorted(participantes_ids),
    }

    texto = json.dumps(
        datos,
        sort_keys=True,
        ensure_ascii=False
    )

    return hashlib.sha256(texto.encode("utf-8")).hexdigest()


def verificar_reintento(
    registro: RegistroIdempotenciaGasto,
    usuario_id: str,
    huella: str
) -> str:
    """
    Comprueba que la operación corresponde al registro existente.

    Devuelve el ID del gasto ya registrado si los datos coinciden.
    Lanza ErrorConflictoIdempotencia si son diferentes.
    """

    if registro.usuario_id != usuario_id or registro.huella != huella:
        raise ErrorConflictoIdempotencia()

    return registro.gasto_id
