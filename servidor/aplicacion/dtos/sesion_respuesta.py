from dataclasses import dataclass
from datetime import datetime

from dominio.usuario import Usuario


@dataclass(frozen=True)
class SesionRespuesta:
    usuario: Usuario
    token: str
    fecha_expiracion: datetime