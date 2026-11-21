from dataclasses import dataclass;

@dataclass(frozen=True)
class RegistroIdempotenciaGasto:
    clave: str
    usuario_id: str
    huella: str
    gasto_id: str


