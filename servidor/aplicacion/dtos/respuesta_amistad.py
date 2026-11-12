from dataclasses import dataclass


@dataclass(frozen=True)
class RespuestaAmistad:
    id: str
    estado: str