import hashlib
import json
from dataclasses import dataclass;
from decimal import Decimal;


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
        "tipo_division":  tipo_division,
        "valores": {uid: format(v.normalize(), "f") for uid, v in (valores or {}).items()},
        "participantes_ids": sorted(participantes_ids),
    }
    texto = json.dumps(datos, sort_keys=True)
    return hashlib.sha256(texto.encode()).hexdigest()